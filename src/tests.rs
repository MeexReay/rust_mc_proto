use uuid::Uuid;

use super::*;
use std::{net::TcpListener, thread};

#[test]
fn test_data_transfer() -> Result<(), ProtocolError> {

    thread::spawn(move || -> Result<(), ProtocolError> {
        let listener =
            TcpListener::bind("localhost:44447").or(Err(ProtocolError::StreamConnectError))?;

        for stream in listener.incoming() {
            let mut stream = MCConnTcp::new(stream.or(Err(ProtocolError::StreamConnectError))?);

            stream.set_compression(Some(5));

            let mut packet = stream.read_packet()?;

            stream.write_packet(&Packet::build(packet.id(), |pack| {
                pack.write_boolean(packet.read_boolean()?)?;
                pack.write_byte(packet.read_byte()?)?;
                pack.write_bytes(&packet.read_bytes(10)?)?;
                pack.write_double(packet.read_double()?)?;
                pack.write_float(packet.read_float()?)?;
                pack.write_i128_varint(packet.read_i128_varint()?)?;
                pack.write_u128_varint(packet.read_u128_varint()?)?;
                pack.write_int(packet.read_int()?)?;
                pack.write_long(packet.read_long()?)?;
                pack.write_short(packet.read_short()?)?;
                pack.write_uuid(&packet.read_uuid()?)?;
                pack.write_string(&packet.read_string()?)?;
                Ok(())
            })?)?;

            stream.set_compression(None);

            let mut packet = stream.read_packet()?;

            stream.write_packet(&Packet::build(packet.id(), |pack| {
                pack.write_boolean(packet.read_boolean()?)?;
                pack.write_byte(packet.read_byte()?)?;
                pack.write_bytes(&packet.read_bytes(10)?)?;
                pack.write_double(packet.read_double()?)?;
                pack.write_float(packet.read_float()?)?;
                pack.write_i128_varint(packet.read_i128_varint()?)?;
                pack.write_u128_varint(packet.read_u128_varint()?)?;
                pack.write_int(packet.read_int()?)?;
                pack.write_long(packet.read_long()?)?;
                pack.write_short(packet.read_short()?)?;
                pack.write_uuid(&packet.read_uuid()?)?;
                pack.write_string(&packet.read_string()?)?;
                Ok(())
            })?)?;
        }

        Ok(())
    });
    
    let conn = MCConnTcp::connect("localhost:44447");

    while let Err(_) = conn {}

    let mut conn = conn?;

    conn.set_compression(Some(5));

    conn.write_packet(&Packet::build(0xfe, |pack| {
        pack.write_boolean(true)?;
        pack.write_byte(0x12)?;
        pack.write_bytes(&vec![0x01, 0x56, 0x47, 0x48, 0xf5, 0xc2, 0x45, 0x98, 0xde, 0x99])?;
        pack.write_double(123456789.123456789f64)?;
        pack.write_float(789456.44422f32)?;
        pack.write_i128_varint(468927513325566)?;
        pack.write_u128_varint(99859652365236523)?;
        pack.write_int(77861346i32)?;
        pack.write_long(789465123545678946i64)?;
        pack.write_short(1233i16)?;
        pack.write_uuid(&Uuid::try_parse("550e8400-e29b-41d4-a716-446655440000").map_err(|_| ProtocolError::CloneError)?)?;
        pack.write_string("&packet.read_string()?")?;
        Ok(())
    })?)?;

    let mut packet = conn.read_packet()?;

    assert_eq!(packet.read_boolean()?, true);
    assert_eq!(packet.read_byte()?, 0x12);
    assert_eq!(packet.read_bytes(10)?, vec![0x01, 0x56, 0x47, 0x48, 0xf5, 0xc2, 0x45, 0x98, 0xde, 0x99]);
    assert_eq!(packet.read_double()?, 123456789.123456789f64);
    assert_eq!(packet.read_float()?, 789456.44422f32);
    assert_eq!(packet.read_i128_varint()?, 468927513325566);
    assert_eq!(packet.read_u128_varint()?, 99859652365236523);
    assert_eq!(packet.read_int()?, 77861346i32);
    assert_eq!(packet.read_long()?, 789465123545678946i64);
    assert_eq!(packet.read_short()?, 1233i16);
    assert_eq!(packet.read_uuid()?, Uuid::try_parse("550e8400-e29b-41d4-a716-446655440000").map_err(|_| ProtocolError::CloneError)?);
    assert_eq!(packet.read_string()?, "&packet.read_string()?");

    conn.set_compression(None);

    conn.write_packet(&Packet::build(0xfe, |pack| {
        pack.write_boolean(true)?;
        pack.write_byte(0x12)?;
        pack.write_bytes(&vec![0x01, 0x56, 0x47, 0x48, 0xf5, 0xc2, 0x45, 0x98, 0xde, 0x99])?;
        pack.write_double(123456789.123456789f64)?;
        pack.write_float(789456.44422f32)?;
        pack.write_i128_varint(468927513325566)?;
        pack.write_u128_varint(99859652365236523)?;
        pack.write_int(77861346i32)?;
        pack.write_long(789465123545678946i64)?;
        pack.write_short(1233i16)?;
        pack.write_uuid(&Uuid::try_parse("550e8400-e29b-41d4-a716-446655440000").map_err(|_| ProtocolError::CloneError)?)?;
        pack.write_string("&packet.read_string()?")?;
        Ok(())
    })?)?;

    let mut packet = conn.read_packet()?;

    assert_eq!(packet.read_boolean()?, true);
    assert_eq!(packet.read_byte()?, 0x12);
    assert_eq!(packet.read_bytes(10)?, vec![0x01, 0x56, 0x47, 0x48, 0xf5, 0xc2, 0x45, 0x98, 0xde, 0x99]);
    assert_eq!(packet.read_double()?, 123456789.123456789f64);
    assert_eq!(packet.read_float()?, 789456.44422f32);
    assert_eq!(packet.read_i128_varint()?, 468927513325566);
    assert_eq!(packet.read_u128_varint()?, 99859652365236523);
    assert_eq!(packet.read_int()?, 77861346i32);
    assert_eq!(packet.read_long()?, 789465123545678946i64);
    assert_eq!(packet.read_short()?, 1233i16);
    assert_eq!(packet.read_uuid()?, Uuid::try_parse("550e8400-e29b-41d4-a716-446655440000").map_err(|_| ProtocolError::CloneError)?);
    assert_eq!(packet.read_string()?, "&packet.read_string()?");

    Ok(())
}

#[test]
fn test_compression() -> Result<(), ProtocolError> {
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
