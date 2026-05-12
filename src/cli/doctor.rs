use anyhow::Result;
use clap::Args as ClapArgs;

use super::Context;
use crate::output;
use crate::scan;

#[derive(ClapArgs, Debug, Default, Clone)]
pub struct Args {}

#[derive(Debug, serde::Serialize)]
pub struct Finding {
    pub severity: &'static str, // "info" | "warn" | "error"
    pub kind: &'static str,
    pub message: String,
}

pub async fn run(_args: Args, ctx: &Context) -> Result<()> {
    let mut findings = Vec::new();

    // Missing PATH directories
    for dir in std::env::var("PATH").unwrap_or_default().split(':') {
        if dir.is_empty() {
            continue;
        }
        let p = std::path::Path::new(dir);
        if !p.exists() {
            findings.push(Finding {
                severity: "warn",
                kind: "missing_path_entry",
                message: format!("PATH entry does not exist: {}", dir),
            });
        }
    }

    // Run scanners to detect shadowed binaries and broken symlinks
    let tools = scan::scan_all().await?;
    for t in &tools {
        if !t.shadowed_by.is_empty() {
            findings.push(Finding {
                severity: "info",
                kind: "shadowed",
                message: format!(
                    "{} at {} is shadowed by {} earlier PATH entr{}",
                    t.name,
                    t.bin_path.display(),
                    t.shadowed_by.len(),
                    if t.shadowed_by.len() == 1 { "y" } else { "ies" },
                ),
            });
        }
    }

    if ctx.json {
        output::json::print(&findings)?;
    } else {
        output::table::print_doctor(&findings);
    }
    Ok(())
}
