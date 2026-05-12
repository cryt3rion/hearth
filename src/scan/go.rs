use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{ScanOutput, Scanner};
use crate::fs_util::symlink::is_executable;
use crate::model::{ClaimedPaths, Source, Tool};

pub struct GoScanner;

#[async_trait]
impl Scanner for GoScanner {
    fn name(&self) -> &'static str {
        "go"
    }

    fn is_available(&self) -> bool {
        go_bin().is_some_and(|d| d.is_dir())
    }

    async fn scan(&self) -> Result<ScanOutput> {
        let Some(bin_dir) = go_bin() else {
            return Ok(ScanOutput::default());
        };
        let mut claimed = ClaimedPaths::default();
        claimed.add_install_root(&bin_dir);

        let mut tools = Vec::new();
        let Ok(rd) = std::fs::read_dir(&bin_dir) else {
            return Ok(ScanOutput::new(tools, claimed));
        };
        for entry in rd.flatten() {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else { continue };
            if !meta.is_file() || !is_executable(&meta) {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()).map(String::from) else {
                continue;
            };
            let mut t = Tool::new(name, path.clone(), Source::Go);
            t.install_path = Some(path.clone());
            claimed.add_bin(path);
            tools.push(t);
        }
        Ok(ScanOutput::new(tools, claimed))
    }
}

fn go_bin() -> Option<PathBuf> {
    if let Ok(b) = std::env::var("GOBIN") {
        let p = PathBuf::from(b);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Ok(p) = std::env::var("GOPATH") {
        let p = PathBuf::from(p).join("bin");
        if p.is_dir() {
            return Some(p);
        }
    }
    let home = std::env::var_os("HOME")?;
    let p = PathBuf::from(home).join("go").join("bin");
    p.is_dir().then_some(p)
}
