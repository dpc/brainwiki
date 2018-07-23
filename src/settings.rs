use crate::Result;
use std::path::{Path, PathBuf};

use crate::util::{deserialize_as_hex, serialize_as_hex};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Site {
    pub version: u32,
    #[serde(default)]
    pub footer: String,
    #[serde(default)]
    pub short_name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub hashed_password: Option<String>,
    #[serde(
        default = "rand_salt",
        serialize_with = "serialize_as_hex",
        deserialize_with = "deserialize_as_hex"
    )]
    pub web_salt: Vec<u8>,
}

fn rand_salt() -> Vec<u8> {
    use ring::rand::SecureRandom;
    let mut salt = vec![1u8; 32];
    ring::rand::SystemRandom::new().fill(&mut salt).unwrap();

    salt
}
impl Default for Site {
    fn default() -> Self {
        Site {
            footer: "Powered by <a href=\"http://github.com/dpc/brainwiki\">BrainWiki</a>".into(),
            short_name: "BrainWiki".into(),
            author: "".into(),
            hashed_password: None,
            version: 0,
            web_salt: rand_salt(),
        }
    }
}

impl Site {
    pub fn dir_to_file(dir: &Path) -> PathBuf {
        dir.join("config.toml")
    }

    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let file_path = Self::dir_to_file(dir);

        if file_path.exists() {
            Self::load_from(&file_path).into()
        } else {
            Ok(Default::default())
        }
    }

    pub fn load_from(file_path: &Path) -> Result<Self> {
        let content = file::get(file_path)?;

        Ok(toml::from_slice(&content)?)
    }

    pub fn set_password(&mut self, cleartext: String) {
        self.hashed_password =
            Some(libpasta::hash_password(cleartext));
    }

    pub fn write_to_dir(&self, dir: &Path) -> Result<()> {
        let file_path = Self::dir_to_file(dir);
        file::put(file_path, toml::to_vec(&self)?)?;
        Ok(())
    }
}
