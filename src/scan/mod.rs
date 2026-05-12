use anyhow::Result;
use async_trait::async_trait;
use futures::future::join_all;

use crate::model::{ClaimedPaths, Tool};

pub mod bun;
pub mod cargo;
pub mod gh_ext;
pub mod go;
pub mod homebrew;
pub mod npm;
pub mod path_scan;
pub mod rustup;

#[async_trait]
pub trait Scanner: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool {
        true
    }
    async fn scan(&self) -> Result<ScanOutput>;
}

#[derive(Debug, Default)]
pub struct ScanOutput {
    pub tools: Vec<Tool>,
    pub claimed: ClaimedPaths,
}

impl ScanOutput {
    pub fn new(tools: Vec<Tool>, claimed: ClaimedPaths) -> Self {
        Self { tools, claimed }
    }
}

/// Run all managed scanners in parallel, then run the PATH/orphan scanner
/// once with the accumulated claims.
pub async fn scan_all() -> Result<Vec<Tool>> {
    let managed: Vec<Box<dyn Scanner>> = vec![
        Box::new(homebrew::HomebrewScanner),
        Box::new(npm::NpmScanner),
        Box::new(bun::BunScanner),
        Box::new(cargo::CargoScanner),
        Box::new(rustup::RustupScanner),
        Box::new(go::GoScanner),
        Box::new(gh_ext::GhExtScanner),
    ];

    let futures = managed.iter().filter(|s| s.is_available()).map(|s| {
        let name = s.name();
        async move {
            let r = s.scan().await;
            (name, r)
        }
    });
    let results = join_all(futures).await;

    let mut all_tools: Vec<Tool> = Vec::new();
    let mut all_claimed = ClaimedPaths::default();
    for (name, res) in results {
        match res {
            Ok(out) => {
                tracing::info!(scanner = name, count = out.tools.len(), "scanner complete");
                merge_claims(&mut all_claimed, out.claimed);
                all_tools.extend(out.tools);
            }
            Err(e) => {
                tracing::warn!(scanner = name, error = %e, "scanner failed");
            }
        }
    }

    // Now run the PATH scan with everything we've claimed.
    let path_scanner = path_scan::PathScanner::new(all_claimed);
    let out = path_scanner.scan().await?;
    all_tools.extend(out.tools);

    Ok(all_tools)
}

fn merge_claims(into: &mut ClaimedPaths, from: ClaimedPaths) {
    let parts = from.into_parts();
    for p in parts.bin_paths {
        into.add_bin(p);
    }
    for r in parts.install_roots {
        into.add_install_root(r);
    }
}

/// Find an executable on the user's PATH. Returns first hit.
pub fn which(cmd: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let p = dir.join(cmd);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
