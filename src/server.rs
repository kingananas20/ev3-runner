use crate::ssh::{SessionError, SshSession};
use axum::{
    Json,
    extract::State,
    response::{Sse, sse::Event},
};
use futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use serde::Deserialize;
use std::{
    io::{self, BufRead},
    sync::Arc,
    thread,
};
use tokio::{
    sync::{Mutex, mpsc::unbounded_channel},
    task::JoinError,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::{error, warn};

#[derive(Debug, Deserialize)]
pub struct Run {
    src_path: String,
    dst_path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
    #[error("ssh error: {0}")]
    SshSession(#[from] SessionError),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("blocking task joinerror: {0}")]
    Join(#[from] JoinError),
}

impl axum::response::IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let status = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
        let body = format!("{self}");
        (status, body).into_response()
    }
}

pub struct SharedData {
    pub sshsession: Mutex<SshSession>,
}

#[axum::debug_handler]
pub async fn run(
    State(state): State<Arc<SharedData>>,
    Json(payload): Json<Run>,
) -> Sse<BoxStream<'static, Result<Event, ServerError>>> {
    let mut session = state.sshsession.lock().await;
    let mut reader = match session.upload_and_exec(&payload.src_path, &payload.dst_path) {
        Ok(r) => r,
        Err(e) => {
            warn!("{e}");
            return Sse::new(stream::once(async { Err::<Event, ServerError>(e.into()) }).boxed());
        }
    };

    let (tx, rx) = unbounded_channel::<Result<String, ServerError>>();

    thread::spawn(move || {
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let text = line.trim_end().to_owned();
                    if tx.send(Ok(text)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    warn!("{e}");
                    tx.send(Err(e.into())).ok();
                    break;
                }
            }
        }

        let mut channel = reader.into_inner();
        channel.wait_eof().ok();
        channel.wait_close().ok();
    });

    let stream = UnboundedReceiverStream::new(rx)
        .map(|res| match res {
            Ok(s) => Ok(Event::default().data(s)),
            Err(e) => {
                error!("{e}");
                Err(e)
            }
        })
        .boxed();

    Sse::new(stream)
}
