use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{info, warn};
use walkdir::WalkDir;

pub fn find_changed_files(root: &Path, since: SystemTime) -> Result<Vec<PathBuf>> {
    let mut changed = Vec::new();

    for entry in WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Skipping unreadable entry: {}", e);
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let mtime = match entry.metadata().ok().and_then(|m| m.modified().ok()) {
            Some(t) => t,
            None => {
                warn!("Cannot read mtime for {}", entry.path().display());
                continue;
            }
        };

        if mtime > since {
            info!(
                "Changed: {} (mtime: {:?})",
                entry.path().display(),
                mtime
            );
            changed.push(entry.path().to_path_buf());
        }
    }

    Ok(changed)
}
