use crate::Result;
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
pub struct PasswordSettings {
    pub hash: String,
    pub seed: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SiteSettings {
    #[serde(default)]
    pub footer: String,
    #[serde(default)]
    pub short_name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub password: Option<PasswordSettings>,
}

impl Default for SiteSettings {
    fn default() -> Self {
        SiteSettings {
            footer: "Powered by <a href=\"http://github.com/dpc/brainwiki\">BrainWiki</a>".into(),
            short_name: "BrainWiki".into(),
            author: "".into(),
            password: None
        }
    }
}

impl SiteSettings {
    pub fn load_or_create_in(dir: &Path) -> Result<Self> {
        let file_path = dir.join("config.toml");

        if let Ok(s) = Self::load_from(&file_path) {
            return Ok(s);
        }

        Ok(Default::default())
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        let content = file::get(path.join("config.yaml"))?;

        Ok(toml::from_slice(&content)?)
    }
}
