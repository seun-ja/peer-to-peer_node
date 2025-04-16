use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

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

    for _incoming_stream in listerner.incoming().flatten() {
        // incoming_stream
    }

    // loop {}
    Ok(())
}
