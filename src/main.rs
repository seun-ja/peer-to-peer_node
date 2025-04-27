use std::{error::Error, io, net::Ipv4Addr, time::Duration};

use clap::Parser as _;
use libp2p::{
    Multiaddr, futures::StreamExt, multiaddr::Protocol, noise, ping, swarm::SwarmEvent, tcp, yamux,
};
use peer_node::{
    cli::{Args, Role},
    comms::message::Message,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    peer_node::tracing::init("info");
    let args = Args::parse();

    let ip_addr = Ipv4Addr::new(127, 0, 0, 1);

    let peer_multi_addr = Multiaddr::from(ip_addr).with(Protocol::Tcp(args.port));

    tracing::info!("Peer addr: {peer_multi_addr}");
    tracing::info!("A {}", args.role);

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_behaviour| ping::Behaviour::default())?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    swarm.listen_on(peer_multi_addr)?;

    match swarm.next().await {
        Some(event) => {
            tracing::info!("Event: {event:?}");
            match event {
                SwarmEvent::NewListenAddr {
                    listener_id,
                    address,
                } => {
                    tracing::info!("Listening with listener {listener_id} on {address}");
                }
                SwarmEvent::ListenerClosed {
                    listener_id,
                    addresses,
                    reason,
                } => {
                    tracing::info!(
                        "Listener with listener {listener_id} closed for listening to  {addresses:?}, reason: {reason:?}"
                    );
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
                _ => {}
            }
        }
        None => {}
    }

    match args.role {
        Role::Receiver => todo!(),
        // loop {
        // let mut msg = [0; 5];
        // let _byte_count = incoming_stream.read(&mut msg)?;

        // let msg: Message = String::from_utf8_lossy(&msg).trim().to_string().into();

        // If it's a rememberMe, store to some DHT and if Comms: Act as instructed

        // tracing::info!("Message received: {msg:?}");
        // },
        Role::Sender => {
            let mut msg = String::new();
            io::stdin().read_line(&mut msg)?;

            tracing::info!("Sending: {msg}");

            let _msg: Message = msg.into();
            Ok(())
        }
    }
}
