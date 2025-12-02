pub use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(
    name = "ev3-runner",
    version,
    about = "Upload and run binaries on LEGO EV3 robots",
    long_about = "A tool to upload and execute programs on LEGO EV3 robots running ev3dev.\n\
                  Uses hash-based deduplication to skip uploads when files haven't changed.\n\
                  Supports password protection and real-time output streaming."
)]
pub struct Cli {
    #[clap(
        short,
        long,
        global = true,
        action = clap::ArgAction::Count,
        help = "Increase logging verbosity (-v: INFO, -vv: DEBUG, -vvv: TRACE)"
    )]
    pub verbose: u8,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    /// Run as server (typically on the EV3)
    #[command(
        long_about = "Start the server to accept file uploads and execute programs.\n\
                            This mode should be run on the EV3 robot."
    )]
    Server(Server),
    /// Run as client (on your development machine)
    #[command(
        long_about = "Connect to a server to upload files or execute programs.\n\
                            This mode should be run on your development machine."
    )]
    Client(Client),
}

#[derive(Debug, clap::Args)]
pub struct Client {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Debug, clap::Subcommand)]
pub enum Action {
    /// Upload a file to the server
    #[command(long_about = "Upload a file to the server without executing it.\n\
                            The file will be made executable on the server.")]
    Upload(ClientArgs),
    /// Upload and run a file on the server
    #[command(
        long_about = "Upload a file to the server and execute it immediately.\n\
                            Output from the program will be streamed back in real-time.\n\
                            If the file hash matches what's already on the server, upload is skipped."
    )]
    Run(ClientArgs),
}

#[derive(Debug, clap::Args)]
pub struct ClientArgs {
    /// Path to the local file to upload
    #[arg(value_name = "FILE")]
    pub filepath: PathBuf,

    /// Server address and port
    #[clap(
        long,
        default_value = "127.0.0.1:6767",
        value_name = "HOST:PORT",
        help = "Server address in format IP:PORT"
    )]
    pub host: String,

    /// Where to save the file on the server
    #[clap(
        short,
        long,
        value_name = "PATH",
        help = "Remote file path (default: same filename as local)"
    )]
    pub remote_path: Option<PathBuf>,

    /// Password for authentication
    #[clap(
        short,
        long,
        default_value = "maker",
        value_name = "PASSWORD",
        help = "Password to authenticate with the server"
    )]
    pub password: String,

    /// Use brickrun
    #[clap(short, long, help = "If the program should be started using brickrun")]
    pub brickrun: bool,

    /// If compression should be used to send the file
    #[clap(short, long, help = "If compression should be used to send the file")]
    pub compression: bool,
}

#[derive(Debug, clap::Args)]
pub struct Server {
    /// Port to listen on
    #[clap(
        short,
        long,
        default_value = "6767",
        value_name = "PORT",
        help = "TCP port for incoming connections"
    )]
    pub server_port: u16,

    /// Server password
    #[clap(
        short,
        long,
        default_value = "maker",
        value_name = "PASSWORD",
        help = "Password required for client authentication"
    )]
    pub password: String,
}
