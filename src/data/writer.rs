use crate::{data::varint::write_varint, zigzag::Zigzag, ProtocolError};
use std::io::Write;
use uuid::Uuid;

/// Packet data writer trait
pub trait DataWriter {
    /// Write bytes
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ProtocolError>;

    /// Write byte
    fn write_byte(&mut self, byte: u8) -> Result<(), ProtocolError> {
        self.write_bytes(&[byte])
    }
    /// Write String
    fn write_string(&mut self, val: &str) -> Result<(), ProtocolError> {
        let bytes = val.as_bytes();
        self.write_usize_varint(bytes.len())?;
        self.write_bytes(bytes)
    }
    /// Write UUID
    fn write_uuid(&mut self, val: &Uuid) -> Result<(), ProtocolError> {
        self.write_bytes(val.as_bytes())
    }
    /// Write Unsigned Short as u16
    fn write_unsigned_short(&mut self, val: u16) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    /// Write Boolean
    fn write_boolean(&mut self, val: bool) -> Result<(), ProtocolError> {
        match self.write_byte(if val { 0x01 } else { 0x00 }) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    /// Write Short as i16
    fn write_short(&mut self, val: i16) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    /// Write Long as i64
    fn write_long(&mut self, val: i64) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    /// Write Float as f32
    fn write_float(&mut self, val: f32) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    /// Write Double as f64
    fn write_double(&mut self, val: f64) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    /// Write Int as i32
    fn write_int(&mut self, val: i32) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }

    /// Write VarInt as usize
    fn write_usize_varint(&mut self, val: usize) -> Result<(), ProtocolError> {
        write_varint!(usize, self, val)
    }
    /// Write VarInt as u8
    fn write_u8_varint(&mut self, val: u8) -> Result<(), ProtocolError> {
        write_varint!(u8, self, val)
    }
    /// Write VarInt as u16
    fn write_u16_varint(&mut self, val: u16) -> Result<(), ProtocolError> {
        write_varint!(u16, self, val)
    }
    /// Write VarInt as u32
    fn write_u32_varint(&mut self, val: u32) -> Result<(), ProtocolError> {
        write_varint!(u32, self, val)
    }
    /// Write VarInt as u64
    fn write_u64_varint(&mut self, val: u64) -> Result<(), ProtocolError> {
        write_varint!(u64, self, val)
    }
    /// Write VarInt as u128
    fn write_u128_varint(&mut self, val: u128) -> Result<(), ProtocolError> {
        write_varint!(u128, self, val)
    }

    /// Write VarInt as isize
    fn write_isize_varint(&mut self, val: isize) -> Result<(), ProtocolError> {
        self.write_usize_varint(val.zigzag())
    }
    /// Write VarInt as i8
    fn write_i8_varint(&mut self, val: i8) -> Result<(), ProtocolError> {
        self.write_u8_varint(val.zigzag())
    }
    /// Write VarInt as i16
    fn write_i16_varint(&mut self, val: i16) -> Result<(), ProtocolError> {
        self.write_u16_varint(val.zigzag())
    }
    /// Write VarInt as i32
    fn write_i32_varint(&mut self, val: i32) -> Result<(), ProtocolError> {
        self.write_u32_varint(val.zigzag())
    }
    /// Write VarInt as i64
    fn write_i64_varint(&mut self, val: i64) -> Result<(), ProtocolError> {
        self.write_u64_varint(val.zigzag())
    }
    /// Write VarInt as i128
    fn write_i128_varint(&mut self, val: i128) -> Result<(), ProtocolError> {
        self.write_u128_varint(val.zigzag())
    }
}

impl<W: Write> DataWriter for W {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        match self.write_all(bytes) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::WriteError),
        }
    }
}