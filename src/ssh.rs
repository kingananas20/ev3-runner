use ssh2::{Channel, DisconnectCode, KeyboardInteractivePrompt, Session};
use std::convert::Infallible;
use std::fs::File;
use std::io::BufReader;
use std::io::{self};
use std::net::TcpStream as StdTcpStream;
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;
use tracing::{info, warn};

struct AuthMethods {
    password: bool,
    keyboard_interactive: bool,
    public_key: bool,
}

impl FromStr for AuthMethods {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Self {
            password: false,
            keyboard_interactive: false,
            public_key: false,
        };
        let parts = s.split(",");
        for part in parts {
            match part.to_lowercase().trim() {
                "password" => result.password = true,
                "keyboard-interactive" => result.keyboard_interactive = true,
                "publickey" => result.public_key = true,
                _ => {}
            }
        }

        Ok(result)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Io error: {0}")]
    Io(#[from] io::Error),
    #[error("ssh2 error: {0}")]
    Session(#[from] ssh2::Error),
    #[error("Authorization using public keys is not supported")]
    PubKeyUnsupported,
}

pub struct SshSession {
    session: Session,
}

impl SshSession {
    #[tracing::instrument(skip(password))]
    pub fn connect(username: &str, host: &str, password: &str) -> Result<Self, SessionError> {
        let tcp_stream = StdTcpStream::connect(host)?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp_stream);
        session.set_compress(true);
        session.handshake()?;

        let auth_methods = session
            .auth_methods(username)?
            .parse::<AuthMethods>()
            .unwrap();

        if auth_methods.password {
            session.userauth_password(username, password)?;
        } else if auth_methods.keyboard_interactive {
            let mut prompter = Prompter {
                password: password.to_owned(),
            };
            session.userauth_keyboard_interactive(username, &mut prompter)?;
        } else {
            return Err(SessionError::PubKeyUnsupported);
        }

        info!("Connected to {username}@{host}");

        Ok(Self { session })
    }

    #[tracing::instrument(skip(self))]
    pub fn upload_and_exec(
        &mut self,
        src_path: &Path,
        dst_path: &Path,
    ) -> Result<BufReader<Channel>, SessionError> {
        info!("uploading and executing...");
        self.upload(src_path, dst_path)?;

        let reader = self.exec(dst_path)?;

        Ok(reader)
    }

    #[tracing::instrument(skip(self))]
    fn upload(&mut self, src_path: &Path, dst_path: &Path) -> Result<(), SessionError> {
        let instant = Instant::now();

        let mut local_file = File::open(src_path)?;
        let meta = local_file.metadata()?;

        let mut remote_file = self.session.scp_send(dst_path, 0o755, meta.len(), None)?;

        std::io::copy(&mut local_file, &mut remote_file)?;

        info!(
            "{} bytes file uploaded in {:?}",
            meta.len(),
            instant.elapsed()
        );

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn exec(&mut self, filepath: &Path) -> Result<BufReader<Channel>, SessionError> {
        info!("Executing {}", filepath.display());
        let mut channel = self.get_channel()?;

        channel.exec(&format!("./{}", filepath.display()))?;
        let reader = BufReader::new(channel);

        Ok(reader)
    }

    fn get_channel(&mut self) -> Result<Channel, SessionError> {
        Ok(self.session.channel_session()?)
    }

    #[tracing::instrument(skip(self))]
    pub fn disconnect(&self) -> Result<(), SessionError> {
        self.session
            .disconnect(Some(DisconnectCode::ByApplication), "client closed", None)?;
        info!("Session disconnectd");
        Ok(())
    }
}

impl Drop for SshSession {
    fn drop(&mut self) {
        self.disconnect().ok();
    }
}

struct Prompter {
    password: String,
}

impl KeyboardInteractivePrompt for Prompter {
    fn prompt<'a>(
        &mut self,
        _username: &str,
        _instructions: &str,
        _prompts: &[ssh2::Prompt<'a>],
    ) -> Vec<String> {
        vec![self.password.clone()]
    }
}
