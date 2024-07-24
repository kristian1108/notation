use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchRequest {
    query: String,
    filter: SearchFilter,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SearchFilter {
    value: String,
    property: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub results: Vec<SearchResultItem>,
}

impl Default for SearchResult {
    fn default() -> Self {
        SearchResult { results: vec![] }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResultItem {
    pub object: String,
    pub id: String,
    pub url: String,
    pub parent: SearchResultItemParent,
    pub properties: SearchResultItemProperty,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResultItemParent {
    pub page_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResultItemProperty {
    pub title: TitleInner,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TitleInner {
    pub title: Vec<TitleArrayElement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TitleArrayElement {
    pub plain_text: String,
}

impl SearchRequest {
    pub fn new(query: String) -> Self {
        SearchRequest {
            query,
            filter: SearchFilter {
                value: "page".to_string(),
                property: "object".to_string(),
            },
        }
    }
}
