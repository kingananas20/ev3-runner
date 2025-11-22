use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[clap(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    Server(Server),
    Client(Client),
}

#[derive(Debug, clap::Args)]
pub struct Client {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, clap::Subcommand)]
pub enum Action {
    /// Upload the file
    Upload(ClientArgs),
    /// Upload and run the file
    Run(ClientArgs),
}

#[derive(Debug, clap::Args)]
pub struct ClientArgs {
    /// Path of the file to upload and execute
    pub filepath: PathBuf,
    /// Websocket of the server
    pub host: String,
}

#[derive(Debug, clap::Args)]
pub struct Server {
    /// Port of the local server
    #[clap(short, long, default_value = "6767")]
    pub server_port: u16,
}
