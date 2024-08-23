use std::{net::TcpListener, sync::{Arc, Mutex}, thread};
use rust_mc_proto::{DataBufferReader, DataBufferWriter, MCConnTcp, MinecraftConnection, Packet, ProtocolError};

/*

    Example of simple server that sends motd 
    to client like a vanilla minecraft server

*/

struct MinecraftServer {
    server_ip: String,
    server_port: u16,
    motd: String
}

impl MinecraftServer {
    fn new(server_ip: &str,
            server_port: u16,
            motd: &str) -> Self {
        MinecraftServer {
            server_ip: server_ip.to_string(),
            server_port,
            motd: motd.to_string()
        }
    }

    fn start(self) {
        let addr = self.server_ip.clone() + ":" + &self.server_port.to_string();
        let listener = TcpListener::bind(addr).unwrap();
        let server = Arc::new(Mutex::new(self));

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let local_server = server.clone();

            thread::spawn(move || {
                Self::accept_client(MinecraftConnection::new(stream), local_server).unwrap();
            });
        }
    }

    fn accept_client(mut conn: MCConnTcp, server: Arc<Mutex<MinecraftServer>>) -> Result<(), ProtocolError> {
        let mut handshake = false;
        
        loop {
            let Ok(mut packet) = conn.read_packet() else { break; };
    
            if handshake {
                if packet.id() == 0x00 {
                    let motd = server.lock().unwrap().motd.clone();

                    conn.write_packet(&Packet::build(0x00, |status| 
                        status.write_string(&motd)
                    )?)?;
                } else if packet.id() == 0x01 {
                    conn.write_packet(&Packet::build(0x01, |status| 
                        status.write_long(packet.read_long()?)
                    )?)?;
                }
            } else if packet.id() == 0x00 {
                let protocol_version = packet.read_i32_varint()?;
                let server_address = packet.read_string()?;
                let server_port = packet.read_unsigned_short()?;
                let next_state = packet.read_u8_varint()?;
    
                if next_state != 1 { break; }
    
                println!("Client handshake info:");
                println!("  IP: {}", conn.get_ref().peer_addr().unwrap());
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
}

fn main() {
    let server = MinecraftServer::new(
        "127.0.0.1", 
        25565, 
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
    server.start();
}
