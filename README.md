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
- `atomic_clone` - clone MinecraftConnection like TcpStream. with compression and is_alive fields

## How to use

Example of receiving motd:

```rust
use rust_mc_proto::{DataBufferReader, DataBufferWriter, MCConnTcp, Packet, ProtocolError};

fn main() -> Result<(), ProtocolError> {
    let mut conn = MCConnTcp::connect("mc.hypixel.net:25565")?; // connecting

    conn.write_packet(&Packet::build(0x00, |packet| {
        packet.write_u16_varint(765)?; // protocol_version
        packet.write_string("mc.hypixel.net")?; // server_address
        packet.write_unsigned_short(25565)?; // server_port
        packet.write_u8_varint(1) // next_state
    })?)?; // handshake packet

    conn.write_packet(&Packet::empty(0x00))?; // status request packet

    Ok(println!("motd: {}", conn.read_packet()?.read_string()?)) // status response packet
}
```

[More examples](https://github.com/MeexReay/rust_mc_proto/tree/main/examples) \
[Documentation](https://docs.rs/rust_mc_proto/)

### Contributing

If you would like to contribute to the project, feel free to fork the repository and submit a pull request.

### License
This project is licensed under the WTFPL License