use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{ScanOutput, Scanner};
use crate::model::{ClaimedPaths, Source, Tool};

/// One tool per installed toolchain. Individual `rustc`/`cargo`/etc. in PATH
/// resolve into the toolchain dir (claimed root) so the orphan scanner ignores them.
/// Also claims `~/.cargo/bin` since those are rustup-managed shims.
pub struct RustupScanner;

#[async_trait]
impl Scanner for RustupScanner {
    fn name(&self) -> &'static str {
        "rustup"
    }

    fn is_available(&self) -> bool {
        rustup_home().is_some_and(|h| h.join("toolchains").is_dir())
    }

    async fn scan(&self) -> Result<ScanOutput> {
        let Some(home) = rustup_home() else {
            return Ok(ScanOutput::default());
        };
        let toolchains = home.join("toolchains");
        let mut claimed = ClaimedPaths::default();
        claimed.add_install_root(&toolchains);
        if let Some(cargo_home) = cargo_home() {
            claimed.add_install_root(cargo_home.join("bin"));
        }

        let mut tools = Vec::new();
        let Ok(rd) = std::fs::read_dir(&toolchains) else {
            return Ok(ScanOutput::new(tools, claimed));
        };
        for entry in rd.flatten() {
            let toolchain_path = entry.path();
            if !toolchain_path.is_dir() {
                continue;
            }
            let Some(toolchain) = toolchain_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(String::from)
            else {
                continue;
            };
            // Display path: prefer the rustup shim in ~/.cargo/bin so users see
            // something they can actually invoke.
            let bin_path = cargo_home()
                .map(|c| c.join("bin").join("rustup"))
                .unwrap_or_else(|| toolchain_path.clone());
            let mut t = Tool::new(
                format!("rustup:{toolchain}"),
                bin_path,
                Source::Rustup {
                    toolchain: toolchain.clone(),
                },
            );
            t.install_path = Some(toolchain_path.clone());
            t.version = Some(toolchain);
            tools.push(t);
        }
        Ok(ScanOutput::new(tools, claimed))
    }
}

fn rustup_home() -> Option<PathBuf> {
    if let Ok(r) = std::env::var("RUSTUP_HOME") {
        return Some(PathBuf::from(r));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".rustup"))
}

fn cargo_home() -> Option<PathBuf> {
    if let Ok(c) = std::env::var("CARGO_HOME") {
        return Some(PathBuf::from(c));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cargo"))
}
