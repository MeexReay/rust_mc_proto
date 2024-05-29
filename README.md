# rust_mc_proto
lightweight minecraft packets protocol support in pure rust \
has compression (`MinecraftConnection::set_compression`) \
all types of packets you can find on [wiki.vg](https://wiki.vg/) \
[on crates](https://crates.io/crates/rust_mc_proto)
[on github](https://github.com/MeexReay/rust_mc_proto)

## how to use it

for reference:
```rust
pub type MCConn<T> = MinecraftConnection<T>;
pub type MCConnTcp = MinecraftConnection<TcpStream>;
```

example of receiving motd: (`cargo run --example recv_motd`)
```rust
use rust_mc_proto::{Packet, ProtocolError, MCConnTcp, DataBufferReader, DataBufferWriter};

/*

    Example of receiving motd from the server
    Sends handshake, status request and receiving one

*/

fn send_handshake(conn: &mut MCConnTcp,
                protocol_version: u16,
                server_address: &str,
                server_port: u16,
                next_state: u8) -> Result<(), ProtocolError> {
    let mut packet = Packet::empty(0x00);

    packet.write_u16_varint(protocol_version)?;
    packet.write_string(server_address)?;
    packet.write_unsigned_short(server_port)?;
    packet.write_u8_varint(next_state)?;

    conn.write_packet(&packet)?;

    Ok(())
}

fn send_status_request(conn: &mut MCConnTcp) -> Result<(), ProtocolError> {
    let packet = Packet::empty(0x00);
    conn.write_packet(&packet)?;

    Ok(())
}

fn read_status_response(conn: &mut MCConnTcp) -> Result<String, ProtocolError> {
    let mut packet = conn.read_packet()?;

    packet.read_string()
}

fn main() {
    let mut conn = MCConnTcp::connect("localhost:25565").unwrap();

    send_handshake(&mut conn, 765, "localhost", 25565, 1).unwrap();
    send_status_request(&mut conn).unwrap();

    let motd = read_status_response(&mut conn).unwrap();

    println!("Motd: {}", motd);
}
```

example of simple server that only send motd and ping (`cargo run --example status_server`)
```rust
use std::{net::TcpListener, sync::{Arc, Mutex}, thread};

use rust_mc_proto::{DataBufferReader, DataBufferWriter, MCConn, MCConnTcp, MinecraftConnection, Packet, ProtocolError};

/*

    Example of simple server that sends motd 
    to client like a vanilla minecraft server

*/

struct MinecraftServer {
    server_ip: String,
    server_port: u16,
    protocol_version: u16,
    motd: String
}

impl MinecraftServer {
    fn new(server_ip: &str,
            server_port: u16,
            protocol_version: u16,
            motd: &str) -> Self {
        MinecraftServer {
            server_ip: server_ip.to_string(),
            server_port,
            protocol_version,
            motd: motd.to_string()
        }
    }
}

fn accept_client(mut conn: MCConnTcp, server: Arc<Mutex<MinecraftServer>>) -> Result<(), ProtocolError> {
    let mut handshake = false;
    
    loop {
        let mut packet = match conn.read_packet() {
            Ok(i) => i,
            Err(_) => { break; },
        };

        if handshake {
            if packet.id == 0x00 {
                let mut status = Packet::empty(0x00);
                status.write_string(&server.lock().unwrap().motd)?;
                conn.write_packet(&status)?;
            } else if packet.id == 0x01 {
                let mut status = Packet::empty(0x01);
                status.write_long(packet.read_long()?)?;
                conn.write_packet(&status)?;
            }
        } else if packet.id == 0x00 {
            let protocol_version = packet.read_u16_varint()?;
            let server_address = packet.read_string()?;
            let server_port = packet.read_unsigned_short()?;
            let next_state = packet.read_u8_varint()?;

            if next_state != 1 { break; }

            println!("Client handshake info:");
            println!("  IP: {}", conn.stream.peer_addr().unwrap());
            println!("  Protocol version: {}", protocol_version);
            println!("  Server address: {}", server_address);
            println!("  Server port: {}", server_port);

            handshake = true;
        } else {
            break;
        }
    }

    conn.close();

    Ok(())
}

fn main() {
    let server = MinecraftServer::new(
        "localhost", 
        25565, 
        765,
        "{}" // TODO: write real motd
    );

    let addr = server.server_ip.clone() + ":" + &server.server_port.to_string();
    let listener = TcpListener::bind(addr).unwrap();
    let server = Arc::new(Mutex::new(server));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let local_server = server.clone();

        thread::spawn(move || {
            accept_client(MinecraftConnection::new(stream), local_server).unwrap();
        });
    }
}
```

also you can get minecraft connection from any stream: `MinecraftConnection::from_stream`

I think this crate can be used for a server on rust idk -_-