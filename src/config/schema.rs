use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration for claude-box
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Default environment to open when running `claude-box` without arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_environment: Option<String>,

    /// Map of environment name to configuration
    #[serde(default)]
    pub environments: HashMap<String, EnvironmentConfig>,
}

/// Known provider presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderPreset {
    /// Default Anthropic API (no custom provider)
    Anthropic,
    /// MiniMax API
    MiniMax,
    /// Z.AI API
    Zai,
    /// Custom provider configuration
    Custom,
}

impl Default for ProviderPreset {
    fn default() -> Self {
        Self::Anthropic
    }
}

impl ProviderPreset {
    pub fn all() -> &'static [ProviderPreset] {
        &[
            ProviderPreset::Anthropic,
            ProviderPreset::MiniMax,
            ProviderPreset::Zai,
            ProviderPreset::Custom,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ProviderPreset::Anthropic => "Anthropic (Default)",
            ProviderPreset::MiniMax => "MiniMax",
            ProviderPreset::Zai => "Z.AI",
            ProviderPreset::Custom => "Custom",
        }
    }

    /// Get default base URL for preset
    pub fn base_url(&self) -> Option<&'static str> {
        match self {
            ProviderPreset::Anthropic => None,
            ProviderPreset::MiniMax => Some("https://api.minimax.io/anthropic"),
            ProviderPreset::Zai => Some("https://api.z.ai/api/anthropic"),
            ProviderPreset::Custom => None,
        }
    }

    /// Get default models for preset
    pub fn default_models(&self) -> Option<ProviderModels> {
        match self {
            ProviderPreset::Anthropic => None,
            ProviderPreset::MiniMax => Some(ProviderModels {
                opus: "MiniMax-M2.1".to_string(),
                sonnet: "MiniMax-M2.1".to_string(),
                haiku: "MiniMax-M2.1".to_string(),
            }),
            ProviderPreset::Zai => Some(ProviderModels {
                opus: "glm-4.7".to_string(),
                sonnet: "glm-4.7".to_string(),
                haiku: "glm-4.5-air".to_string(),
            }),
            ProviderPreset::Custom => None,
        }
    }
}

/// Custom model names for provider
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderModels {
    pub opus: String,
    pub sonnet: String,
    pub haiku: String,
}

/// Provider configuration for custom API endpoints
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    /// Which preset to use
    #[serde(default)]
    pub preset: ProviderPreset,

    /// API key/token for the provider
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Custom base URL (only used for Custom preset)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_base_url: Option<String>,

    /// Custom model names (optional, overrides preset defaults)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models: Option<ProviderModels>,
}

impl ProviderConfig {
    /// Check if this provider requires custom configuration
    pub fn is_custom(&self) -> bool {
        self.preset != ProviderPreset::Anthropic
    }

    /// Get the effective base URL
    pub fn effective_base_url(&self) -> Option<String> {
        match self.preset {
            ProviderPreset::Custom => self.custom_base_url.clone(),
            _ => self.preset.base_url().map(|s| s.to_string()),
        }
    }

    /// Get effective models (custom override or preset default)
    pub fn effective_models(&self) -> Option<ProviderModels> {
        self.models.clone().or_else(|| self.preset.default_models())
    }
}

/// Configuration for a single environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Display name for the environment
    pub name: String,

    /// Border color in hex format (e.g., "#3498db")
    #[serde(default = "default_border_color")]
    pub border_color: String,

    /// When the environment was created
    pub created_at: DateTime<Utc>,

    /// Custom provider configuration
    #[serde(default)]
    pub provider: ProviderConfig,
}

fn default_border_color() -> String {
    "#3498db".to_string()
}

/// Special value indicating no border (run Claude directly)
pub const NO_BORDER: &str = "none";

impl EnvironmentConfig {
    /// Create a new environment config with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            border_color: default_border_color(),
            created_at: Utc::now(),
            provider: ProviderConfig::default(),
        }
    }

    /// Parse border color to ratatui Color
    pub fn border_color_rgb(&self) -> (u8, u8, u8) {
        parse_hex_color(&self.border_color).unwrap_or((52, 152, 219)) // Default blue
    }

    /// Check if this environment should run without a border
    pub fn is_no_border(&self) -> bool {
        self.border_color.eq_ignore_ascii_case(NO_BORDER)
    }
}

/// Parse a hex color string like "#3498db" to RGB tuple
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some((r, g, b))
}

impl Config {
    /// Get environment config by name
    pub fn get_environment(&self, name: &str) -> Option<&EnvironmentConfig> {
        self.environments.get(name)
    }

    /// Get the default environment name, if set and exists
    pub fn get_default_environment(&self) -> Option<&str> {
        self.default_environment.as_ref().and_then(|name| {
            if self.environments.contains_key(name) {
                Some(name.as_str())
            } else {
                None
            }
        })
    }

    /// Add a new environment
    pub fn add_environment(&mut self, key: String, config: EnvironmentConfig) {
        self.environments.insert(key, config);
    }

    /// Remove an environment
    pub fn remove_environment(&mut self, name: &str) -> Option<EnvironmentConfig> {
        // Clear default if it was the removed environment
        if self.default_environment.as_deref() == Some(name) {
            self.default_environment = None;
        }
        self.environments.remove(name)
    }

    /// Set the default environment
    pub fn set_default(&mut self, name: Option<String>) {
        self.default_environment = name;
    }

    /// List all environment names
    pub fn list_environments(&self) -> Vec<&str> {
        self.environments.keys().map(|s| s.as_str()).collect()
    }
}
