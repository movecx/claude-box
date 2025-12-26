use crate::config::{EnvironmentConfig, ProviderPreset};
use anyhow::{anyhow, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::path::Path;
use std::process::{Command, Stdio};

/// Wrapper around the Claude Code child process
pub struct ClaudeProcess {
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn Child + Send + Sync>,
}

/// Spawn Claude Code with the given configuration
pub fn spawn_claude_code(
    claude_config_dir: &Path,
    env_config: &EnvironmentConfig,
    size: PtySize,
    working_dir: &Path,
) -> Result<ClaudeProcess> {
    // Find the claude binary
    let claude_path = which::which("claude")
        .map_err(|_| anyhow!("Claude Code not found in PATH. Install with: npm install -g @anthropic-ai/claude-code"))?;

    // Create PTY
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(size)?;

    // Build command with environment variables
    let mut cmd = CommandBuilder::new(&claude_path);

    // Set working directory to where claude-box was invoked
    cmd.cwd(working_dir);

    // Set CLAUDE_CONFIG_DIR for isolation
    cmd.env("CLAUDE_CONFIG_DIR", claude_config_dir.to_string_lossy().to_string());

    // Pass through important system environment variables
    if let Ok(path) = std::env::var("PATH") {
        cmd.env("PATH", path);
    }
    if let Ok(term) = std::env::var("TERM") {
        cmd.env("TERM", term);
    } else {
        cmd.env("TERM", "xterm-256color");
    }
    if let Ok(home) = std::env::var("HOME") {
        cmd.env("HOME", home);
    }
    if let Ok(user) = std::env::var("USER") {
        cmd.env("USER", user);
    }
    #[cfg(windows)]
    {
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            cmd.env("USERPROFILE", userprofile);
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            cmd.env("APPDATA", appdata);
        }
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            cmd.env("LOCALAPPDATA", localappdata);
        }
        if let Ok(systemroot) = std::env::var("SystemRoot") {
            cmd.env("SystemRoot", systemroot);
        }
    }

    // Apply provider configuration if not using default Anthropic
    if env_config.provider.preset != ProviderPreset::Anthropic {
        // Set base URL if available
        if let Some(base_url) = env_config.provider.effective_base_url() {
            cmd.env("ANTHROPIC_BASE_URL", base_url);
        }

        // Set API key/token if provided
        if let Some(ref api_key) = env_config.provider.api_key {
            cmd.env("ANTHROPIC_AUTH_TOKEN", api_key);
        }

        // Always set timeout and disable non-essential traffic for custom providers
        cmd.env("API_TIMEOUT_MS", "3000000");
        cmd.env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1");

        // Set custom models if available
        if let Some(models) = env_config.provider.effective_models() {
            if !models.opus.is_empty() {
                cmd.env("ANTHROPIC_MODEL", &models.opus);
                cmd.env("ANTHROPIC_DEFAULT_OPUS_MODEL", &models.opus);
            }
            if !models.sonnet.is_empty() {
                cmd.env("ANTHROPIC_DEFAULT_SONNET_MODEL", &models.sonnet);
            }
            if !models.haiku.is_empty() {
                cmd.env("ANTHROPIC_SMALL_FAST_MODEL", &models.haiku);
                cmd.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", &models.haiku);
            }
        }
    }

    // Spawn the process
    let child = pair.slave.spawn_command(cmd)?;

    Ok(ClaudeProcess {
        master: pair.master,
        child,
    })
}

/// Run Claude Code directly without PTY wrapper (no border mode)
/// Inherits stdin/stdout/stderr for direct terminal access
pub fn run_direct_claude_code(
    claude_config_dir: &Path,
    env_config: &EnvironmentConfig,
    working_dir: &Path,
) -> Result<()> {
    // Find the claude binary
    let claude_path = which::which("claude")
        .map_err(|_| anyhow!("Claude Code not found in PATH. Install with: npm install -g @anthropic-ai/claude-code"))?;

    let mut cmd = Command::new(&claude_path);

    // Set working directory
    cmd.current_dir(working_dir);

    // Inherit stdin/stdout/stderr for direct terminal access
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    // Set CLAUDE_CONFIG_DIR for isolation
    cmd.env("CLAUDE_CONFIG_DIR", claude_config_dir.to_string_lossy().to_string());

    // Pass through important system environment variables
    if let Ok(path) = std::env::var("PATH") {
        cmd.env("PATH", path);
    }
    if let Ok(term) = std::env::var("TERM") {
        cmd.env("TERM", term);
    } else {
        cmd.env("TERM", "xterm-256color");
    }
    if let Ok(home) = std::env::var("HOME") {
        cmd.env("HOME", home);
    }
    if let Ok(user) = std::env::var("USER") {
        cmd.env("USER", user);
    }
    #[cfg(windows)]
    {
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            cmd.env("USERPROFILE", userprofile);
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            cmd.env("APPDATA", appdata);
        }
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            cmd.env("LOCALAPPDATA", localappdata);
        }
        if let Ok(systemroot) = std::env::var("SystemRoot") {
            cmd.env("SystemRoot", systemroot);
        }
    }

    // Apply provider configuration if not using default Anthropic
    if env_config.provider.preset != ProviderPreset::Anthropic {
        // Set base URL if available
        if let Some(base_url) = env_config.provider.effective_base_url() {
            cmd.env("ANTHROPIC_BASE_URL", base_url);
        }

        // Set API key/token if provided
        if let Some(ref api_key) = env_config.provider.api_key {
            cmd.env("ANTHROPIC_AUTH_TOKEN", api_key);
        }

        // Always set timeout and disable non-essential traffic for custom providers
        cmd.env("API_TIMEOUT_MS", "3000000");
        cmd.env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1");

        // Set custom models if available
        if let Some(models) = env_config.provider.effective_models() {
            if !models.opus.is_empty() {
                cmd.env("ANTHROPIC_MODEL", &models.opus);
                cmd.env("ANTHROPIC_DEFAULT_OPUS_MODEL", &models.opus);
            }
            if !models.sonnet.is_empty() {
                cmd.env("ANTHROPIC_DEFAULT_SONNET_MODEL", &models.sonnet);
            }
            if !models.haiku.is_empty() {
                cmd.env("ANTHROPIC_SMALL_FAST_MODEL", &models.haiku);
                cmd.env("ANTHROPIC_DEFAULT_HAIKU_MODEL", &models.haiku);
            }
        }
    }

    // Spawn and wait for the process
    let mut child = cmd.spawn()?;
    child.wait()?;

    Ok(())
}
