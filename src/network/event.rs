use std::error::Error;

use libp2p::{
    Multiaddr, Swarm,
    futures::StreamExt as _,
    gossipsub::{self, TopicHash},
    kad,
    swarm::SwarmEvent,
};
use tokio::{
    io::{self, AsyncBufReadExt as _},
    select,
};

use crate::{cli::Role, comms::message::Message};

use super::behaviour::{PeerBehavior, PeerBehaviorEvent};

pub async fn event_runner(
    mut swarm: Swarm<PeerBehavior>,
    role: Role,
    peer_address: Option<String>,
    _bootstrap: Option<String>,
    topic: Topic,
) -> Result<(), Box<dyn Error>> {
    match role {
        Role::Receiver => loop {
            if let Some(event) = swarm.next().await {
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
                    SwarmEvent::Behaviour(PeerBehaviorEvent::Gossipsub(
                        gossipsub::Event::Message {
                            propagation_source,
                            message,
                            ..
                        },
                    )) => {
                        let message: Message =
                            String::from_utf8_lossy(&message.data).to_string().into();

                        tracing::info!(
                            "Got a message: {message} from PeerId: {propagation_source}",
                        );
                    }
                    SwarmEvent::Behaviour(PeerBehaviorEvent::Kademlia(
                        kad::Event::InboundRequest { request },
                    )) => {
                        tracing::info!("{request:?}")
                    }
                    _ => {}
                }
            }
        },
        Role::Sender => {
            if let Some(addr) = peer_address {
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
                        SwarmEvent::Behaviour(PeerBehaviorEvent::Gossipsub(
                            gossipsub::Event::Message {
                                propagation_source,
                                message,
                                ..
                            },
                        )) => {
                            let message: Message =
                                String::from_utf8_lossy(&message.data).to_string().into();

                            tracing::info!(
                                "Got a message: {message} from PeerId: {propagation_source}",
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
                        _ => {}
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Topic(pub String);

impl From<Topic> for TopicHash {
    fn from(val: Topic) -> Self {
        TopicHash::from_raw(&val.0)
    }
}
