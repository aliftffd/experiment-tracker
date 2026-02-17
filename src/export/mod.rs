pub mod csv;
pub mod latex;
pub mod markdown;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Write export content to a file in the exports directory
/// Returns the full path of the written file
pub fn write_export(
    base_dir: &Path,
    run_name: &str,
    extension: &str,
    content: &str,
) -> Result<PathBuf> {
    // Create exports directory
    let export_dir = base_dir.join("exports");
    std::fs::create_dir_all(&export_dir)
        .with_context(|| format!("Failed to create export dir: {}", export_dir.display()))?;

    // Generate filename with timestamp to avoid overwriting
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let safe_name = sanitize_filename(run_name);
    let filename = format!("{}_{}.{}", safe_name, timestamp, extension);
    let filepath = export_dir.join(&filename);

    std::fs::write(&filepath, content)
        .with_context(|| format!("Failed to write export: {}", filepath.display()))?;

    Ok(filepath)
}

/// Write a compare export
pub fn write_compare_export(base_dir: &Path, extension: &str, content: &str) -> Result<PathBuf> {
    write_export(base_dir, "comparison", extension, content)
}

/// Sanitize a string for use as a filename
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Get the base directory for exports (first watch dir, or current directory)
pub fn export_base_dir(watch_dirs: &[PathBuf]) -> PathBuf {
    watch_dirs
        .first()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("."))
}
