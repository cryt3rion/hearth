use anyhow::Result;
use clap::Args as ClapArgs;
use std::collections::HashMap;

use super::Context;
use crate::{model::Tool, output, scan};

#[derive(ClapArgs, Debug, Default, Clone)]
pub struct Args {}

pub async fn run(_args: Args, ctx: &Context) -> Result<()> {
    let mut tools = scan::scan_all().await?;
    crate::fs_util::sizes::populate_sizes(&mut tools, ctx.no_cache || ctx.refresh).await?;

    let mut by_name: HashMap<String, Vec<Tool>> = HashMap::new();
    for t in tools.into_iter() {
        by_name.entry(t.name.clone()).or_default().push(t);
    }
    let mut dupes: Vec<Tool> = by_name
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .flat_map(|(_, v)| v.into_iter())
        .collect();
    dupes.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then(a.source.label().cmp(b.source.label()))
    });

    if ctx.json {
        output::json::print(&dupes)?;
    } else {
        output::table::print_list(&dupes);
    }
    Ok(())
}
