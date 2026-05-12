pub mod cache;
pub mod cli;
pub mod fs_util;
pub mod model;
pub mod output;
pub mod scan;

use anyhow::Result;

pub async fn run(args: cli::Args) -> Result<()> {
    cli::run(args).await
}
