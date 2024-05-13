use std::{net::TcpListener, sync::Arc, thread};

use rust_mc_proto::{DataBufferReader, DataBufferWriter, MCConn, MCConnTcp, MinecraftConnection, Packet, ProtocolError};

/*

    Example of simple server that sends motd 
    to client like an vanilla minecraft server

*/

fn accept_client(mut conn: MCConnTcp) {
    loop { 
        let packet = match conn.read_packet() {
            Ok(p) => p,
            Err(_) => { break },
        };

        dbg!(packet);
    }
}

fn main() {
    let listener = TcpListener::bind("localhost:25565").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();


        thread::spawn(move || {
            accept_client(MinecraftConnection::new(stream));
        });
    }
}
