use std::{net::TcpListener, sync::{atomic::AtomicBool, atomic::Ordering}, thread};
use std::sync::mpsc::channel;
use rust_mc_proto::{DataBufferReader, DataBufferWriter, MCConn, MCConnTcp, MinecraftConnection, Packet, ProtocolError};

const LONG_TEXT: &str = "some_long_text_wow_123123123123123123123123";

fn main() {
    let (tx, rx) = channel::<()>();

    let server_tx = tx.clone();
    thread::spawn(move || {
        let listener = TcpListener::bind("localhost:44447").unwrap();

        server_tx.send(()).unwrap();

        for stream in listener.incoming() {
            let mut stream = MCConnTcp::new(stream.unwrap());
            stream.set_compression(2);

            let packet = stream.read_packet().unwrap();
            stream.write_packet(&packet).unwrap();
        }
    });

    rx.recv().unwrap();

    let mut conn = MCConnTcp::connect("localhost:44447").unwrap();
    conn.set_compression(2);
    
    let mut packet = Packet::empty(0x12);
    packet.write_string(LONG_TEXT).unwrap();
    conn.write_packet(&packet).unwrap();

    let mut packet = conn.read_packet().unwrap();
    if packet.id == 0x12 && packet.read_string().unwrap() == LONG_TEXT {
        println!("success");
    }
}
