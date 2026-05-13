use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use super::{ScanOutput, Scanner};
use crate::model::{ClaimedPaths, Source, Tool};

pub struct CargoScanner;

#[async_trait]
impl Scanner for CargoScanner {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn is_available(&self) -> bool {
        cargo_home().is_some_and(|h| h.join(".crates2.json").is_file())
    }

    async fn scan(&self) -> Result<ScanOutput> {
        let Some(home) = cargo_home() else {
            return Ok(ScanOutput::default());
        };
        let bin_dir = home.join("bin");
        let manifest = home.join(".crates2.json");

        let mut claimed = ClaimedPaths::default();
        claimed.add_install_root(home.join("registry"));

        let Ok(bytes) = std::fs::read(&manifest) else {
            return Ok(ScanOutput::default());
        };
        let parsed: Crates2 = serde_json::from_slice(&bytes)?;

        let mut tools = Vec::new();
        for (key, entry) in parsed.installs {
            // key is "name vers (registry+url)"
            let crate_name = key.split_whitespace().next().unwrap_or("").to_string();
            let version = key.split_whitespace().nth(1).map(|s| s.to_string());

            for bin in &entry.bins {
                let bin_path = bin_dir.join(bin);
                let mut t = Tool::new(bin.clone(), bin_path.clone(), Source::Cargo);
                t.version = version.clone();
                t.package_id = Some(crate_name.clone());
                claimed.add_bin(bin_path);
                tools.push(t);
            }
        }

        Ok(ScanOutput::new(tools, claimed))
    }
}

#[derive(Deserialize, Debug)]
struct Crates2 {
    installs: HashMap<String, Crates2Entry>,
}

#[derive(Deserialize, Debug)]
struct Crates2Entry {
    #[serde(default)]
    bins: Vec<String>,
}

fn cargo_home() -> Option<PathBuf> {
    if let Ok(c) = std::env::var("CARGO_HOME") {
        return Some(PathBuf::from(c));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cargo"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_crates2_fixture() {
        let bytes = include_bytes!("../../tests/fixtures/crates2.json");
        let parsed: Crates2 = serde_json::from_slice(bytes).expect("crates2.json parses");
        let mut installs: Vec<(String, Crates2Entry)> = parsed.installs.into_iter().collect();
        installs.sort_by(|a, b| a.0.cmp(&b.0));
        insta::assert_debug_snapshot!(installs);
    }
}
