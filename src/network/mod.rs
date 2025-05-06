pub mod behaviour;
pub mod event;

use libp2p::{Multiaddr, PeerId};

use crate::utils::error::Error;

/// Nodes Peer
pub struct Peer {
    pub id: PeerId,
    pub addr: Multiaddr,
    pub setup: bool,
}

impl Peer {
    pub fn new(id: String, addr: String) -> Result<Self, Error> {
        Ok(Peer {
            id: id
                .parse::<PeerId>()
                .map_err(|err| Error::InvalidPeerId(err.to_string()))?,
            addr: addr
                .parse::<Multiaddr>()
                .map_err(|err| Error::InvalidMultiaddr(err.to_string()))?,
            setup: false,
        })
    }
}
