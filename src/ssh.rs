use ssh2::{Channel, DisconnectCode, KeyboardInteractivePrompt, Session, Sftp};
use std::convert::Infallible;
use std::fs::File;
use std::hash::Hasher;
use std::io;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream as StdTcpStream;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;
use tracing::{info, warn};
use twox_hash::XxHash64;

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
        src_path: &str,
        dst_path: &str,
    ) -> Result<BufReader<Channel>, SessionError> {
        self.session.set_compress(true);
        if !self.same_checksum(src_path, dst_path)? {
            self.upload(src_path, dst_path)?;
        }

        let reader = self.exec(dst_path)?;

        Ok(reader)
    }

    #[tracing::instrument(skip(self))]
    fn same_checksum(&mut self, src_path: &str, dst_path: &str) -> Result<bool, SessionError> {
        let sftp = self.get_sftp()?;
        let Ok(remote_file) = sftp.open(dst_path) else {
            info!("File does not exist. Checksum false");
            return Ok(false);
        };
        let mut reader = BufReader::new(remote_file);
        let mut server_hasher = XxHash64::with_seed(0);
        let mut buf = [0u8; 256 * 1024];

        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            server_hasher.write(&buf[..n]);
        }
        let server_checksum = server_hasher.finish();

        let file = File::open(src_path)?;
        let mut reader = BufReader::new(file);
        let mut client_hasher = XxHash64::with_seed(0);
        let mut buf = [0u8; 256 * 1024];

        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            client_hasher.write(&buf[..n]);
        }
        let client_checksum = client_hasher.finish();

        let result = server_checksum == client_checksum;

        info!("Same checksums? {result}");
        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    fn upload(&mut self, src_path: &str, dst_path: &str) -> Result<(), SessionError> {
        let instant = Instant::now();

        let local_file = File::open(src_path)?;
        let mut reader = BufReader::new(local_file);

        let sftp = self.get_sftp()?;
        let dst_path = Path::new(dst_path);
        if let Some(parent) = dst_path.parent() {
            let mut current = PathBuf::new();
            for component in parent.components() {
                current = current.join(component);
                sftp.mkdir(&current, 0o755).ok();
            }
        }

        let remote_file = sftp.create(dst_path)?;
        let mut writer = BufWriter::new(remote_file);

        let mut buffer = [0u8; 256 * 1024];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            writer.write_all(&buffer[..n])?;
        }

        writer.flush()?;

        let mut stat = sftp.stat(dst_path)?;
        stat.perm = Some(0o755);
        sftp.setstat(dst_path, stat)?;

        info!("File uploaded in {:?}", instant.elapsed());

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    fn exec(&mut self, filepath: &str) -> Result<BufReader<Channel>, SessionError> {
        info!("Executing {filepath}");
        let mut channel = self.get_channel()?;

        channel.exec(&format!("./{filepath}"))?;
        let reader = BufReader::new(channel);

        Ok(reader)
    }

    fn get_channel(&mut self) -> Result<Channel, SessionError> {
        Ok(self.session.channel_session()?)
    }

    fn get_sftp(&mut self) -> Result<Sftp, SessionError> {
        Ok(self.session.sftp()?)
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
