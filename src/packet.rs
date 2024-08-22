use crate::data_buffer::{DataBufferReader, DataBufferWriter};
use crate::ProtocolError;
use bytebuffer::ByteBuffer;
use std::io::{Read, Write};

/// Minecraft packet
#[derive(Debug, Clone)]
pub struct Packet {
    id: u8,
    buffer: ByteBuffer,
}

impl Packet {
    /// Create new packet from id and buffer
    pub fn new(id: u8, buffer: ByteBuffer) -> Packet {
        Packet { id, buffer }
    }

    /// Create new packet from packet data
    pub fn from_data(data: &[u8]) -> Result<Packet, ProtocolError> {
        let mut buf = ByteBuffer::from_bytes(data);

        let (packet_id, packet_id_size) = buf.read_u8_varint_size()?;
        let packet_data =
            DataBufferReader::read_bytes(&mut buf, data.len() - packet_id_size as usize)?;

        Ok(Packet {
            id: packet_id,
            buffer: ByteBuffer::from_bytes(&packet_data),
        })
    }

    /// Create new packet from id and bytes in buffer
    pub fn from_bytes(id: u8, data: &[u8]) -> Packet {
        Packet {
            id,
            buffer: ByteBuffer::from_bytes(data),
        }
    }

    /// Create new packet with id and empty buffer
    pub fn empty(id: u8) -> Packet {
        Packet {
            id,
            buffer: ByteBuffer::new(),
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

    /// Get mutable reference of buffer
    pub fn buffer_mut(&mut self) -> &mut ByteBuffer {
        &mut self.buffer
    }

    /// Get immutable reference of buffer
    pub fn buffer(&self) -> &ByteBuffer {
        &self.buffer
    }

    /// Set packet buffer
    pub fn set_buffer(&mut self, buffer: ByteBuffer) {
        self.buffer = buffer;
    }

    /// Get buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Is buffer empty
    pub fn is_empty(&self) -> bool {
        self.buffer.len() == 0
    }

    /// Get buffer bytes
    pub fn get_bytes(&self) -> Vec<u8> {
        self.buffer.as_bytes().to_vec()
    }
}

impl DataBufferReader for Packet {
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = vec![0; size];
        match self.buffer.read_exact(&mut buf) {
            Ok(_) => Ok(buf),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
}

impl DataBufferWriter for Packet {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        match self.buffer.write_all(bytes) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::WriteError),
        }
    }
}
