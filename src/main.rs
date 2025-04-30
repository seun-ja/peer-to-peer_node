use std::{error::Error, net::Ipv4Addr, time::Duration};

use clap::Parser as _;
use libp2p::{
    Multiaddr,
    futures::StreamExt,
    gossipsub::{self, AllowAllSubscriptionFilter, Config, IdentityTransform, MessageAuthenticity},
    kad::{self, store::MemoryStore},
    multiaddr::Protocol,
    noise,
    swarm::SwarmEvent,
    tcp, yamux,
};
use peer_node::{
    behavior::{PeerBehavior, PeerBehaviorEvent},
    cli::{Args, Role},
};
use tokio::{io, io::AsyncBufReadExt, select};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    peer_node::tracing::init("info");
    let args = Args::parse();

    let ip_addr = Ipv4Addr::new(0, 0, 0, 0);

    let peer_multi_addr = Multiaddr::from(ip_addr).with(Protocol::Tcp(0));

    tracing::info!("Peer addr: {peer_multi_addr}");
    tracing::info!("A {}", args.role);

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|keypair| {
            let peer_id = keypair.public().to_peer_id();
            let store = MemoryStore::new(peer_id);
            let kademlia = kad::Behaviour::new(peer_id, store);

            let gossipsub: gossipsub::Behaviour<IdentityTransform, AllowAllSubscriptionFilter> =
                gossipsub::Behaviour::new(
                    MessageAuthenticity::Signed(keypair.clone()),
                    Config::default(),
                )
                .expect("Gossipsub initiation fails");

            PeerBehavior {
                kademlia,
                gossipsub,
            }
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    let topic = gossipsub::IdentTopic::new("peer-network");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    swarm.listen_on(peer_multi_addr)?;

    match args.role {
        Role::Receiver => loop {
            if let Some(event) = swarm.next().await {
                tracing::info!("Received event: {event:?}");
                match event {
                    SwarmEvent::NewListenAddr {
                        listener_id,
                        address,
                    } => {
                        tracing::info!("Listening with listener {listener_id} on {address}");
                    }
                    SwarmEvent::IncomingConnection {
                        connection_id,
                        local_addr,
                        send_back_addr,
                    } => {
                        tracing::info!(
                            "Incoming connection {connection_id} from {local_addr} to {send_back_addr}"
                        );
                    }
                    SwarmEvent::ConnectionClosed {
                        peer_id,
                        connection_id,
                        endpoint,
                        num_established,
                        cause,
                    } => {
                        tracing::info!(
                            "Connection {connection_id} with peer {peer_id} closed, endpoint: {endpoint:?}, num_established: {num_established}, cause: {cause:?}"
                        );
                    }
                    SwarmEvent::ConnectionEstablished {
                        peer_id,
                        connection_id,
                        endpoint,
                        num_established,
                        concurrent_dial_errors,
                        established_in,
                    } => {
                        let addr = endpoint.get_remote_address().clone();
                        swarm.add_peer_address(peer_id, addr.clone());
                        tracing::info!(
                            "Connection {connection_id} with peer {peer_id} established, endpoint: {addr}, num_established: {num_established}, concurrent_dial_errors: {concurrent_dial_errors:?}, established_in: {established_in:?}"
                        );
                    }
                    SwarmEvent::Behaviour(PeerBehaviorEvent::Gossipsub(
                        gossipsub::Event::Message {
                            propagation_source,
                            message_id,
                            message,
                        },
                    )) => {
                        tracing::info!(
                            "Got a message: {message:?} from {propagation_source} with id {message_id}",
                        );
                    }
                    _ => {}
                }
            }
        },
        Role::Sender => {
            if let Some(addr) = args.peer_address {
                let peer_addr: Multiaddr = addr.parse()?;
                if let Err(err) = swarm.dial(peer_addr) {
                    tracing::error!("Dialing peer address: {} fails. reason: {}", addr, err);
                }
            } else {
                tracing::warn!("No peer address provided");
            }

            let mut stdin = io::BufReader::new(io::stdin()).lines();
            loop {
                select! {
                    Ok(Some(line)) = stdin.next_line() => {
                        if let Err(e) = swarm
                            .behaviour_mut().gossipsub
                            .publish(topic.clone(), line.as_bytes()) {
                            tracing::warn!("Publish error: {e:?}");
                        }
                    }
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            tracing::info!("Listening on {address:?}")
                        }
                        SwarmEvent::Behaviour(event) => tracing::info!("{event:?}"),
                        _ => {}
                    }
                }
            }
        }
    }
}
