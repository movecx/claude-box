use anyhow::Result;
use std::fs;
use std::path::Path;

/// Create the claude-data directory structure for an environment
pub fn create_claude_data_structure(claude_data_dir: &Path) -> Result<()> {
    // Create the main claude-data directory
    fs::create_dir_all(claude_data_dir)?;

    // Create subdirectories that Claude Code expects
    let subdirs = [
        "plans",
        "projects",
        "file-history",
        "todos",
        "plugins",
        "plugins/cache",
        "plugins/marketplaces",
        "ide",
        "downloads",
        "debug",
        "shell-snapshots",
        "statsig",
    ];

    for subdir in subdirs {
        fs::create_dir_all(claude_data_dir.join(subdir))?;
    }

    Ok(())
}
