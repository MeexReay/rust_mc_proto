# rust_mc_proto
lightweight minecraft packets protocol support in pure rust \
has compression (`MinecraftConnection::set_compression`) \
all types of packets you can find on [wiki.vg](https://wiki.vg/) \
[crates](https://crates.io/crates/rust_mc_proto)
[github](https://github.com/MeexReay/rust_mc_proto)

## setup

stable

```toml
rust_mc_proto = "0.1.15"
```

unstable

```toml
rust_mc_proto = { git = "https://github.com/MeexReay/rust_mc_proto" }
```

features:
- atomic_compression (default)

## how to use it

for reference:
```rust
pub type MCConn<T> = MinecraftConnection<T>;
pub type MCConnTcp = MinecraftConnection<TcpStream>;
```

example of receiving motd:

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
    let mut conn = MCConnTcp::connect("sloganmc.ru:25565").unwrap();

    send_handshake(&mut conn, 765, "sloganmc.ru", 25565, 1).unwrap();
    send_status_request(&mut conn).unwrap();

    let motd = read_status_response(&mut conn).unwrap();

    dbg!(motd);
}
```

[more examples](https://github.com/MeexReay/rust_mc_proto/tree/main/examples)

this crate can be used for a server on rust idk -_-