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
        "localhost", 
        25565, 
        765,
        "{\"version\":{\"protocol\":765,\"name\":\"Куриный ништяк\"},\"players\":{\"online\":0,\"max\":1},\"description\":{\"extra\":[{\"extra\":[{\"color\":\"aqua\",\"text\":\"☄\"},\"  \",{\"bold\":true,\"extra\":[{\"color\":\"#00D982\",\"text\":\"S\"},{\"color\":\"#0DCB8B\",\"text\":\"l\"},{\"color\":\"#1BBC93\",\"text\":\"o\"},{\"color\":\"#28AE9C\",\"text\":\"g\"},{\"color\":\"#35A0A5\",\"text\":\"a\"},{\"color\":\"#4392AE\",\"text\":\"n\"},{\"color\":\"#5083B6\",\"text\":\"M\"},{\"color\":\"#5D75BF\",\"text\":\"C\"},{\"color\":\"#6A67C8\",\"text\":\".\"},{\"color\":\"#7858D0\",\"text\":\"r\"},{\"color\":\"#854AD9\",\"text\":\"u\"}],\"text\":\"\"},\"  \",{\"color\":\"aqua\",\"text\":\"☄\"},\"  \"],\"text\":\"\"},\"\\n\",{\"extra\":[{\"extra\":[{\"bold\":true,\"color\":\"#A1999E\",\"text\":\"░\"},\"    \",{\"color\":\"#D1C7CD\",\"text\":\"прикол\"},\"    \",{\"bold\":true,\"color\":\"#A1999E\",\"text\":\"░\"}],\"text\":\"\"}],\"text\":\"                    \"}],\"text\":\"                   \"},\"favicon\":\"data:image/png;base64,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\\u003d\"}"
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
