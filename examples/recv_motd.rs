use rust_mc_proto::{Packet, ProtocolError, MCConnTcp, DataReader, DataWriter};

/*

    Example of receiving motd from the server
    Sends handshake, status request and receiving one

*/

fn send_handshake(
    conn: &mut MCConnTcp,
    protocol_version: u16,
    server_address: &str,
    server_port: u16,
    next_state: u8
) -> Result<(), ProtocolError> {
    conn.write_packet(&Packet::build(0x00, |packet| {
        packet.write_u16_varint(protocol_version)?;
        packet.write_string(server_address)?;
        packet.write_unsigned_short(server_port)?;
        packet.write_u8_varint(next_state)
    })?)
}

fn send_status_request(conn: &mut MCConnTcp) -> Result<(), ProtocolError> {
    conn.write_packet(&Packet::empty(0x00))
}

fn read_status_response(conn: &mut MCConnTcp) -> Result<String, ProtocolError> {
    conn.read_packet()?.read_string()
}

fn main() {
    let mut conn = MCConnTcp::connect("mc.hypixel.net:25565").unwrap();

    send_handshake(&mut conn, 765, "mc.hypixel.net", 25565, 1).unwrap();
    send_status_request(&mut conn).unwrap();

    let motd = read_status_response(&mut conn).unwrap();

    dbg!(motd);
}
