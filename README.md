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

example how to get motd
```rust
use rust_mc_proto::{Packet, ProtocolError, MCConnTcp, DataBufferReader, DataBufferWriter};

fn send_handshake(conn: &mut MCConnTcp,
                protocol_version: u16,
                server_address: &str,
                server_port: u16,
                next_state: u8) -> Result<(), ProtocolError> {
    let mut packet = Packet::empty(0x00);

    packet.write_u16_varint(protocol_version)?;
    packet.write_string(server_address.to_string())?;
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

also you can get minecraft connection from any stream: `MinecraftConnection::from_stream`

I think this crate can be used for a server on rust idk -_-