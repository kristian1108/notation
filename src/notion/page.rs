use std::fmt::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum PageContentType {
    Paragraph,
    ChildPage,
    #[default]
    #[serde(other)]
    Unknown,
}

impl Display for PageContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageContentType::Paragraph => write!(f, "Paragraph"),
            PageContentType::ChildPage => write!(f, "Child Page"),
            PageContentType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetPageContentResponse {
    pub results: Vec<PageContentResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageContentResult {
    #[serde(rename = "type")]
    pub content_type: PageContentType,
    pub rich_text: Option<Value>,
    pub id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreatePageResponse {
    pub id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PageParent {
    page_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PageProperties {
    title: TitleSubProperties,
}

#[derive(Clone, Serialize, Deserialize)]
struct TitleSubProperties {
    title: Vec<TitleText>,
    #[serde(rename = "type")]
    property_type: String,
    id: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct TitleText {
    #[serde(rename = "type")]
    text_type: String,
    text: TitleTextInner,
}

#[derive(Clone, Serialize, Deserialize)]
struct TitleTextInner {
    content: String,
}

impl PageProperties {
    pub fn new(title: String) -> Self {
        PageProperties {
            title: TitleSubProperties {
                title: vec![TitleText {
                    text_type: "text".to_string(),
                    text: TitleTextInner {
                        content: title.to_string(),
                    },
                }],
                property_type: "title".to_string(),
                id: "title".to_string(),
            },
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PageEmojiIcon {
    #[serde(rename = "type")]
    icon_type: String,
    emoji: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PageCover {
    #[serde(rename = "type")]
    cover_type: String,
    external: PageCoverExternalUrl,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PageCoverExternalUrl {
    url: String,
}

impl PageCover {
    pub fn new(external_url: String) -> Self {
        PageCover {
            cover_type: "external".to_string(),
            external: PageCoverExternalUrl { url: external_url },
        }
    }
}

impl PageEmojiIcon {
    pub fn new(emoji: String) -> Self {
        PageEmojiIcon {
            icon_type: "emoji".to_string(),
            emoji,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreatePageRequest {
    parent: PageParent,
    properties: PageProperties,
    children: Value,
    icon: Option<PageEmojiIcon>,
    cover: Option<PageCover>,
}

impl CreatePageRequest {
    pub fn new(parent_id: String, title: String) -> Self {
        CreatePageRequest {
            parent: PageParent { page_id: parent_id },
            properties: PageProperties::new(title),
            children: Value::Array(vec![]),
            icon: None,
            cover: None,
        }
    }

    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(PageEmojiIcon::new(icon));
        self
    }

    pub fn with_cover(mut self, cover: PageCover) -> Self {
        self.cover = Some(cover);
        self
    }

    pub fn with_children(mut self, children: Value) -> Self {
        self.children = children;
        self
    }
}
