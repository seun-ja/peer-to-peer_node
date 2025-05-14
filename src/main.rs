use std::{error::Error, net::Ipv4Addr, time::Duration};

use clap::Parser as _;
use libp2p::{
    Multiaddr, PeerId,
    kad::{self, Mode, store::MemoryStore},
    mdns,
    multiaddr::Protocol,
    noise, tcp, yamux,
};
use peer_node::{
    cli::Args,
    network::{behaviour::PeerBehavior, event::event_runner},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    peer_node::tracing::init("info");
    let args: Args = Args::parse();

    let ip_addr: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);

    let peer_multi_addr: Multiaddr = Multiaddr::from(ip_addr).with(Protocol::Tcp(0));

    tracing::info!("A {}", args.role);

    let mut swarm: libp2p::Swarm<PeerBehavior> = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|keypair| {
            let peer_id: libp2p::PeerId = keypair.public().to_peer_id();
            let store: MemoryStore = MemoryStore::new(peer_id);
            let kademlia: kad::Behaviour<MemoryStore> = kad::Behaviour::new(peer_id, store);

            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)
                .expect("mDNS initiation fails");

            PeerBehavior { kademlia, mdns }
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    // Roles to transition
    match args.role {
        peer_node::cli::Role::BootstapNode => {
            swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server))
        }
        peer_node::cli::Role::Sender => swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Client)),
    }

    swarm.listen_on(peer_multi_addr)?;

    let _bootstrap_peer_id: Option<PeerId> = if let Some(bootstrap_peer_id) = args.bootstrap_peer_id
    {
        Some(bootstrap_peer_id.parse()?)
    } else {
        None
    };

    let _bootstrap_peer_mutli_address: Option<Multiaddr> =
        if let Some(peer_mutli_address) = args.peer_mutli_address {
            Some(peer_mutli_address.parse()?)
        } else {
            None
        };

    event_runner(swarm).await
}
