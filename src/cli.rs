pub use clap::Parser;
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
    /// Path of the file on the server
    #[clap(short, long)]
    pub remote_path: Option<PathBuf>,
    /// Ip and port of the server in addr:port format
    #[clap(default_value = "127.0.0.1:6767")]
    pub host: String,
    /// Password for secure connection
    #[clap(short, long, default_value = "maker")]
    pub password: String,
}

#[derive(Debug, clap::Args)]
pub struct Server {
    /// Port of the local server
    #[clap(short, long, default_value = "6767")]
    pub server_port: u16,
    /// Password of the server
    #[clap(short, long, default_value = "maker")]
    pub password: String,
}
