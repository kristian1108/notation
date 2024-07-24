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
        let config_path = std::env::var("NOTATION_CONFIG").unwrap_or("./Notation.toml".to_string());
        let path_buf = std::path::PathBuf::from(&config_path);
        let s = Config::builder()
            .add_source(config::File::from(path_buf))
            .add_source(config::Environment::with_prefix("NOTATION"))
            .build()?;
        let result: Self = s.try_deserialize()?;
        Ok(result)
    }
}
