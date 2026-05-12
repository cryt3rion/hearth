use anyhow::Result;
use clap::Args as ClapArgs;

use super::Context;
use crate::output;
use crate::scan;

#[derive(ClapArgs, Debug, Default, Clone)]
pub struct Args {
    /// Filter by source (repeatable): brew, cask, npm, bun, cargo, rustup, go, gh-ext, app, manual
    #[arg(long, value_delimiter = ',')]
    pub source: Vec<String>,

    /// Filter by category (ai, devtool, container, lang, other)
    #[arg(long)]
    pub category: Option<String>,

    /// Sort order: name | size | source
    #[arg(long, default_value = "name")]
    pub sort: String,

    /// Limit results
    #[arg(long)]
    pub limit: Option<usize>,
}

pub async fn run(args: Args, ctx: &Context) -> Result<()> {
    let mut tools = scan::scan_all().await?;
    crate::fs_util::sizes::populate_sizes(&mut tools, ctx.no_cache || ctx.refresh).await?;

    // filter
    if !args.source.is_empty() {
        tools.retain(|t| {
            args.source
                .iter()
                .any(|s| s.eq_ignore_ascii_case(t.source.label()))
        });
    }
    if let Some(ref cat) = args.category {
        let want = cat.to_ascii_lowercase();
        tools.retain(|t| {
            t.category
                .map(|c| format!("{c:?}").to_ascii_lowercase() == want)
                .unwrap_or(false)
        });
    }

    // sort
    match args.sort.as_str() {
        "size" => tools.sort_by_key(|t| std::cmp::Reverse(t.size_bytes)),
        "source" => tools.sort_by(|a, b| {
            a.source
                .label()
                .cmp(b.source.label())
                .then(a.name.cmp(&b.name))
        }),
        _ => tools.sort_by(|a, b| a.name.cmp(&b.name)),
    }

    if let Some(n) = args.limit {
        tools.truncate(n);
    }

    if ctx.json {
        output::json::print(&tools)?;
    } else {
        output::table::print_list(&tools);
    }
    Ok(())
}
