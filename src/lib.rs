pub mod cli;
mod client;
mod hash;
pub mod protocol;
mod server;
mod transport;

pub use crate::client::client;
pub use server::server;

const BUFFER_SIZE: usize = 64 * 1024;

// Logging
use tracing::Level;
use tracing_subscriber::fmt::SubscriberBuilder;

pub fn setup_logging(verbosity: u8) {
    let subscriber = SubscriberBuilder::default();
    let subscriber = match verbosity {
        0 => subscriber.with_max_level(Level::WARN),
        1 => subscriber.with_max_level(Level::INFO),
        2 => subscriber.with_max_level(Level::DEBUG),
        3.. => subscriber.with_max_level(Level::TRACE),
    };
    let subscriber = subscriber.with_file(true).with_line_number(true);
    subscriber.init();
}
