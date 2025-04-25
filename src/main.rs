use std::{
    io::{self, Read, Write as _},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
};

use clap::Parser as _;
use peer_node::{
    cli::{Args, Role},
    comms::message::Message,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    peer_node::tracing::init("info");
    let args = Args::parse();

    let address = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), args.port);
    let listerner = TcpListener::bind(address)?;

    tracing::info!("Node address: {}", address);
    tracing::info!("A {}", args.role);

    match args.role {
        Role::Receiver => loop {
            for mut incoming_stream in listerner.incoming().flatten() {
                let mut msg = [0; 5];
                let _byte_count = incoming_stream.read(&mut msg)?;

                let msg: Message = String::from_utf8_lossy(&msg).trim().to_string().into();

                // If it's a rememberMe, store to some DHT and if Comms: Act as instructed

                tracing::info!("Message received: {msg:?}");
            }
        },
        Role::Sender => {
            let mut msg = String::new();
            io::stdin().read_line(&mut msg)?;

            tracing::info!("Sending: {msg}");

            let mut outgoing_stream = TcpStream::connect(args.address)?;

            let msg: Message = msg.into();

            outgoing_stream.write_all(msg.to_string().as_bytes())?;

            // Wait for the message to be sent before exiting
            outgoing_stream.flush()?;

            Ok(())
        }
    }
}
