mod clientsession;
mod validation;
mod version;

use crate::{
    cli::{Action, Client},
    client::clientsession::ClientSession,
    protocol::{self},
};
use clientsession::ClientError;

pub fn client(config: Client) -> Result<(), ClientError> {
    let (action, args) = match config.action {
        Action::Upload(args) => (protocol::Action::Upload, args),
        Action::Run(args) => (protocol::Action::Run(args.brickrun), args),
    };

    let mut session = ClientSession::connect(args, action)?;
    session.dispatch()?;

    Ok(())
}
