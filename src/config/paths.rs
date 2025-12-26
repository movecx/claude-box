use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

/// Platform-specific paths for claude-box
pub struct Paths {
    project_dirs: ProjectDirs,
}

impl Paths {
    pub fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "claude-box", "claude-box")
            .ok_or_else(|| anyhow!("Could not determine project directories"))?;
        Ok(Self { project_dirs })
    }

    /// Config directory (for config.toml)
    /// - Linux: ~/.config/claude-box/
    /// - macOS: ~/Library/Application Support/claude-box/
    /// - Windows: %APPDATA%\claude-box\
    pub fn config_dir(&self) -> PathBuf {
        self.project_dirs.config_dir().to_path_buf()
    }

    /// Config file path
    pub fn config_file(&self) -> PathBuf {
        self.config_dir().join("config.toml")
    }

    /// Data directory (for environments)
    /// - Linux: ~/.local/share/claude-box/
    /// - macOS: ~/Library/Application Support/claude-box/
    /// - Windows: %LOCALAPPDATA%\claude-box\
    pub fn data_dir(&self) -> PathBuf {
        self.project_dirs.data_dir().to_path_buf()
    }

    /// Environments directory
    pub fn environments_dir(&self) -> PathBuf {
        self.data_dir().join("environments")
    }

    /// Get path for a specific environment
    pub fn environment_dir(&self, name: &str) -> PathBuf {
        self.environments_dir().join(name)
    }

    /// Get the claude-data directory for an environment (used as CLAUDE_CONFIG_DIR)
    pub fn claude_data_dir(&self, env_name: &str) -> PathBuf {
        self.environment_dir(env_name).join("claude-data")
    }
}
