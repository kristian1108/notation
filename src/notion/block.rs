use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Paragraph,
    #[serde(rename = "heading_1")]
    Heading1,
    #[serde(rename = "heading_2")]
    Heading2,
    #[serde(rename = "heading_3")]
    Heading3,
    Code,
    BulletedListItem,
    NumberedListItem,
    Image,
    Table,
    TableRow,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AppendBlockRequest {
    pub children: Vec<AppendBlockRequestChild>,
}

impl AppendBlockRequest {
    pub fn new() -> Self {
        AppendBlockRequest { children: vec![] }
    }

    pub fn new_child(child: AppendBlockRequestChild) -> Self {
        AppendBlockRequest {
            children: vec![child],
        }
    }

    pub fn new_children(children: Vec<AppendBlockRequestChild>) -> Self {
        AppendBlockRequest { children }
    }

    pub fn append_child(&mut self, child: AppendBlockRequestChild) {
        self.children.push(child);
    }

    pub fn extend_children(&mut self, children: Vec<AppendBlockRequestChild>) {
        self.children.extend(children);
    }

    pub fn children(&self) -> Vec<AppendBlockRequestChild> {
        self.children.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendBlockRequestChild {
    pub object: String,
    #[serde(rename = "type")]
    pub block_type: BlockType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_1: Option<RichTextParent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_2: Option<RichTextParent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading_3: Option<RichTextParent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<RichTextParent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<RichTextParent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulleted_list_item: Option<RichTextParent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub numbered_list_item: Option<RichTextParent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ImageParent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<TableParent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_row: Option<TableRowParent>,
}

pub fn get_heading_text(
    field_depth: u8,
    requested_depth: u8,
    content: String,
) -> Option<RichTextParent> {
    if requested_depth == field_depth || (field_depth == 3 && requested_depth > 3) {
        Some(RichTextParent::new_text(content))
    } else {
        None
    }
}

impl AppendBlockRequestChild {
    pub fn new(block_type: BlockType) -> Self {
        AppendBlockRequestChild {
            object: "block".to_string(),
            block_type,
            heading_1: None,
            heading_2: None,
            heading_3: None,
            paragraph: None,
            code: None,
            bulleted_list_item: None,
            numbered_list_item: None,
            image: None,
            table: None,
            table_row: None,
        }
    }

    pub fn get_rich_text_blocks(&self) -> Option<Vec<NotionBlock>>
    {
        if let Some(h) = &self.heading_1 {
            Some(h.get_blocks())
        } else if let Some(h) = &self.heading_2 {
            Some(h.get_blocks())
        } else if let Some(h) = &self.heading_3 {
            Some(h.get_blocks())
        } else if let Some(p) = &self.paragraph {
            Some(p.get_blocks())
        } else if let Some(c) = &self.code {
            Some(c.get_blocks())
        } else if let Some(b) = &self.bulleted_list_item {
            Some(b.get_blocks())
        } else if let Some(n) = &self.numbered_list_item {
            Some(n.get_blocks())
        } else {
            None
        }
    }

    pub fn new_rich_text(block_type: BlockType, rich_text: Vec<NotionBlock>) -> Self {
        AppendBlockRequestChild::new(block_type).with_rich_text(rich_text)
    }

    pub fn new_paragraph_block(content: String) -> Self {
        let formatted_content = content.replace("\n", " ");
        AppendBlockRequestChild {
            object: "block".to_string(),
            block_type: BlockType::Paragraph,
            heading_1: None,
            heading_2: None,
            heading_3: None,
            paragraph: Some(RichTextParent::new_text(formatted_content)),
            code: None,
            bulleted_list_item: None,
            numbered_list_item: None,
            image: None,
            table: None,
            table_row: None,
        }
    }

    pub fn new_heading_block(content: String, depth: u8) -> Self {
        let block_type = match depth {
            1 => BlockType::Heading1,
            2 => BlockType::Heading2,
            3 => BlockType::Heading3,
            _ => BlockType::Heading3,
        };

        AppendBlockRequestChild {
            object: "block".to_string(),
            block_type,
            heading_1: get_heading_text(1, depth, content.clone()),
            heading_2: get_heading_text(2, depth, content.clone()),
            heading_3: get_heading_text(3, depth, content),
            paragraph: None,
            code: None,
            bulleted_list_item: None,
            numbered_list_item: None,
            image: None,
            table: None,
            table_row: None,
        }
    }

    pub fn new_code_block(content: Vec<String>, language: String) -> Self {
        AppendBlockRequestChild {
            object: "block".to_string(),
            block_type: BlockType::Code,
            heading_1: None,
            heading_2: None,
            heading_3: None,
            paragraph: None,
            code: Some(RichTextParent::new_code(content, language)),
            bulleted_list_item: None,
            numbered_list_item: None,
            image: None,
            table: None,
            table_row: None,
        }
    }

    pub fn new_bulleted_list_item_block(content: String) -> Self {
        AppendBlockRequestChild {
            object: "block".to_string(),
            block_type: BlockType::BulletedListItem,
            heading_1: None,
            heading_2: None,
            heading_3: None,
            paragraph: None,
            code: None,
            bulleted_list_item: Some(RichTextParent::new_text(content)),
            numbered_list_item: None,
            image: None,
            table: None,
            table_row: None,
        }
    }

    pub fn new_numbered_list_item_block(content: String) -> Self {
        AppendBlockRequestChild {
            object: "block".to_string(),
            block_type: BlockType::NumberedListItem,
            heading_1: None,
            heading_2: None,
            heading_3: None,
            paragraph: None,
            code: None,
            bulleted_list_item: None,
            numbered_list_item: Some(RichTextParent::new_text(content)),
            image: None,
            table: None,
            table_row: None,
        }
    }

    pub fn new_external_image_block(url: String) -> Self {
        AppendBlockRequestChild {
            object: "block".to_string(),
            block_type: BlockType::Image,
            heading_1: None,
            heading_2: None,
            heading_3: None,
            paragraph: None,
            code: None,
            bulleted_list_item: None,
            numbered_list_item: None,
            image: Some(ImageParent {
                image_type: "external".to_string(),
                external: ExternalImageInner { url },
            }),
            table: None,
            table_row: None,
        }
    }

    pub fn new_table_block(
        table_width: usize,
        has_column_header: bool,
        has_row_header: bool,
        rows: Vec<AppendBlockRequestChild>,
    ) -> Self {
        AppendBlockRequestChild {
            object: "block".to_string(),
            block_type: BlockType::Table,
            heading_1: None,
            heading_2: None,
            heading_3: None,
            paragraph: None,
            code: None,
            bulleted_list_item: None,
            numbered_list_item: None,
            image: None,
            table: Some(TableParent {
                table_width,
                has_column_header,
                has_row_header,
                children: rows,
            }),
            table_row: None,
        }
    }

    pub fn new_table_row_block(cells: Vec<NotionBlock>) -> Self {
        let mut formatted_cells = Vec::new();
        for c in cells {
            formatted_cells.push(vec![c]);
        }
        AppendBlockRequestChild {
            object: "block".to_string(),
            block_type: BlockType::TableRow,
            heading_1: None,
            heading_2: None,
            heading_3: None,
            paragraph: None,
            code: None,
            bulleted_list_item: None,
            numbered_list_item: None,
            image: None,
            table: None,
            table_row: Some(TableRowParent { cells: formatted_cells }),
        }
    }

    pub fn with_rich_text(mut self, rich_text: Vec<NotionBlock>) -> Self {
        match self.block_type {
            BlockType::NumberedListItem => {
                self.numbered_list_item = Some(RichTextParent::new(rich_text));
            }
            BlockType::BulletedListItem => {
                self.bulleted_list_item = Some(RichTextParent::new(rich_text));
            }
            BlockType::Paragraph => {
                self.paragraph = Some(RichTextParent::new(rich_text));
            }
            BlockType::Heading1 => {
                self.heading_1 = Some(RichTextParent::new(rich_text));
            }
            BlockType::Heading2 => {
                self.heading_2 = Some(RichTextParent::new(rich_text));
            }
            BlockType::Heading3 => {
                self.heading_3 = Some(RichTextParent::new(rich_text));
            }
            BlockType::Code => {
                self.code = Some(RichTextParent::new(rich_text));
            }
            _ => {}
        }

        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichTextParent {
    pub rich_text: Vec<NotionBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageParent {
    #[serde(rename = "type")]
    pub image_type: String,
    pub external: ExternalImageInner,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableParent {
    pub table_width: usize,
    pub has_column_header: bool,
    pub has_row_header: bool,
    pub children: Vec<AppendBlockRequestChild>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableRowParent {
    pub cells: Vec<Vec<NotionBlock>>,
}

impl RichTextParent {
    pub fn new(rich_text: Vec<NotionBlock>) -> Self {
        RichTextParent {
            rich_text,
            language: None,
        }
    }

    pub fn new_text(content: String) -> Self {
        RichTextParent {
            rich_text: vec![NotionBlock::new_text_block(content)],
            language: None,
        }
    }

    pub fn new_code(content: Vec<String>, language: String) -> Self {
        let mut rich_text = Vec::new();
        for c in content {
            rich_text.push(NotionBlock::new_code_block(c))
        }

        RichTextParent {
            rich_text,
            language: Some(language),
        }
    }

    pub fn get_blocks(&self) -> Vec<NotionBlock>
    {
        self.rich_text.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotionBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<TextAnnotations>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalImageInner {
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextBlock {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<TextLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextLink {
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextAnnotations {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub code: bool,
    pub color: String,
}

impl TextAnnotations {
    pub fn bold() -> Self {
        TextAnnotations {
            bold: true,
            italic: false,
            strikethrough: false,
            underline: false,
            code: false,
            color: "default".to_string(),
        }
    }

    pub fn code() -> Self {
        TextAnnotations {
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            code: true,
            color: "default".to_string(),
        }
    }
}

impl NotionBlock {
    pub fn new_text_block(content: String) -> Self {
        NotionBlock {
            block_type: "text".to_string(),
            text: Some(TextBlock {
                content,
                link: None,
            }),
            annotations: None,
        }
    }

    pub fn new_link_block(content: String, link: String) -> Self {
        NotionBlock {
            block_type: "text".to_string(),
            text: Some(TextBlock {
                content,
                link: Some(TextLink { url: link }),
            }),
            annotations: None,
        }
    }

    pub fn new_code_block(content: String) -> Self {
        NotionBlock {
            block_type: "text".to_string(),
            text: Some(TextBlock {
                content,
                link: None,
            }),
            annotations: None,
        }
    }

    pub fn with_annotations(mut self, annotations: TextAnnotations) -> Self {
        self.annotations = Some(annotations);
        self
    }
}
