use std::fmt::Display;

use clap::{Parser, arg};
use clap_derive::ValueEnum;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Node's role
    #[arg(short, long, default_value_t = Role::Sender)]
    pub role: Role,

    /// Peer's MultiAddress
    #[arg(short)]
    pub peer_mutli_address: Option<String>,

    /// Bootstrap Nodes
    #[arg(short)]
    pub bootstrap_peer_id: Option<String>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Role {
    BootstapNode,
    Sender,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Role::BootstapNode => write!(f, "Receiver"),
            Role::Sender => write!(f, "Sender"),
        }
    }
}
