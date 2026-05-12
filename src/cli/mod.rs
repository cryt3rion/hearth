use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod ai;
pub mod doctor;
pub mod duplicates;
pub mod list;
pub mod orphans;
pub mod size;

#[derive(Parser, Debug)]
#[command(
    name = "hearth",
    version,
    about = "Unified inventory of CLI tools installed on your Mac",
    long_about = None,
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Output as JSON instead of a table
    #[arg(long, global = true)]
    pub json: bool,

    /// Ignore cached sizes / scanner results
    #[arg(long, global = true)]
    pub no_cache: bool,

    /// Force-refresh all caches
    #[arg(long, global = true)]
    pub refresh: bool,

    /// Verbose logging (-v info, -vv debug, -vvv trace)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List every installed CLI tool (default when no command given)
    List(list::Args),
    /// Show binaries in PATH not claimed by any package manager
    Orphans(orphans::Args),
    /// Same-name tools from multiple sources or multiple versions of one tool
    Duplicates(duplicates::Args),
    /// Sorted by disk usage; defaults to top 20
    Size(size::Args),
    /// Diagnostics: broken symlinks, stale kegs, shadowed binaries, etc.
    Doctor(doctor::Args),
    /// Convenience filter for AI CLI tools
    Ai(ai::Args),
}

pub async fn run(args: Args) -> Result<()> {
    init_tracing(args.verbose);

    let cmd = args.command.unwrap_or(Command::List(list::Args::default()));
    let ctx = Context {
        json: args.json,
        no_cache: args.no_cache,
        refresh: args.refresh,
    };

    match cmd {
        Command::List(a) => list::run(a, &ctx).await,
        Command::Orphans(a) => orphans::run(a, &ctx).await,
        Command::Duplicates(a) => duplicates::run(a, &ctx).await,
        Command::Size(a) => size::run(a, &ctx).await,
        Command::Doctor(a) => doctor::run(a, &ctx).await,
        Command::Ai(a) => ai::run(a, &ctx).await,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Context {
    pub json: bool,
    pub no_cache: bool,
    pub refresh: bool,
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| format!("hearth={level}"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init()
        .ok();
}
