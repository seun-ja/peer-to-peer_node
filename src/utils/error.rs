#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid peer ID: {0}")]
    InvalidPeerId(String),
    #[error("Invalid multiaddr: {0}")]
    InvalidMultiaddr(String),
}
