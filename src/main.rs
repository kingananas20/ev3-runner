mod cli;
mod server;

use clap::Parser;
use cli::{Cli, Commands};
use server::server;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();
    let cli = Cli::parse();

    match cli.command {
        Commands::Server(config) => server(config).await?,
        Commands::Client(config) => println!("{config:?}"),
    }

    Ok(())
}

fn setup_logging() {
    let subscriber = FmtSubscriber::new();
    subscriber.init();
}
