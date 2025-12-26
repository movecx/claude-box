mod paths;
mod schema;

pub use paths::Paths;
pub use schema::{Config, EnvironmentConfig, ProviderConfig, ProviderModels, ProviderPreset};

use anyhow::Result;
use std::fs;

impl Config {
    /// Load config from the default location, or create default if not exists
    pub fn load() -> Result<Self> {
        let paths = Paths::new()?;
        let config_path = paths.config_file();

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Save config to the default location
    pub fn save(&self) -> Result<()> {
        let paths = Paths::new()?;
        let config_path = paths.config_file();

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}
