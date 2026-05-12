use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use super::{ScanOutput, Scanner};
use crate::model::{ClaimedPaths, Source, Tool};

pub struct BunScanner;

#[async_trait]
impl Scanner for BunScanner {
    fn name(&self) -> &'static str {
        "bun"
    }

    fn is_available(&self) -> bool {
        bun_global_root().is_some()
    }

    async fn scan(&self) -> Result<ScanOutput> {
        let Some(root) = bun_global_root() else {
            return Ok(ScanOutput::default());
        };
        let install_dir = root.join("install").join("global");
        let bin_dir = root.join("bin");
        let pkg_json = install_dir.join("package.json");

        let mut claimed = ClaimedPaths::default();
        claimed.add_install_root(install_dir.join("node_modules"));

        let mut tools = Vec::new();

        let Ok(bytes) = std::fs::read(&pkg_json) else {
            return Ok(ScanOutput::new(tools, claimed));
        };
        let Ok(parsed) = serde_json::from_slice::<BunPkgJson>(&bytes) else {
            return Ok(ScanOutput::new(tools, claimed));
        };

        for (pkg_name, version) in parsed.dependencies.unwrap_or_default() {
            let scope = if pkg_name.starts_with('@') {
                pkg_name.split('/').next().map(|s| s.to_string())
            } else {
                None
            };
            let short_name = pkg_name
                .split('/')
                .next_back()
                .unwrap_or(&pkg_name)
                .to_string();
            let bin_path = bin_dir.join(&short_name);
            let install_path = install_dir.join("node_modules").join(&pkg_name);
            let mut t = Tool::new(short_name.clone(), bin_path.clone(), Source::Bun);
            t.version = Some(strip_version_specifier(&version));
            t.install_path = Some(install_path);
            t.package_id = Some(pkg_name.clone());
            t.aliases = Vec::new();
            t.shadowed_by = Vec::new();
            let _ = scope; // metadata is on package_id
            claimed.add_bin(bin_path);
            tools.push(t);
        }
        Ok(ScanOutput::new(tools, claimed))
    }
}

#[derive(Deserialize, Debug)]
struct BunPkgJson {
    #[serde(default)]
    dependencies: Option<HashMap<String, String>>,
}

fn bun_global_root() -> Option<PathBuf> {
    if let Ok(install) = std::env::var("BUN_INSTALL") {
        let p = PathBuf::from(install);
        if p.is_dir() {
            return Some(p);
        }
    }
    let home = dirs_home()?;
    let candidate = home.join(".bun");
    candidate.is_dir().then_some(candidate)
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn strip_version_specifier(s: &str) -> String {
    // npm versions can be "^1.2.3", "~1.2.3", "1.2.3", "latest", etc.
    s.trim_start_matches(|c: char| !c.is_ascii_digit())
        .to_string()
}
