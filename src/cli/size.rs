use anyhow::Result;
use clap::Args as ClapArgs;

use super::Context;
use crate::{output, scan};

#[derive(ClapArgs, Debug, Clone)]
pub struct Args {
    /// Show only the top N tools by size
    #[arg(long, default_value_t = 20)]
    pub top: usize,
}

impl Default for Args {
    fn default() -> Self {
        Self { top: 20 }
    }
}

pub async fn run(args: Args, ctx: &Context) -> Result<()> {
    let mut tools = scan::scan_all().await?;
    crate::fs_util::sizes::populate_sizes(&mut tools, ctx.no_cache || ctx.refresh).await?;
    tools.sort_by_key(|t| std::cmp::Reverse(t.size_bytes));
    let total: u64 = tools.iter().map(|t| t.size_bytes).sum();
    tools.truncate(args.top);

    if ctx.json {
        output::json::print(&tools)?;
    } else {
        output::table::print_size(&tools, total);
    }
    Ok(())
}
