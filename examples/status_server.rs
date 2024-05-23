use std::{net::TcpListener, sync::{Arc, Mutex}, thread};

use rust_mc_proto::{DataBufferReader, DataBufferWriter, MCConn, MCConnTcp, MinecraftConnection, Packet, ProtocolError};

/*

    Example of simple server that sends motd 
    to client like an vanilla minecraft server

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
            Err(_) => { 
                break; 
            },
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
            let protocol_version = packet.read_i32_varint()?;
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
        "127.0.0.1", 
        25565, 
        765,
        "{
            \"version\":{
                \"protocol\":765,
                \"name\":\"Version name\"
            },
            \"players\":{
                \"online\":0,
                \"max\":1,
                \"sample\":[
                    {
                        \"uuid\": \"\",
                        \"name\": \"Notch\"
                    }
                ]
            },
            \"description\": {
                \"text\": \"Hello World!\",
                \"color\": \"red\",
                \"bold\": true
            },
            \"favicon\": \"data:image/png;base64,R0lGODlhAQABAIAAAP///wAAACwAAAAAAQABAAACAkQBADs=\"
        }"
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
