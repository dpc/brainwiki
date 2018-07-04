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
    pub fn from_markdown(markdown: String) -> Self {
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

        page
    }
    pub fn read_from_file(path: &Path) -> Result<Self> {
        let md = fs::read_to_string(path)?;

        Ok(Self::from_markdown(md))
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

    pub fn suggested_filename(&self) -> String {
        let mut in_break = true;
        let mut filename = String::new();

        for ch in self.title.chars() {
            if ch.is_alphanumeric() {
                filename.push_str(ch.to_lowercase().to_string().as_str());
                in_break = false;
            } else {
                if !in_break {
                    in_break = true;
                    filename.push('_');
                }
            }
        }
        filename
    }
}
