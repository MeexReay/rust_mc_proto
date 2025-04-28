//! Minecraft packet struct

use crate::data::{DataReader, DataWriter};
use crate::ProtocolError;
use std::io::Cursor;

/// Minecraft packet
#[derive(Debug, Clone)]
pub struct Packet {
    id: u8,
    cursor: Cursor<Vec<u8>>,
}

impl Packet {
    /// Create new packet from id and buffer
    pub fn new(id: u8, cursor: Cursor<Vec<u8>>) -> Packet {
        Packet { id, cursor }
    }

    /// Create new packet from raw packet (id + data)
    pub fn from_data(data: &[u8]) -> Result<Packet, ProtocolError> {
        let mut cursor = Cursor::new(data);

        let (packet_id, packet_id_size) = cursor.read_u8_varint_size()?;
        let packet_data =
            DataReader::read_bytes(&mut cursor, data.len() - packet_id_size as usize)?;

        Ok(Packet {
            id: packet_id,
            cursor: Cursor::new(packet_data),
        })
    }

    /// Create new packet from id and bytes in buffer
    pub fn from_bytes(id: u8, data: &[u8]) -> Packet {
        Packet {
            id,
            cursor: Cursor::new(data.to_vec()),
        }
    }

    /// Create new packet with id and empty buffer
    pub fn empty(id: u8) -> Packet {
        Packet {
            id,
            cursor: Cursor::new(Vec::new()),
        }
    }

    /// Build packet with lambda
    pub fn build<F>(id: u8, builder: F) -> Result<Packet, ProtocolError>
    where
        F: FnOnce(&mut Packet) -> Result<(), ProtocolError>,
    {
        let mut packet = Self::empty(id);
        builder(&mut packet)?;
        Ok(packet)
    }

    /// Get packet id
    pub fn id(&self) -> u8 {
        self.id
    }

    /// Set packet id
    pub fn set_id(&mut self, id: u8) {
        self.id = id;
    }

    /// Set packet cursor
    pub fn set_cursor(&mut self, cursor: Cursor<Vec<u8>>) {
        self.cursor = cursor;
    }

    /// Get cursor length
    pub fn len(&self) -> usize {
        self.cursor.get_ref().len() - self.cursor.position() as usize
    }

    /// Is cursor empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get cursor remaining bytes
    pub fn get_bytes(&self) -> &[u8] {
        &self.cursor.get_ref()
    }

    /// Get mutable reference to cursor
    pub fn get_mut(&mut self) -> &mut Cursor<Vec<u8>> {
        &mut self.cursor
    }

    /// Get immutable reference to cursor
    pub fn get_ref(&self) -> &Cursor<Vec<u8>> {
        &self.cursor
    }

    /// Get inner cursor
    pub fn into_inner(self) -> Cursor<Vec<u8>> {
        self.cursor
    }
}

impl Into<Cursor<Vec<u8>>> for Packet {
    fn into(self) -> Cursor<Vec<u8>> {
        self.cursor
    }
}

impl DataReader for Packet {
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError> {
        self.cursor.read_bytes(size)
    }
}

impl DataWriter for Packet {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        self.cursor.write_bytes(bytes)
    }
}
