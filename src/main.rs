use std::{error::Error, net::Ipv4Addr, time::Duration};

use clap::Parser as _;
use libp2p::{
    Multiaddr, futures::StreamExt, multiaddr::Protocol, noise, ping, swarm::SwarmEvent, tcp, yamux,
};
use peer_node::cli::{Args, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    peer_node::tracing::init("info");
    let args = Args::parse();

    let ip_addr = Ipv4Addr::new(0, 0, 0, 0);

    let peer_multi_addr = Multiaddr::from(ip_addr).with(Protocol::Tcp(0));
    // .with(Protocol::Udp(args.port))
    // .with(Protocol::QuicV1);

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
                        tracing::info!(
                            "Connection {connection_id} with peer {peer_id} established, endpoint: {endpoint:?}, num_established: {num_established}, concurrent_dial_errors: {concurrent_dial_errors:?}, established_in: {established_in:?}"
                        );
                    }
                    SwarmEvent::Behaviour(behaviour) => {
                        tracing::info!("Behaviour event: {:?}", behaviour);
                    }
                    _ => {}
                }
            }
        },
        // loop {
        // let mut msg = [0; 5];
        // let _byte_count = incoming_stream.read(&mut msg)?;

        // let msg: Message = String::from_utf8_lossy(&msg).trim().to_string().into();

        // If it's a rememberMe, store to some DHT and if Comms: Act as instructed

        // tracing::info!("Message received: {msg:?}");
        // },
        Role::Sender => {
            if let Some(addr) = args.peer_address {
                let peer_addr: Multiaddr = addr.parse()?;
                if let Err(err) = swarm.dial(peer_addr) {
                    tracing::info!("Dialing peer address: {} fails. reason: {}", addr, err);
                }
            } else {
                tracing::error!("No peer address provided");
            }

            // let mut msg = String::new();
            // io::stdin().read_line(&mut msg)?;

            // tracing::info!("Sending: {msg}");

            // let _msg: Message = msg.into();
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {address:?}")
                    }
                    SwarmEvent::Behaviour(event) => println!("{event:?}"),
                    _ => {}
                }
            }
        }
    }
}
