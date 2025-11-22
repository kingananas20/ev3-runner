mod cli;
mod client;
mod hash;
mod protocol;
mod server;

use crate::client::client;
use clap::Parser;
use cli::{Cli, Commands};
use server::server;
use tracing::Level;
use tracing_subscriber::fmt::SubscriberBuilder;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    setup_logging(cli.verbose);

    match cli.command {
        Commands::Server(config) => server(config)?,
        Commands::Client(config) => client(config)?,
    }

    Ok(())
}

fn setup_logging(verbosity: u8) {
    let subscriber = SubscriberBuilder::default();
    let subscriber = match verbosity {
        0 => subscriber.with_max_level(Level::WARN),
        1 => subscriber.with_max_level(Level::INFO),
        2 => subscriber.with_max_level(Level::DEBUG),
        3.. => subscriber.with_max_level(Level::TRACE),
    };
    subscriber.init();
}
