# rust_mc_proto
Lightweight minecraft packets protocol support in pure rust \
Has compression (`MinecraftConnection::set_compression`) \
This crate can be used for a server on rust idk -_-

## Setup

```toml
rust_mc_proto = "0.1.16" # stable version
rust_mc_proto = { git = "https://github.com/MeexReay/rust_mc_proto" } # unstable version
```

Features:
- `atomic_compression` (default)

## How to use

For reference:
```rust
pub type MCConn<T> = MinecraftConnection<T>;
pub type MCConnTcp = MinecraftConnection<TcpStream>;
```

Example of receiving motd:

```rust
use rust_mc_proto::{Packet, ProtocolError, MCConnTcp, DataBufferReader, DataBufferWriter};

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
    let mut conn = MCConnTcp::connect("sloganmc.ru:25565").unwrap();

    send_handshake(&mut conn, 765, "sloganmc.ru", 25565, 1).unwrap();
    send_status_request(&mut conn).unwrap();

    let motd = read_status_response(&mut conn).unwrap();

    dbg!(motd);
}
```

[More examples](https://github.com/MeexReay/rust_mc_proto/tree/main/examples)

### Contributing

If you would like to contribute to the project, feel free to fork the repository and submit a pull request.

### License
This project is licensed under the WTFPL License