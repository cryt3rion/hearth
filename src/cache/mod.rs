use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct SizeCache {
    pub version: u32,
    pub entries: HashMap<PathBuf, SizeEntry>,
}

impl Default for SizeCache {
    fn default() -> Self {
        Self {
            version: CACHE_VERSION,
            entries: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SizeEntry {
    pub bytes: u64,
    /// Unix timestamp of the path's mtime at the time of caching
    pub mtime_secs: u64,
}

impl SizeCache {
    pub fn load_or_default() -> Self {
        match cache_file() {
            Some(path) => match std::fs::read(&path) {
                Ok(bytes) => match serde_json::from_slice::<SizeCache>(&bytes) {
                    Ok(c) if c.version == CACHE_VERSION => c,
                    _ => Self::default(),
                },
                Err(_) => Self::default(),
            },
            None => Self::default(),
        }
    }

    pub fn save(&self) -> Result<()> {
        let Some(path) = cache_file() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(self).context("serialize size cache")?;
        std::fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }

    pub fn lookup(&self, path: &Path, mtime: SystemTime) -> Option<u64> {
        let mtime_secs = mtime.duration_since(UNIX_EPOCH).ok()?.as_secs();
        let entry = self.entries.get(path)?;
        if entry.mtime_secs == mtime_secs {
            Some(entry.bytes)
        } else {
            None
        }
    }

    pub fn insert(&mut self, path: PathBuf, mtime: SystemTime, bytes: u64) {
        let Ok(d) = mtime.duration_since(UNIX_EPOCH) else {
            return;
        };
        self.entries.insert(
            path,
            SizeEntry {
                bytes,
                mtime_secs: d.as_secs(),
            },
        );
    }
}

fn cache_file() -> Option<PathBuf> {
    directories::ProjectDirs::from("sh", "hearth", "hearth")
        .map(|dirs| dirs.cache_dir().join("sizes.json"))
}
