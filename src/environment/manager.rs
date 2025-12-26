use crate::config::{Config, EnvironmentConfig, Paths};
use crate::environment::create_claude_data_structure;
use anyhow::{anyhow, Result};
use std::fs;

/// Manager for creating, deleting, and listing environments
pub struct EnvironmentManager {
    paths: Paths,
    config: Config,
}

impl EnvironmentManager {
    pub fn new() -> Result<Self> {
        let paths = Paths::new()?;
        let config = Config::load()?;
        Ok(Self { paths, config })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    /// Create a new environment
    pub fn create_environment(&mut self, key: &str, name: String, border_color: Option<String>) -> Result<()> {
        // Validate key (alphanumeric, dash, underscore only)
        if !is_valid_env_key(key) {
            return Err(anyhow!(
                "Environment key must contain only letters, numbers, dashes, and underscores"
            ));
        }

        // Check if environment already exists
        if self.config.environments.contains_key(key) {
            return Err(anyhow!("Environment '{}' already exists", key));
        }

        // Create the environment directory structure
        let claude_data_dir = self.paths.claude_data_dir(key);
        create_claude_data_structure(&claude_data_dir)?;

        // Create environment config
        let mut env_config = EnvironmentConfig::new(name);
        if let Some(color) = border_color {
            env_config.border_color = color;
        }

        // Add to config and save
        self.config.add_environment(key.to_string(), env_config);
        self.config.save()?;

        Ok(())
    }

    /// Delete an environment
    pub fn delete_environment(&mut self, key: &str) -> Result<()> {
        // Remove from config (save original state for rollback)
        let default_before = self.config.default_environment.clone();
        let removed = self
            .config
            .remove_environment(key)
            .ok_or_else(|| anyhow!("Environment '{}' does not exist", key))?;
        if let Err(err) = self.config.save() {
            self.config.add_environment(key.to_string(), removed);
            self.config.set_default(default_before);
            return Err(err);
        }

        // Delete the environment directory
        let env_dir = self.paths.environment_dir(key);
        if env_dir.exists() {
            if let Err(err) = fs::remove_dir_all(&env_dir) {
                self.config.add_environment(key.to_string(), removed);
                self.config.set_default(default_before);
                if let Err(rollback_err) = self.config.save() {
                    return Err(anyhow!(
                        "Failed to delete environment directory for '{}': {}. Failed to rollback config: {}",
                        key,
                        err,
                        rollback_err
                    ));
                }
                return Err(err.into());
            }
        }

        Ok(())
    }

    /// List all environments
    pub fn list_environments(&self) -> Vec<(&str, &EnvironmentConfig)> {
        self.config
            .environments
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect()
    }

    /// Get the CLAUDE_CONFIG_DIR path for an environment
    pub fn get_claude_config_dir(&self, key: &str) -> Result<std::path::PathBuf> {
        if !self.config.environments.contains_key(key) {
            return Err(anyhow!("Environment '{}' does not exist", key));
        }
        Ok(self.paths.claude_data_dir(key))
    }

    /// Set the default environment
    pub fn set_default(&mut self, key: Option<&str>) -> Result<()> {
        if let Some(k) = key {
            if !self.config.environments.contains_key(k) {
                return Err(anyhow!("Environment '{}' does not exist", k));
            }
        }
        self.config.set_default(key.map(|s| s.to_string()));
        self.config.save()?;
        Ok(())
    }

    /// Reload config from disk
    pub fn reload(&mut self) -> Result<()> {
        self.config = Config::load()?;
        Ok(())
    }

    /// Save current config
    pub fn save(&self) -> Result<()> {
        self.config.save()
    }
}

/// Check if an environment key is valid
fn is_valid_env_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 64
        && key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}
