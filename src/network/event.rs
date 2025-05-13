use std::error::Error;

use bytes::Bytes;
use libp2p::{
    Multiaddr, PeerId, Swarm,
    futures::StreamExt as _,
    gossipsub::{self, TopicHash},
    kad::{self, Record},
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
    peer_address: Option<Multiaddr>,
    bootstrap: Option<PeerId>,
    topic: Topic,
) -> Result<(), Box<dyn Error>> {
    match role {
        Role::BootstapNode => loop {
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

                        let message_bytes: Bytes = message.into();

                        let query_id = swarm.behaviour_mut().kademlia.put_record(
                            Record::new("key".as_bytes().to_vec(), message_bytes.to_vec()),
                            kad::Quorum::All,
                        )?;

                        tracing::info!("Record sent: {query_id}");

                        let query_id_closest = swarm
                            .behaviour_mut()
                            .kademlia
                            .get_closest_peers("key".as_bytes().to_vec());

                        tracing::info!("Record sent closest: {query_id_closest}");
                    }
                    SwarmEvent::ConnectionEstablished {
                        peer_id, endpoint, ..
                    } => {
                        swarm
                            .behaviour_mut()
                            .kademlia
                            .add_address(&peer_id, endpoint.get_remote_address().clone());
                    }
                    SwarmEvent::Behaviour(PeerBehaviorEvent::Kademlia(
                        kad::Event::InboundRequest { request },
                    )) => {
                        tracing::info!("{request:?}")
                    }
                    SwarmEvent::Behaviour(PeerBehaviorEvent::Kademlia(
                        kad::Event::OutboundQueryProgressed { result, stats, .. },
                    )) => {
                        tracing::info!("Kad event result: {result:?}, with {stats:?}")
                    }
                    SwarmEvent::Behaviour(PeerBehaviorEvent::Kademlia(
                        kad::Event::RoutingUpdated {
                            peer,
                            is_new_peer,
                            addresses,
                            old_peer,
                            ..
                        },
                    )) => {
                        tracing::info!(
                            "Routing updated: with {peer}, is it new? {is_new_peer}. \n {old_peer:?} kicked out"
                        );
                        tracing::info!("known address: {addresses:#?}")
                    }
                    _ => {}
                }
            }
        },
        Role::Sender => {
            if let Some(addr) = peer_address {
                swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&bootstrap.unwrap(), addr.clone()); // TODO: remove unwrap

            // if let Err(err) = swarm.dial(addr.clone()) {
            //     tracing::error!("Dialing peer address: {} fails. reason: {}", addr, err);
            // }
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
                            let query_id = swarm.behaviour_mut().kademlia.put_record(
                                Record::new("key".as_bytes().to_vec(), message.data.clone()),
                                kad::Quorum::All,
                            )?;

                            tracing::info!("Record sent: {query_id}");

                            let query_id_closest = swarm.behaviour_mut().kademlia.get_closest_peers("key".as_bytes().to_vec());

                            tracing::info!("Record sent closest: {query_id_closest}");

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
                        SwarmEvent::Behaviour(PeerBehaviorEvent::Kademlia(
                            kad::Event::InboundRequest { request },
                        )) => {
                            tracing::info!("{request:?}")
                        }
                        SwarmEvent::Behaviour(PeerBehaviorEvent::Kademlia(
                            kad::Event::OutboundQueryProgressed { result, stats, .. }
                        )) => {
                            tracing::info!("Kad event result: {result:?}, with {stats:?}")
                        }
                        SwarmEvent::Behaviour(PeerBehaviorEvent::Kademlia(
                            kad::Event::RoutingUpdated {
                                peer, is_new_peer, addresses, old_peer, ..
                            },
                        )) => {
                            tracing::info!("Routing updated: with {peer}, is it new? {is_new_peer}. \n {old_peer:?} kicked out" );
                            tracing::info!("known address: {addresses:#?}")
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
