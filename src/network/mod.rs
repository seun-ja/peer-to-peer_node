pub mod behaviour;
pub mod event;

use libp2p::{Multiaddr, PeerId};

pub struct Peer {
    _id: PeerId,
    _addr: Multiaddr,
}
