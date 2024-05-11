use rust_mc_proto::{Connection, Packet, ProtocolError};
use varint_rs::{VarintReader, VarintWriter};
use std::io::Write;

fn send_handshake(conn: &mut Connection,
                protocol_version: u16,
                server_address: &str,
                server_port: u16,
                next_state: u8) -> Result<(), ProtocolError> {
    let mut packet = Packet::empty(0x00);
    packet.write_u16_varint(protocol_version)?;
    packet.write_string(server_address.to_string())?;
    packet.write_unsigned_short(server_port)?;
    packet.write_u8_varint(next_state)?;

    packet.buffer.set_rpos(0);
    packet.buffer.set_wpos(0);

    conn.write_packet(&packet)
}

fn send_status_request(conn: &mut Connection) -> Result<(), ProtocolError> {
    conn.write_packet(&Packet::empty(0x00))
}

fn read_status_response(conn: &mut Connection) -> Result<String, ProtocolError> {
    let mut packet = conn.read_packet()?;
    packet.read_string()
}

fn main() {
    let mut conn = Connection::new("localhost:25565");

    send_handshake(&mut conn, 765, "localhost", 25565, 1).unwrap();
    send_status_request(&mut conn).unwrap();
    println!("{}", read_status_response(&mut conn).unwrap());
}
