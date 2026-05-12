use anyhow::Result;
use clap::Args as ClapArgs;

use super::Context;
use crate::{output, scan};

#[derive(ClapArgs, Debug, Default, Clone)]
pub struct Args {}

pub async fn run(_args: Args, ctx: &Context) -> Result<()> {
    let mut tools = scan::scan_all().await?;
    crate::fs_util::sizes::populate_sizes(&mut tools, ctx.no_cache || ctx.refresh).await?;
    tools.retain(|t| t.source.is_manual());
    tools.sort_by_key(|t| std::cmp::Reverse(t.size_bytes));

    if ctx.json {
        output::json::print(&tools)?;
    } else {
        output::table::print_list(&tools);
    }
    Ok(())
}
