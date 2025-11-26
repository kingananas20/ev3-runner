mod clientsession;
mod verification;
mod version;

use crate::{
    cli::{Action, Client},
    client::clientsession::ClientSession,
    protocol::{self},
};
use clientsession::ClientError;

pub fn client(config: Client) -> Result<(), ClientError> {
    let (args, action) = match config.action {
        Action::Upload(args) => (args, protocol::Action::Upload),
        Action::Run(args) => (args, protocol::Action::Run),
    };

    let mut session = ClientSession::connect(args, action)?;
    session.dispatch()?;

    Ok(())
}
