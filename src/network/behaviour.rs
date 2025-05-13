use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, kad};

#[derive(NetworkBehaviour)]
pub struct PeerBehavior {
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
}

pub struct PeerDht {
    pub kademlia: kad::PeerRecord,
}
