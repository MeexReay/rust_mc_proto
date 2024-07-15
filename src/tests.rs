use super::*;
use std::{net::TcpListener, thread};

#[test]
fn test_compression_server_client() -> Result<(), ProtocolError> {
    fn test(first_text: &str) -> Result<bool, ProtocolError> {
        let Ok(mut conn) = MCConnTcp::connect("localhost:44447") else { return test(first_text) };
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
        let listener = TcpListener::bind("localhost:44447").or(Err(ProtocolError::StreamConnectError))?;

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
    let packet_1 = Packet::build(0x12, |p| {
        p.write_bytes(b"1234567890qwertyuiopasdfghjklzxcvbnm")
    })?;

    let compression = Arc::new(AtomicUsize::new(5));

    let mut buffer = ByteBuffer::new();

    write_packet_atomic(&mut buffer, compression.clone(), Ordering::Acquire, &packet_1)?;

    buffer.set_rpos(0);
    buffer.set_wpos(0);

    let mut packet_2 = read_packet_atomic(&mut buffer, compression.clone(), Ordering::Acquire)?;

    assert_eq!(packet_2.read_bytes(36)?, b"1234567890qwertyuiopasdfghjklzxcvbnm");

    Ok(())
}