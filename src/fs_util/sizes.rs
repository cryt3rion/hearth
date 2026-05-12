use anyhow::Result;
use jwalk::WalkDir;
use std::path::Path;

use crate::cache::SizeCache;
use crate::model::Tool;

/// Populate `size_bytes` on each tool. Walks `install_path` when present,
/// otherwise stats `bin_path`. Cached by (path, mtime).
pub async fn populate_sizes(tools: &mut [Tool], skip_cache: bool) -> Result<()> {
    let mut cache = if skip_cache {
        SizeCache::default()
    } else {
        SizeCache::load_or_default()
    };

    for tool in tools.iter_mut() {
        let target = tool
            .install_path
            .as_deref()
            .unwrap_or(tool.bin_path.as_path());

        let Ok(meta) = std::fs::metadata(target) else {
            continue;
        };
        let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);

        if !skip_cache {
            if let Some(bytes) = cache.lookup(target, mtime) {
                tool.size_bytes = bytes;
                continue;
            }
        }

        let bytes = if meta.is_dir() {
            dir_size(target)
        } else {
            meta.len()
        };
        tool.size_bytes = bytes;
        cache.insert(target.to_path_buf(), mtime, bytes);
    }

    if !skip_cache {
        let _ = cache.save();
    }
    Ok(())
}

fn dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    for entry in WalkDir::new(path).skip_hidden(false).follow_links(false) {
        let Ok(entry) = entry else { continue };
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_file() {
            total += meta.len();
        }
    }
    total
}
