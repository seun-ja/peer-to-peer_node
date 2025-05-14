use std::error::Error;

use libp2p::{
    Swarm,
    futures::StreamExt as _,
    kad::{self, Record, RecordKey},
    swarm::SwarmEvent,
};
use tokio::{
    io::{self, AsyncBufReadExt as _},
    select,
};

use super::behaviour::{PeerBehavior, PeerBehaviorEvent};

pub async fn event_runner(mut swarm: Swarm<PeerBehavior>) -> Result<(), Box<dyn Error>> {
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                let mut args = line.split(' ');

                match args.next() {
                    Some("GET") => {
                        swarm.behaviour_mut().kademlia.get_record(RecordKey::new(b"key"));
                    },
                    Some("RECORD") => {
                        if let Some(value) = args.next() {
                            swarm.behaviour_mut().kademlia.put_record(
                                Record::new("key".as_bytes().to_vec(), value.as_bytes().to_vec()),
                                kad::Quorum::One,
                            )?;
                        }
                    }
                    _ => {}
                }
            }

            Some(event) = swarm.next() =>
                match event {
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
                    SwarmEvent::Behaviour(PeerBehaviorEvent::Mdns(
                        libp2p::mdns::Event::Discovered(list),
                    )) => {
                        for (peer_id, multi_addr) in list {
                            swarm
                                .behaviour_mut()
                                .kademlia
                                .add_address(&peer_id, multi_addr);
                        }
                    }
                    SwarmEvent::Behaviour(PeerBehaviorEvent::Kademlia(
                        kad::Event::OutboundQueryProgressed { result, .. },
                    )) => {
                        match result {
                            kad::QueryResult::GetProviders(Err(err)) => {
                                eprintln!("Failed to get providers: {err:?}");
                            }
                            kad::QueryResult::GetRecord(Ok(
                                kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                                    record: kad::Record { key, value, .. },
                                    ..
                                })
                            )) => {
                                tracing::info!(
                                    "Got record {:?} {:?}",
                                    std::str::from_utf8(key.as_ref())?,
                                    std::str::from_utf8(&value)?, // TODO: #10 `BUG` error handling would break program
                                );
                            }
                            kad::QueryResult::GetRecord(Ok(_)) => {}
                            kad::QueryResult::GetRecord(Err(err)) => {
                                tracing::info!("Failed to get record: {err:?}");
                            }
                            kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                                tracing::info!(
                                    "Successfully put record {:?}",
                                    std::str::from_utf8(key.as_ref())?
                                );
                            }
                            kad::QueryResult::PutRecord(Err(err)) => {
                                tracing::info!("Failed to put record: {err:?}");
                            }
                            kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
                                tracing::info!(
                                    "Successfully put provider record {:?}",
                                    std::str::from_utf8(key.as_ref())?
                                );
                            }
                            kad::QueryResult::StartProviding(Err(err)) => {
                                eprintln!("Failed to put provider record: {err:?}");
                            }
                            _ => {}
                        }
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
    }
}
