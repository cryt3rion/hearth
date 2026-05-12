use clap::Parser;

#[tokio::main]
async fn main() {
    let args = hearth::cli::Args::parse();
    if let Err(err) = hearth::run(args).await {
        eprintln!("hearth: {err:#}");
        std::process::exit(1);
    }
}
