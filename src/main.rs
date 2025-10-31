mod server;
mod ssh;

use crate::server::{SharedData, run};
use crate::ssh::SshSession;
use anyhow::Context;
use axum::Router;
use axum::routing::post;
use clap::Parser;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, clap::Parser)]
struct Cli {
    /// Address and port (addr:port) of the robot
    host: String,
    /// Username of the robot (ev3dev default `robot`)
    #[clap(short, long, default_value = "robot")]
    username: String,
    /// Password to connect (ev3dev default `maker`)
    #[clap(short, long, default_value = "maker")]
    password: String,
    /// Port of the local server
    #[clap(short, long, default_value = "6767")]
    server_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();
    let config = Cli::parse();

    let session = SshSession::connect(&config.username, &config.host, &config.password)
        .context("Couldn't connect to the ssh server")?;

    let shared_data = Arc::new(SharedData {
        sshsession: Mutex::new(session),
    });

    let routes = Router::new()
        .route("/run", post(run))
        .with_state(shared_data);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", &config.server_port)).await?;
    info!("Listening on 127.0.0.1:{}", &config.server_port);

    axum::serve(listener, routes).await?;

    Ok(())
}

fn setup_logging() {
    let subscriber = FmtSubscriber::new();
    subscriber.init();
}
