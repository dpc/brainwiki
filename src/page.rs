use markdown;
use std::fs;
use std::path::Path;

use Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page {
    pub title: String,
    pub html: String,
    pub md: String,
    pub tags: Vec<String>,
}

impl Page {
    pub fn from_markdown(markdown: String, path: &Path) -> Result<Self> {
        let (tags, html, title) = markdown::parse_markdown(&markdown);

        let page = Page {
            html: html,
            md: markdown,
            title: if title.is_empty() {
                tags.join("/")
            } else {
                title
            },
            tags: tags,
        };

        Ok(page)
    }
    pub fn read_from_file(path: &Path) -> Result<Self> {
        let md = fs::read_to_string(path)?;

        Self::from_markdown(md, path)
    }

    pub fn url(&self) -> String {
        "/".to_string() + self.tags.join("/").as_str()
    }

    pub fn to_full_url(&self, prefer_exact: bool) -> String {
        let mut location = String::from("/") + self.tags.join("/").as_str();
        if !prefer_exact {
            location += "/";
        }

        location
    }
}
