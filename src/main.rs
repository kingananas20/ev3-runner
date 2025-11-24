use clap::Parser;
use ev3_runner::{Cli, Commands, client, server, setup_logging};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    setup_logging(cli.verbose);

    match cli.command {
        Commands::Server(config) => server(config)?,
        Commands::Client(config) => client(config)?,
    }

    Ok(())
}
