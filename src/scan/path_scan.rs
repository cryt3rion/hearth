use anyhow::Result;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::{ScanOutput, Scanner};
use crate::fs_util::symlink::{canonicalize_safe, is_executable};
use crate::model::{ClaimedPaths, Source, Tool};

pub struct PathScanner {
    claimed: ClaimedPaths,
}

impl PathScanner {
    pub fn new(claimed: ClaimedPaths) -> Self {
        Self { claimed }
    }
}

const SYSTEM_DIR_PREFIXES: &[&str] = &[
    "/usr/bin",
    "/bin",
    "/sbin",
    "/usr/sbin",
    "/System/",
    "/Library/Apple/",
    "/var/run/com.apple.security.cryptexd/",
    "/var/run/com.apple.security.",
    "/Library/TeX/",
];

#[async_trait]
impl Scanner for PathScanner {
    fn name(&self) -> &'static str {
        "path"
    }

    async fn scan(&self) -> Result<ScanOutput> {
        let path_var = std::env::var("PATH").unwrap_or_default();
        let mut seen_dirs: HashSet<PathBuf> = HashSet::new();
        let mut seen_names: HashMap<String, PathBuf> = HashMap::new();
        // canonical target -> Tool index in `tools`, for alias dedup
        let mut by_canonical: HashMap<PathBuf, usize> = HashMap::new();
        let mut tools: Vec<Tool> = Vec::new();

        for raw_dir in path_var.split(':') {
            if raw_dir.is_empty() {
                continue;
            }
            let dir = PathBuf::from(raw_dir);
            if !seen_dirs.insert(dir.clone()) {
                continue;
            }
            if is_system_dir(&dir) {
                continue;
            }
            let Ok(rd) = std::fs::read_dir(&dir) else {
                continue;
            };

            for entry in rd.flatten() {
                let bin_path = entry.path();
                let Ok(meta) = entry.metadata() else { continue };
                if !meta.is_file() && !meta.file_type().is_symlink() {
                    continue;
                }
                // For symlinks, follow to test executability; for regular files, check bits.
                let exec = if meta.file_type().is_symlink() {
                    std::fs::metadata(&bin_path)
                        .map(|m| is_executable(&m))
                        .unwrap_or(false)
                } else {
                    is_executable(&meta)
                };
                if !exec {
                    continue;
                }

                let Some(name) = bin_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(String::from)
                else {
                    continue;
                };

                // Shadowing: if we've already seen this name earlier in PATH, record it.
                if let Some(earlier) = seen_names.get(&name) {
                    if let Some(idx) = by_canonical.iter().find_map(|(_, i)| {
                        if tools[*i].bin_path == *earlier {
                            Some(*i)
                        } else {
                            None
                        }
                    }) {
                        tools[idx].shadowed_by.push(bin_path.clone());
                    }
                    continue;
                }
                seen_names.insert(name.clone(), bin_path.clone());

                // Already claimed by a managed scanner?
                if self.claimed.claims_bin(&bin_path) {
                    continue;
                }

                let canonical = canonicalize_safe(&bin_path);

                // Ownership via install_root prefix
                if let Some(c) = canonical.as_ref() {
                    if self.claimed.claims_canonical(c) {
                        continue;
                    }
                }

                // Alias dedup by canonical target
                if let Some(c) = canonical.as_ref() {
                    if let Some(&idx) = by_canonical.get(c) {
                        tools[idx].aliases.push(bin_path);
                        continue;
                    }
                }

                // .app bundle?
                let source = match classify(canonical.as_deref(), &bin_path, &dir) {
                    Classification::AppBundle(p) => Source::AppBundle { app_path: p },
                    Classification::Manual => Source::Manual { dir: dir.clone() },
                };

                let mut t = Tool::new(name.clone(), bin_path.clone(), source);
                if let Some(c) = canonical.clone() {
                    t.symlink_target = Some(c.clone());
                    if c != bin_path {
                        // If the canonical lives inside a dedicated tool dir
                        // (e.g., ~/.local/share/<name>/...), size that whole dir.
                        // Otherwise just size the binary file.
                        t.install_path = Some(find_install_root(&c, &name).unwrap_or(c.clone()));
                    }
                    by_canonical.insert(c, tools.len());
                }
                tools.push(t);
            }
        }

        Ok(ScanOutput::new(tools, ClaimedPaths::default()))
    }
}

enum Classification {
    AppBundle(PathBuf),
    Manual,
}

fn classify(canonical: Option<&Path>, _bin: &Path, _dir: &Path) -> Classification {
    if let Some(c) = canonical {
        // walk up until we find a .app component
        for ancestor in c.ancestors() {
            if let Some(name) = ancestor.file_name().and_then(|s| s.to_str()) {
                if name.ends_with(".app") {
                    return Classification::AppBundle(ancestor.to_path_buf());
                }
            }
        }
    }
    Classification::Manual
}

fn is_system_dir(dir: &Path) -> bool {
    let s = dir.to_string_lossy();
    SYSTEM_DIR_PREFIXES
        .iter()
        .any(|prefix| s.starts_with(prefix))
}

/// Walk up from the canonical binary path looking for a directory named after
/// the tool itself (typical layout: ~/.local/share/<tool>/versions/X.Y.Z/<tool>,
/// /opt/<tool>/bin/<tool>, etc.). Returns the matching directory so we can size
/// the whole install rather than just the launcher binary.
fn find_install_root(canonical: &Path, bin_name: &str) -> Option<PathBuf> {
    let lower = bin_name.to_ascii_lowercase();
    for ancestor in canonical.ancestors().skip(1) {
        let Some(file_name) = ancestor.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let fl = file_name.to_ascii_lowercase();
        if fl == lower || fl.contains(&lower) {
            // Reject obvious system roots
            let s = ancestor.to_string_lossy();
            if s == "/"
                || s.starts_with("/usr")
                || s.starts_with("/bin")
                || s.starts_with("/opt/homebrew")
            {
                continue;
            }
            return Some(ancestor.to_path_buf());
        }
    }
    None
}
