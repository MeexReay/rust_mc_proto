use super::*;
use std::{net::TcpListener, thread};

#[test]
fn test_compression_server_client() -> Result<(), ProtocolError> {
    fn test(first_text: &str) -> Result<bool, ProtocolError> {
        let Ok(mut conn) = MCConnTcp::connect("localhost:44447") else {
            return test(first_text);
        };
        conn.set_compression(Some(5));

        let mut packet = Packet::empty(0x12);
        packet.write_string(first_text)?;
        conn.write_packet(&packet)?;

        println!("[c -> s] sent packet with text \"{}\"", first_text);

        let mut packet = conn.read_packet()?;
        let text = packet.read_string()?;

        println!("[c <- s] read packet with text \"{}\"", text);

        Ok(packet.id() == 0x12 && text == first_text)
    }

    thread::spawn(move || -> Result<(), ProtocolError> {
        let listener =
            TcpListener::bind("localhost:44447").or(Err(ProtocolError::StreamConnectError))?;

        for stream in listener.incoming() {
            let mut stream = MCConnTcp::new(stream.or(Err(ProtocolError::StreamConnectError))?);
            stream.set_compression(Some(5));

            let mut packet = stream.read_packet()?;
            let text = packet.read_string()?;
            println!("[s <- c] read packet with text \"{}\"", text);
            stream.write_packet(&packet)?;
            println!("[s -> c] sent packet with text \"{}\"", text);
        }

        Ok(())
    });

    assert!(test("12bcvf756iuyu,.,.")? && test("a")?);

    Ok(())
}

#[test]
fn test_compression_atomic_bytebuffer() -> Result<(), ProtocolError> {
    let mut conn = MCConn::new(ByteBuffer::new());
    conn.set_compression(Some(5));

    let mut packet_1 = Packet::empty(0x12);
    packet_1.write_bytes(b"1234567890qwertyuiopasdfghjklzxcvbnm")?;
    conn.write_packet(&packet_1)?;

    let mut packet_2 = conn.read_packet()?;

    assert_eq!(
        packet_2.read_bytes(36)?,
        b"1234567890qwertyuiopasdfghjklzxcvbnm"
    );

    Ok(())
}
