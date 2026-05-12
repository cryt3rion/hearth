use anyhow::Result;
use clap::Args as ClapArgs;

use super::Context;
use crate::{output, scan};

#[derive(ClapArgs, Debug, Default, Clone)]
pub struct Args {}

/// Hardcoded v0.1.0 AI denylist. v0.2 replaces this with a curated YAML registry.
const AI_NAMES: &[&str] = &[
    "claude",
    "claude-code",
    "codex",
    "cursor-agent",
    "gemini",
    "openai",
    "anthropic",
    "ollama",
    "aider",
    "llm",
    "mods",
    "fabric",
    "cody",
    "tgpt",
    "butterfish",
    "exa",
    "perplexity",
];

pub async fn run(_args: Args, ctx: &Context) -> Result<()> {
    let mut tools = scan::scan_all().await?;
    crate::fs_util::sizes::populate_sizes(&mut tools, ctx.no_cache || ctx.refresh).await?;
    tools.retain(|t| is_ai_named(&t.name));
    tools.sort_by(|a, b| a.name.cmp(&b.name));

    if ctx.json {
        output::json::print(&tools)?;
    } else {
        output::table::print_list(&tools);
    }
    Ok(())
}

fn is_ai_named(name: &str) -> bool {
    let normalized: String = name
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c == ' ' || c == '_' { '-' } else { c })
        .collect();
    AI_NAMES.iter().any(|a| {
        normalized == *a
            || normalized.starts_with(&format!("{a}-"))
            || normalized.ends_with(&format!("-{a}"))
            || normalized.contains(&format!("-{a}-"))
    })
}
