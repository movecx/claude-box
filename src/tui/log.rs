use crate::config::Paths;
use chrono::Utc;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;

pub fn log_tui_error(context: &str, err: &dyn std::error::Error) {
    let paths = match Paths::new() {
        Ok(paths) => paths,
        Err(_) => return,
    };

    let config_dir = paths.config_dir();
    if create_dir_all(&config_dir).is_err() {
        return;
    }

    let log_path = config_dir.join("tui.log");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(
            file,
            "[{}] {}: {}",
            Utc::now().to_rfc3339(),
            context,
            err
        );
    }
}
