mod server;
mod ssh;

use crate::server::{SharedData, run};
use crate::ssh::SshSession;
use anyhow::Context;
use axum::Router;
use axum::routing::post;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let session = SshSession::connect("robot", "192.168.137.3:22", "maker")
        .context("Couldn't connect to the ssh server")?;

    let shared_data = Arc::new(SharedData {
        sshsession: Mutex::new(session),
    });

    let routes = Router::new()
        .route("/run", post(run))
        .with_state(shared_data);

    let server_port = 6767u16;
    let listener = TcpListener::bind(format!("127.0.0.1:{server_port}")).await?;
    info!("Listening on 127.0.0.1:{server_port}");

    axum::serve(listener, routes).await?;

    Ok(())
}

fn setup_logging() {
    let subscriber = FmtSubscriber::new();
    subscriber.init();
}
