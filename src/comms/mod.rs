use libp2p::{Multiaddr, PeerId};

pub mod message;

pub struct Peer {
    _id: PeerId,
    _addr: Multiaddr,
}
