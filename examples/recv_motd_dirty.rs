use rust_mc_proto::{DataReader, DataWriter, MCConnTcp, Packet, ProtocolError};

fn main() -> Result<(), ProtocolError> {
    let mut conn = MCConnTcp::connect("localhost:25565")?; // connecting

    conn.write_packet(&Packet::build(0x00, |packet| {
        packet.write_u16_varint(765)?; // protocol_version
        packet.write_string("localhost")?; // server_address
        packet.write_unsigned_short(25565)?; // server_port
        packet.write_u8_varint(1) // next_state
    })?)?; // handshake packet

    conn.write_packet(&Packet::empty(0x00))?; // status request packet

    Ok(println!("motd: {}", conn.read_packet()?.read_string()?)) // status response packet
}
