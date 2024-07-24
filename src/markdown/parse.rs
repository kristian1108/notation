use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::{Parser};
use markdown::mdast::{List, Node, Paragraph, Table};
use markdown::ParseOptions;
use reqwest::Url;

use crate::markdown::util::split_args;
use crate::notion::block::{AppendBlockRequest, AppendBlockRequestChild, BlockType, NotionBlock, TextAnnotations};
use crate::notion::language::NotionCodeLanguage;

pub static MAX_CODE_LENGTH: usize = 2000;

#[derive(Debug, Clone)]
pub struct NotationParseResult {
    inner: Node,
    path: String,
    file_name: String,
}

#[derive(Debug, Clone, Parser)]
pub struct NotationDocArguments {
    #[clap(short, long, value_parser)]
    pub emoji: Option<String>,
    #[clap(short, long, value_parser)]
    pub title: Option<String>,
}

impl Default for NotationDocArguments {
    fn default() -> Self {
        NotationDocArguments {
            emoji: None,
            title: None,
        }
    }
}

pub fn build_paragraph(
    p: &Paragraph,
    file_path: &String,
    page_id: &String,
    path_to_page_id: &HashMap<PathBuf, String>,
    page_title: &String,
) -> Result<Vec<AppendBlockRequestChild>> {
    let mut pblocks = Vec::new();
    let mut request_children = Vec::new();
    let mut first_content_line = 0;

    for c in p.children.iter() {
        match c {
            Node::Text(t) => {
                if t.value.starts_with("--") {
                    if let Some(p) = &t.position {
                        if p.start.line == 1 {
                            continue;
                        }
                    }
                }
                let parsed_content = t.value.replace("\n", " ");
                if first_content_line == 0 {
                    if t.value.trim().is_empty() {
                        continue;
                    } else if let Some(p) = &t.position {
                        first_content_line = p.start.line;
                    }
                }
                pblocks.push(NotionBlock::new_text_block(parsed_content))
            }
            Node::Link(l) => {
                let link_url = l.url.clone();
                let use_url = if link_url.starts_with("#") {
                    format!("https://www.notion.so/{}", page_id)
                } else if link_url.starts_with(".") {
                    let page_url: Vec<&str> = l.url.split("#").collect();
                    let relative_path =
                        PathBuf::from_str(page_url.first().unwrap_or(&l.url.as_str()))?;
                    let base_path = PathBuf::from_str(file_path.as_str())?;
                    let base_path = base_path.parent().unwrap_or(base_path.as_path());
                    let full_path = base_path.join(relative_path);
                    let full_path = reconcile_path(&full_path)?;
                    if let Some(pid) = path_to_page_id.get(&full_path) {
                        let formatted_pid = pid.replace("-", "");
                        let formatted_page_title = page_title.replace(" ", "-");
                        format!(
                            "https://www.notion.so/{}-{}",
                            formatted_page_title, formatted_pid
                        )
                    } else {
                        return Err(anyhow!("(page={}) failed to build paragraph, detected invalid link url: {}, found no fallback alternative", file_path, l.url.clone()));
                    }
                } else {
                    link_url.clone()
                };

                Url::parse(use_url.as_str()).map_err(|e| anyhow!("(page={}) failed to build paragraph, detected invalid link url: {}, err: {:?}", file_path, l.url.clone(), e))?;

                let text = l.children.first();

                if let Some(t) = text {
                    if let Node::Text(t) = t {
                        pblocks.push(NotionBlock::new_link_block(t.value.clone(), use_url))
                    }
                } else {
                    pblocks.push(NotionBlock::new_link_block(l.url.clone(), use_url))
                }
            }
            Node::Image(i) => {
                if !pblocks.is_empty() {
                    request_children.push(AppendBlockRequestChild::new_rich_text(
                        BlockType::Paragraph,
                        pblocks.clone(),
                    ));
                    pblocks.clear();
                }
                Url::parse(i.url.as_str()).map_err(|e| anyhow!("(page={}) failed to build paragraph, detected invalid image url: {}, err: {:?}", file_path, i.url.clone(), e))?;
                request_children.push(AppendBlockRequestChild::new_external_image_block(
                    i.url.clone(),
                ));
            }
            Node::Strong(s) => {
                for sc in s.children.iter() {
                    match sc {
                        Node::Text(t) => {
                            let parsed_content = t.value.replace("\n", " ");
                            pblocks.push(NotionBlock::new_text_block(parsed_content).with_annotations(TextAnnotations::bold()))
                        }
                        _ => {}
                    }
                }
            }
            Node::InlineCode(c) => {
                pblocks.push(NotionBlock::new_text_block(c.value.replace("\n", " ")).with_annotations(TextAnnotations::code()))
            }
            Node::InlineMath(m) => {
                pblocks.push(NotionBlock::new_text_block(m.value.replace("\n", " ")).with_annotations(TextAnnotations::code()))
            }
            _ => {}
        }
    }

    if !pblocks.is_empty() {
        request_children.push(AppendBlockRequestChild::new_rich_text(
            BlockType::Paragraph,
            pblocks,
        ));
    }

    Ok(request_children)
}

pub fn build_list(
    list: &List,
    file_path: &String,
    page_id: &String,
    path_to_page_id: &HashMap<PathBuf, String>,
    page_title: &String,
) -> Result<Vec<AppendBlockRequestChild>> {
    let mut children = Vec::new();

    for c in list.children.iter() {
        match c {
            Node::ListItem(li) => {
                for cc in li.children.iter() {
                    match cc {
                        Node::Paragraph(p) => {
                            let paragraph_blocks = build_paragraph(p, file_path, page_id, path_to_page_id, page_title)?;
                            let mut lblocks = Vec::new();
                            for p in paragraph_blocks {
                                if let Some(rtb) = p.get_rich_text_blocks() {
                                    lblocks.extend(rtb);
                                }
                            }
                            let block_type = if list.ordered {
                                BlockType::NumberedListItem
                            } else {
                                BlockType::BulletedListItem
                            };
                            children.push(AppendBlockRequestChild::new_rich_text(block_type, lblocks));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(children)
}

pub fn build_table(table: &Table) -> Vec<AppendBlockRequestChild> {
    let mut rows = Vec::new();
    let mut table_length = 0;

    for r in table.children.iter() {
        let mut cells = Vec::new();
        match r {
            Node::TableRow(tr) => {
                if tr.children.len() > table_length {
                    table_length = tr.children.len();
                }
                for c in tr.children.iter() {
                    match c {
                        Node::TableCell(tc) => {
                            for cc in tc.children.iter() {
                                match cc {
                                    Node::Text(it) => {
                                       let parsed_content = it.value.replace("\n", " ");
                                        cells.push(NotionBlock::new_text_block(parsed_content))
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        rows.push(AppendBlockRequestChild::new_table_row_block(cells))
    }

    vec!(AppendBlockRequestChild::new_table_block(table_length, true, true, rows))
}

pub fn recurse_markdown_tree(
    request: &mut AppendBlockRequest,
    node: &Node,
    parent: &Node,
    path: &String,
    page_id: &String,
    path_to_page_id: &HashMap<PathBuf, String>,
    page_title: &String,
) -> Result<()> {
    match node {
        Node::Heading(h) => {
            for c in h.children.iter() {
                recurse_markdown_tree(
                    request,
                    c,
                    node,
                    path,
                    page_id,
                    path_to_page_id,
                    page_title,
                )?;
            }
        }
        Node::List(l) => {
            request.extend_children(build_list(
                l,
                path,
                page_id,
                path_to_page_id,
                page_title,
            )?);
        }
        Node::ListItem(li) => {
            for c in li.children.iter() {
                recurse_markdown_tree(
                    request,
                    c,
                    parent,
                    path,
                    page_id,
                    path_to_page_id,
                    page_title,
                )?;
            }
        }
        Node::Text(t) => match parent {
            Node::Heading(h) => {
                request.append_child(AppendBlockRequestChild::new_heading_block(
                    t.value.clone(),
                    h.depth,
                ));
            }
            Node::Root(_) => {
                request.append_child(AppendBlockRequestChild::new_paragraph_block(
                    t.value.clone(),
                ));
            }
            Node::ListItem(_) => {
                request.append_child(AppendBlockRequestChild::new_bulleted_list_item_block(
                    t.value.clone(),
                ));
            }
            Node::List(l) => {
                if l.ordered {
                    request.append_child(AppendBlockRequestChild::new_numbered_list_item_block(
                        t.value.clone(),
                    ));
                } else {
                    request.append_child(AppendBlockRequestChild::new_bulleted_list_item_block(
                        t.value.clone(),
                    ));
                }
            }
            _ => {}
        },
        Node::Paragraph(p) => {
            request.extend_children(build_paragraph(
                p,
                path,
                page_id,
                path_to_page_id,
                page_title,
            )?);
        }
        Node::Code(c) => {
            let mut code_chunks = Vec::new();
            for chunk in c.value.as_bytes().chunks(MAX_CODE_LENGTH) {
                code_chunks.push(String::from(std::str::from_utf8(chunk).unwrap()));
            }

            let code_language_string = c.lang.clone().unwrap_or(String::from("plain text"));
            let parsed_code_language = NotionCodeLanguage::from_str(code_language_string.as_str())
                .unwrap_or(NotionCodeLanguage::PlainText);

            request.append_child(AppendBlockRequestChild::new_code_block(
                code_chunks,
                parsed_code_language.to_string(),
            ));
        }
        Node::Root(r) => {
            for c in r.children.iter() {
                recurse_markdown_tree(
                    request,
                    c,
                    node,
                    path,
                    page_id,
                    path_to_page_id,
                    page_title,
                )?;
            }
        }
        Node::Table(t) => {
            request.extend_children(build_table(t));
        }
        _ => {}
    }

    Ok(())
}

impl NotationParseResult {
    pub fn new(n: Node, path: String) -> Result<Self> {
        let pb = PathBuf::from_str(path.as_str())?;
        let file_name = pb.file_stem().unwrap().to_str().unwrap().to_string();
        Ok(NotationParseResult {
            inner: n,
            path,
            file_name,
        })
    }

    pub fn to_notion(
        &self,
        page_id: &String,
        path_to_page_id: &HashMap<PathBuf, String>,
    ) -> Result<AppendBlockRequest> {
        let mut request = AppendBlockRequest::new_children(vec![]);
        recurse_markdown_tree(
            &mut request,
            &self.inner,
            &self.inner,
            &self.path,
            page_id,
            path_to_page_id,
            &self
                .get_arguments()?
                .title
                .unwrap_or(self.file_name.clone()),
        )?;
        Ok(request)
    }

    pub fn get_arguments(&self) -> Result<NotationDocArguments> {
        if let Some(c) = self.inner.children() {
            let first_line = c.first();
            if let Some(fl) = first_line {
                if let Node::Paragraph(p) = fl {
                    for pc in p.children.iter() {
                        if let Node::Text(t) = pc {
                            let arg_value = format!("bin {}", t.value.as_str());
                            let args = NotationDocArguments::try_parse_from(
                                split_args(arg_value.as_str()).iter(),
                            )?;
                            return Ok(args);
                        }
                    }
                }
            }
        }
        Ok(NotationDocArguments::default())
    }
}

pub fn reconcile_path(path: &PathBuf) -> Result<PathBuf> {
    let mut p = PathBuf::new();
    for c in path.components() {
        match c {
            Component::Normal(n) => {
                let component_string = n
                    .to_str()
                    .ok_or(anyhow!("failed to convert path component to string"))?;
                let component_string = component_string
                    .strip_prefix("\"")
                    .unwrap_or(component_string);
                let component_string = component_string
                    .strip_suffix("\"")
                    .unwrap_or(component_string);
                let decoded_str = percent_encoding::percent_decode_str(component_string)
                    .decode_utf8()
                    .map_err(|e| anyhow!("failed to decode path component: {:?}", e))?;
                p.push(decoded_str.into_owned());
            }
            Component::ParentDir => {
                p.pop();
            }
            _ => {}
        }
    }

    Ok(p)
}

pub async fn parse_file(path: &Path) -> Result<NotationParseResult> {
    let contents = tokio::fs::read_to_string(path).await?;
    let parsing_options = ParseOptions::gfm();
    let pr = markdown::to_mdast(&contents, &parsing_options).map_err(|e| anyhow::anyhow!(e))?;
    Ok(NotationParseResult::new(pr, format!("{path:?}"))?)
}

pub fn get_md_glob_pattern(dir: String) -> String {
    if dir.ends_with(".md") {
        dir.clone()
    } else {
        format!("{}/**/*.md", dir.strip_suffix("/").unwrap_or(dir.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::markdown::parse::NotationDocArguments;
    use crate::markdown::util::split_args;

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_doc_arguments() {
        let arg_string = "bin --emoji 😮‍💨";
        let args = NotationDocArguments::parse_from(split_args(arg_string).iter());
        println!("{:?}", args);
    }

    #[tokio::test(flavor = "multi_thread")]
    pub async fn test_trim() {
        let arg_string = "\n\n\n";
        assert!(arg_string.trim().is_empty());
    }
}
