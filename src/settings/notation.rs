use std::env;
use anyhow::Result;
use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct NotationSettings {
    pub notion: Notion,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Notion {
    pub secret: String,
    pub parent_page: String,
}

impl NotationSettings {
    pub fn new() -> Result<Self> {
        let config_path = env::var("NOTATION_CONFIG").unwrap_or_else(|_| {
            let mut home_dir = dirs::home_dir().expect("Could not find home directory");
            home_dir.push(".notation/Notation.toml");
            home_dir.to_str().unwrap().to_string()
        });
        let path_buf = std::path::PathBuf::from(&config_path);
        let s = Config::builder()
            .add_source(config::File::from(path_buf))
            .add_source(config::Environment::with_prefix("NOTATION"))
            .build()?;
        let result: Self = s.try_deserialize()?;
        Ok(result)
    }
}
