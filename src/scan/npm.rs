use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;

use super::{ScanOutput, Scanner};
use crate::model::{ClaimedPaths, Source, Tool};

pub struct NpmScanner;

#[async_trait]
impl Scanner for NpmScanner {
    fn name(&self) -> &'static str {
        "npm"
    }

    fn is_available(&self) -> bool {
        crate::scan::which("npm").is_some()
    }

    async fn scan(&self) -> Result<ScanOutput> {
        let prefix = npm_prefix().await?;
        let lib_node_modules = prefix.join("lib").join("node_modules");
        let bin_dir = prefix.join("bin");

        let mut claimed = ClaimedPaths::default();
        claimed.add_install_root(&lib_node_modules);

        let raw = match run_npm_ls().await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "npm ls failed");
                return Ok(ScanOutput::new(Vec::new(), claimed));
            }
        };

        let parsed: NpmList = serde_json::from_str(&raw).context("parse `npm ls -g` JSON")?;
        let mut tools = Vec::new();

        for (pkg_name, dep) in parsed.dependencies.unwrap_or_default() {
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

            // Best-effort: assume the bin is `bin_dir/<short_name>` unless the package's
            // `bin` field says otherwise. We can't read package.json for every install
            // cheaply; for v0.1, use the conventional path.
            let bin_path = bin_dir.join(&short_name);
            let install_path = lib_node_modules.join(&pkg_name);

            let mut t = Tool::new(
                short_name.clone(),
                bin_path.clone(),
                Source::Npm {
                    scope: scope.clone(),
                },
            );
            t.version = dep.version;
            t.install_path = Some(install_path.clone());
            t.package_id = Some(pkg_name.clone());

            claimed.add_bin(bin_path);
            tools.push(t);
        }

        Ok(ScanOutput::new(tools, claimed))
    }
}

#[derive(Deserialize, Debug)]
struct NpmList {
    #[serde(default)]
    dependencies: Option<HashMap<String, NpmDep>>,
}

#[derive(Deserialize, Debug)]
struct NpmDep {
    version: Option<String>,
}

async fn npm_prefix() -> Result<PathBuf> {
    let out = Command::new("npm")
        .args(["prefix", "-g"])
        .output()
        .await
        .context("run `npm prefix -g`")?;
    if !out.status.success() {
        anyhow::bail!("npm prefix -g failed");
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

async fn run_npm_ls() -> Result<String> {
    let out = Command::new("npm")
        .args(["ls", "-g", "--depth=0", "--json"])
        .output()
        .await
        .context("run `npm ls -g`")?;
    // npm ls returns nonzero exit code on missing peer deps but still emits valid JSON
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
