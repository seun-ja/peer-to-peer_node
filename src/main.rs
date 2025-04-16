use std::{
    io::Read,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
};

use clap::Parser as _;
use peer_node::cli::Args;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    peer_node::tracing::init("info");
    let args = Args::parse();

    let address = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), args.port);
    let listerner = TcpListener::bind(address)?;

    tracing::info!("Node address: {}", address);
    tracing::info!("A {}", args.role);

    loop {
        for mut incoming_stream in listerner.incoming().flatten() {
            let mut msg = [0; 16];
            let _byte_count = incoming_stream.read(&mut msg)?;

            tracing::info!("Message received: {:?}", msg);
        }
    }
}
