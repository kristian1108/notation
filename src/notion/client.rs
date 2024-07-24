use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use anyhow::Result;
use glob::glob;
use reqwest::{Client, ClientBuilder, header, StatusCode};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{json, to_string, Value};

use crate::generate_random_string;
use crate::markdown::parse::{get_md_glob_pattern, NotationDocArguments, parse_file};
use crate::notion::block::AppendBlockRequest;
use crate::notion::page::{
    CreatePageRequest, CreatePageResponse, GetPageContentResponse, PageContentType,
};
use crate::notion::search::{SearchRequest, SearchResult, SearchResultItem};
use crate::settings::notation::{NotationSettings};

#[derive(Clone)]
pub struct NotionClient {
    client: Client,
    base_endpoint: String,
    parent_page_name: String,
}

const NOTION_VERSION: &str = "2022-06-28";
const NOTION_BASE_URL: &str = "https://api.notion.com/v1";
const INTRO_FILENAME: &str = "intro";

impl NotionClient {
    pub fn new() -> Result<Self> {
        let settings = NotationSettings::new()?;
        let mut headers = HeaderMap::new();
        headers.insert("Notion-Version", HeaderValue::from_static(NOTION_VERSION));
        let mut auth_value =
            HeaderValue::from_str(&format!("Bearer {}", settings.notion.secret.clone()))
                .map_err(|e| anyhow!(e))?;
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);
        let client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .map_err(|e| anyhow!(e))?;

        Ok(NotionClient {
            client,
            base_endpoint: NOTION_BASE_URL.to_string(),
            parent_page_name: settings.notion.parent_page.clone(),
        })
    }

    pub fn parent_page_name(&self) -> String {
        self.parent_page_name.clone()
    }

    pub async fn create_page_by_parent_name(
        &self,
        parent_name: String,
        page_name: String,
        emoji: Option<String>,
    ) -> Result<String> {
        let parent_id = self.get_parent_id_by_name(parent_name).await?;
        self.create_page_by_parent_id(parent_id, page_name, emoji)
            .await
    }

    pub async fn create_page_by_parent_id(
        &self,
        parent_id: String,
        page_name: String,
        emoji: Option<String>,
    ) -> Result<String> {
        let url = format!("{}/pages", self.base_endpoint);
        let mut create_page_request = CreatePageRequest::new(parent_id, page_name);
        if let Some(emoji) = emoji {
            create_page_request = create_page_request.with_icon(emoji);
        }

        let response = self
            .client
            .post(&url)
            .json(&create_page_request)
            .send()
            .await?;
        let parsed_response: CreatePageResponse = response.json().await?;

        Ok(parsed_response.id.clone())
    }

    pub async fn get_parent_id_by_name(&self, parent_name: String) -> Result<String> {
        let search_result = self.find_page_by_name(parent_name).await?;
        if search_result.len() != 1 {
            let result_urls = search_result
                .iter()
                .map(|r| r.url.clone())
                .collect::<Vec<String>>()
                .join(", ");
            return Err(anyhow!(
                "need to match exactly one parent page, found {} results ({})",
                search_result.len(),
                result_urls
            ));
        }
        let parent_id = search_result[0].id.clone();
        Ok(parent_id)
    }

    pub async fn delete(&self, resource_id: String, resource_type: &PageContentType) -> Result<()> {
        let url = match resource_type {
            PageContentType::ChildPage => format!("{}/pages/{}", self.base_endpoint, resource_id),
            _ => format!("{}/blocks/{}", self.base_endpoint, resource_id),
        };
        let archive_body = json!({
            "in_trash": true,
        });
        self.client.patch(&url).json(&archive_body).send().await?;
        Ok(())
    }

    pub async fn append_block(
        &self,
        page_or_block_id: String,
        request: &AppendBlockRequest,
    ) -> Result<()> {
        let url = format!(
            "{}/blocks/{}/children",
            self.base_endpoint, page_or_block_id
        );
        let r = self.client.patch(&url).json(request).send().await?;
        let status = r.status();
        if status != StatusCode::OK {
            let response: Value = r.json().await?;
            return Err(anyhow!(
                "(request_status={}) failed to append block: {}",
                status,
                to_string(&response)?
            ));
        }
        Ok(())
    }

    pub async fn find_page_by_name(&self, page_name: String) -> Result<Vec<SearchResultItem>> {
        let lower_name = page_name.to_lowercase();
        let all_related_pages = self.find_all_pages_related_to_name(page_name).await?;
        let filtered_response: Vec<SearchResultItem> = all_related_pages
            .results
            .iter()
            .filter(|x| x.properties.title.title[0].plain_text.to_lowercase() == lower_name)
            .cloned()
            .collect();
        Ok(filtered_response)
    }

    pub async fn find_all_pages_related_to_name(&self, page_name: String) -> Result<SearchResult> {
        let search_request = SearchRequest::new(page_name);
        let endpoint = format!("{}/search", self.base_endpoint);
        let r = self
            .client
            .post(&endpoint)
            .json(&search_request)
            .send()
            .await?;
        let response: Value = r.json().await?;
        let response: SearchResult = serde_json::from_value(response)?;
        Ok(response)
    }

    pub async fn get_page_content_by_id(&self, page_id: String) -> Result<GetPageContentResponse> {
        let url = format!("{}/blocks/{}/children", self.base_endpoint, page_id);
        let response = self.client.get(&url).send().await?;
        let response: GetPageContentResponse = response.json().await?;
        Ok(response)
    }

    pub async fn clear(&self) -> Result<()> {
        let parent_id = self
            .get_parent_id_by_name(self.parent_page_name.clone())
            .await?;
        let page_details = self.get_page_content_by_id(parent_id.clone()).await?;
        for rid in page_details.results.iter() {
            self.delete(rid.id.clone(), &rid.content_type).await?;
        }

        Ok(())
    }

    pub async fn create_pages(&self, dir: String, is_simulate: bool) -> Result<()> {
        let pattern = get_md_glob_pattern(dir.clone());
        let root_page_id = self
            .get_parent_id_by_name(self.parent_page_name.clone())
            .await?;

        let mut paths_to_ids = HashMap::new();
        let mut subdir_path_to_parent_id: HashMap<PathBuf, String> = HashMap::new();

        for entry in glob(&pattern)? {
            let path = entry?;

            if path.is_file() {
                let relative_path = path.strip_prefix(dir.clone()).unwrap();
                let components: Vec<_> = relative_path.components().collect();

                let mut accumulated_components = Vec::new();

                if components.len() > 1 {
                    for component in components.iter().take(components.len() - 1) {
                        if let Some(dir_name) = component.as_os_str().to_str() {
                            let base_path = PathBuf::new().join(accumulated_components.join("/"));
                            let new_subdir_path = base_path.join(dir_name);
                            if subdir_path_to_parent_id.get(&new_subdir_path).is_none() {
                                let parent_dir_id = subdir_path_to_parent_id
                                    .get(&base_path)
                                    .unwrap_or(&root_page_id);
                                let new_dir_id = if is_simulate {
                                    generate_random_string(30)
                                } else {
                                    let parent_path = path.parent().unwrap_or(Path::new("/"));
                                    let intro_path = parent_path.join(format!("{}.md", INTRO_FILENAME));
                                    let page_args = if intro_path.exists() {
                                        let parsed_content = parse_file(&intro_path).await?;
                                        let arguments = parsed_content.get_arguments()?;
                                        arguments
                                    } else {
                                        NotationDocArguments::default()
                                    };
                                    self.create_page_by_parent_id(
                                        parent_dir_id.clone(),
                                        page_args.title.unwrap_or(dir_name.to_string()),
                                        page_args.emoji,
                                    )
                                    .await?
                                };
                                subdir_path_to_parent_id
                                    .insert(new_subdir_path.clone(), new_dir_id.clone());
                            }
                            accumulated_components.push(dir_name.to_string());
                        }
                    }
                }

                let sub_dir_path = PathBuf::new().join(accumulated_components.join("/"));
                let parent_id = subdir_path_to_parent_id
                    .get(&sub_dir_path)
                    .unwrap_or(&root_page_id);
                let parsed_content = parse_file(&path).await?;
                let arguments = parsed_content.get_arguments()?;
                let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                let page_title = arguments.title.unwrap_or(file_name.clone());

                let cr = if is_simulate {
                    generate_random_string(30)
                } else {
                    if file_name.to_lowercase().as_str() == INTRO_FILENAME {
                        parent_id.clone()
                    } else {
                        self.create_page_by_parent_id(parent_id.clone(), page_title, arguments.emoji)
                            .await?
                    }
                };

                paths_to_ids.insert(path.clone(), cr.clone());
            }
        }

        for (path, page_id) in paths_to_ids.iter() {
            let parsed_content = parse_file(&path).await?;
            let notion_request = parsed_content.to_notion(&page_id, &paths_to_ids)?;
            if !is_simulate {
                self.append_block(page_id.clone(), &notion_request).await?;
            }
        }

        Ok(())
    }
}
