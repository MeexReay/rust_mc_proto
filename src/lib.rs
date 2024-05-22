use crate::protocol::packets::{
    login::{
        login_c03_login_acknowledged::LoginC03LoginAcknowledged,
        login_s02_login_success::LoginS02LoginSuccess,
        login_s03_set_compression::LoginS03SetCompression,
    },
    packet::WriteablePacket,
};
use protocol::packets::{
    c00_handshake::C00Handshake, login::login_c00_login_start::LoginC00LoginStart,
};
use rust_mc_proto::{DataBufferReader, MCConnTcp};
pub mod protocol;

#[derive(PartialEq, Clone, Copy)]
pub enum ProtocolState {
    HANDSHAKE,
    STATUS,
    LOGIN,
    CONFIGURATION,
}

pub struct Bot {
    pub connection: MCConnTcp,
    pub protocol_state: ProtocolState,
}

impl Bot {
    pub fn new(connection: MCConnTcp) -> Bot {
        return Bot {
            connection,
            protocol_state: ProtocolState::HANDSHAKE,
        };
    }
}

fn main() {
    let connection = MCConnTcp::connect("192.168.0.14:25565").unwrap();

    let mut bot = Bot::new(connection);

    let result = C00Handshake::new(764, "192.168.0.14", 25565, ProtocolState::LOGIN).send(&mut bot);

    if result.is_err() {
        println!("Failed to send handshake: {}", result.unwrap_err());
        return;
    }

    let result = LoginC00LoginStart::new("NotABot", 0_u128).send(&mut bot);

    if result.is_err() {
        println!("Failed to send status request: {}", result.unwrap_err());
        return;
    }

    loop {
        let mut packet = match bot.connection.read_packet() {
            Ok(packet) => packet,
            Err(err) => {
                println!("Protocol error: {:?}", err);
                return;
            }
        };

        println!("Recvied packet: {}", packet.id);

        if bot.protocol_state == ProtocolState::LOGIN {
            if packet.id == 0x03 {
                let parsed_packet = match LoginS03SetCompression::new(&mut packet) {
                    Ok(packet) => packet,
                    Err(err) => {
                        println!("Protocol error (Login.SetCompression): {:?}", err);
                        return;
                    }
                };
                println!("Enabling compression ({})", parsed_packet.threshold);
                bot.connection.set_compression(parsed_packet.threshold)
            } else if packet.id == 0x02 {
                let _ = match LoginS02LoginSuccess::new(&mut packet) {
                    Ok(packet) => packet,
                    Err(err) => {
                        println!("Protocol error (Login.SetCompression): {:?}", err);
                        return;
                    }
                };

                bot.protocol_state = ProtocolState::CONFIGURATION;
                let result: Result<(), String> = LoginC03LoginAcknowledged::new().send(&mut bot);

                if result.is_err() {
                    println!("Failed to send status request: {}", result.unwrap_err());
                    return;
                }
            } else if bot.protocol_state == ProtocolState::CONFIGURATION {
                if packet.id == 0x01 {
                    println!("FUCK");
                    let reason = match packet.read_string() {
                        Ok(reason) => reason,
                        Err(_) => return,
                    };
                    println!("{}", reason)
                } else if packet.id == 0x02 {
                    println!("Server stopped yapping");
                }
            }
        }
    }
}
