use libp2p::swarm::NetworkBehaviour;
use libp2p::{kad, mdns};

#[derive(NetworkBehaviour)]
pub struct PeerBehavior {
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
}
