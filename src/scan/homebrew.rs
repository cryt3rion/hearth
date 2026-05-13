use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::process::Command;

use super::{ScanOutput, Scanner};
use crate::model::{BrewKind, ClaimedPaths, Source, Tool};

pub struct HomebrewScanner;

#[async_trait]
impl Scanner for HomebrewScanner {
    fn name(&self) -> &'static str {
        "homebrew"
    }

    fn is_available(&self) -> bool {
        crate::scan::which("brew").is_some()
    }

    async fn scan(&self) -> Result<ScanOutput> {
        let prefix = brew_prefix().await?;
        let cellar = prefix.join("Cellar");
        let caskroom = prefix.join("Caskroom");
        let bin_dir = prefix.join("bin");

        let info = run_brew_info().await?;
        let mut tools = Vec::new();
        let mut claimed = ClaimedPaths::default();
        claimed.add_install_root(&cellar);
        claimed.add_install_root(&caskroom);

        // Formulae
        for f in &info.formulae {
            let Some(installed) = f.installed.first() else {
                continue;
            };
            let install_path = cellar.join(&f.name).join(&installed.version);
            let mut t = Tool::new(
                f.name.clone(),
                bin_dir.join(&f.name),
                Source::Homebrew {
                    tap: f.tap.clone().unwrap_or_default(),
                    kind: BrewKind::Formula,
                },
            );
            t.version = Some(installed.version.clone());
            t.install_path = Some(install_path.clone());
            t.homepage = f.homepage.clone();
            t.package_id = Some(format!(
                "{}/{}",
                f.tap.clone().unwrap_or_else(|| "homebrew/core".into()),
                f.name
            ));

            // Walk the keg's bin/ to find every binary it ships
            let keg_bin = install_path.join("bin");
            if keg_bin.is_dir() {
                if let Ok(rd) = std::fs::read_dir(&keg_bin) {
                    for entry in rd.flatten() {
                        let bin = entry.path();
                        claimed.add_bin(&bin);
                        // Also claim the symlink that brew puts in prefix/bin/<name>
                        if let Some(name) = bin.file_name() {
                            claimed.add_bin(bin_dir.join(name));
                        }
                    }
                }
            }
            tools.push(t);
        }

        // Casks
        for c in &info.casks {
            let Some(installed_ver) = c.installed.as_deref() else {
                continue;
            };
            let cask_path = caskroom.join(&c.token).join(installed_ver);
            let mut t = Tool::new(
                c.name.first().cloned().unwrap_or_else(|| c.token.clone()),
                cask_path.clone(), // best effort; orphan scanner will refine if needed
                Source::Homebrew {
                    tap: c.tap.clone().unwrap_or_default(),
                    kind: BrewKind::Cask,
                },
            );
            t.version = Some(installed_ver.to_string());
            t.install_path = Some(cask_path.clone());
            t.homepage = c.homepage.clone();
            t.package_id = Some(format!(
                "{}/{}",
                c.tap.clone().unwrap_or_else(|| "homebrew/cask".into()),
                c.token
            ));

            // Casks often install binaries to prefix/bin (e.g., claude, codex).
            // We can't enumerate them generically without inspecting artifacts;
            // mark the install path as claimed, and let the orphan scanner attribute
            // any prefix/bin entries that canonicalize into this Caskroom path.
            tools.push(t);
        }

        Ok(ScanOutput::new(tools, claimed))
    }
}

#[derive(Deserialize, Debug)]
struct BrewInfo {
    #[serde(default)]
    formulae: Vec<Formula>,
    #[serde(default)]
    casks: Vec<Cask>,
}

#[derive(Deserialize, Debug)]
struct Formula {
    name: String,
    tap: Option<String>,
    homepage: Option<String>,
    #[serde(default)]
    installed: Vec<FormulaInstall>,
}

#[derive(Deserialize, Debug)]
struct FormulaInstall {
    version: String,
}

#[derive(Deserialize, Debug)]
struct Cask {
    token: String,
    #[serde(default)]
    name: Vec<String>,
    tap: Option<String>,
    homepage: Option<String>,
    installed: Option<String>,
}

async fn brew_prefix() -> Result<PathBuf> {
    let out = Command::new("brew")
        .arg("--prefix")
        .output()
        .await
        .context("run `brew --prefix`")?;
    if !out.status.success() {
        anyhow::bail!(
            "brew --prefix failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

async fn run_brew_info() -> Result<BrewInfo> {
    let out = Command::new("brew")
        .args(["info", "--json=v2", "--installed"])
        .output()
        .await
        .context("run `brew info --json=v2 --installed`")?;
    if !out.status.success() {
        anyhow::bail!("brew info failed: {}", String::from_utf8_lossy(&out.stderr));
    }
    serde_json::from_slice(&out.stdout).context("parse `brew info --json=v2` output")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_brew_info_v2_fixture() {
        let json = include_bytes!("../../tests/fixtures/brew_info.json");
        let parsed: BrewInfo = serde_json::from_slice(json).expect("brew_info.json parses");
        insta::assert_debug_snapshot!(parsed);
    }
}
