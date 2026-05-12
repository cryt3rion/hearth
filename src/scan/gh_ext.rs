use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;

use super::{ScanOutput, Scanner};
use crate::model::{ClaimedPaths, Source, Tool};

pub struct GhExtScanner;

#[async_trait]
impl Scanner for GhExtScanner {
    fn name(&self) -> &'static str {
        "gh_ext"
    }

    fn is_available(&self) -> bool {
        gh_ext_dir().is_some_and(|d| d.is_dir())
    }

    async fn scan(&self) -> Result<ScanOutput> {
        let Some(ext_dir) = gh_ext_dir() else {
            return Ok(ScanOutput::default());
        };
        let mut claimed = ClaimedPaths::default();
        claimed.add_install_root(&ext_dir);

        let mut tools = Vec::new();
        let Ok(rd) = std::fs::read_dir(&ext_dir) else {
            return Ok(ScanOutput::new(tools, claimed));
        };
        for entry in rd.flatten() {
            let path = entry.path();
            // gh extensions are dirs named "gh-foo"
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()).map(String::from) else {
                continue;
            };
            let bin = path.join(&name);
            let mut t = Tool::new(name.clone(), bin.clone(), Source::Gh);
            t.install_path = Some(path.clone());
            t.package_id = Some(name);
            claimed.add_bin(bin);
            tools.push(t);
        }
        Ok(ScanOutput::new(tools, claimed))
    }
}

fn gh_ext_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join(".local/share/gh/extensions"))
        .filter(|p| p.is_dir())
}
