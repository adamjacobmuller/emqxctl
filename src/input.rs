use crate::error::AppError;
use anyhow::{Context, Result};

/// Read input from a file path or stdin (if path is "-").
/// Tries JSON first, then YAML.
pub fn read_input_file(path: &str) -> Result<serde_json::Value> {
    let contents = if path == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read from stdin")?;
        buf
    } else {
        if !std::path::Path::new(path).exists() {
            return Err(AppError::FileNotFound(path.to_string()).into());
        }
        std::fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path))?
    };

    // Try JSON first
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&contents) {
        return Ok(value);
    }

    // Try YAML
    if let Ok(value) = serde_yaml::from_str::<serde_json::Value>(&contents) {
        return Ok(value);
    }

    Err(AppError::InvalidInput(format!(
        "Could not parse '{}' as JSON or YAML",
        if path == "-" { "stdin" } else { path }
    ))
    .into())
}
