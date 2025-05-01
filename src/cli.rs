use std::fmt::Display;

use clap::{Parser, arg};
use clap_derive::ValueEnum;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Node's role
    #[arg(short, long, default_value_t = Role::Sender)]
    pub role: Role,

    /// Peer's address
    #[arg(short)]
    pub peer_address: Option<String>,

    /// Bootstrap Nodes
    #[arg(short)]
    pub bootstrap: Option<String>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Role {
    Receiver,
    Sender,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Role::Receiver => write!(f, "Receiver"),
            Role::Sender => write!(f, "Sender"),
        }
    }
}
