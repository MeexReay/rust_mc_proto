use crate::{
    data::varint::{read_varint, size_varint},
    zigzag::Zigzag,
    ProtocolError,
};
use std::io::Read;
use uuid::Uuid;

/// Packet data reader trait
pub trait DataReader {
    /// Read bytes
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError>;

    /// Read byte
    fn read_byte(&mut self) -> Result<u8, ProtocolError> {
        Ok(self.read_bytes(1)?[0])
    }
    /// Read String
    fn read_string(&mut self) -> Result<String, ProtocolError> {
        let size = self.read_usize_varint()?;
        match String::from_utf8(self.read_bytes(size)?) {
            Ok(i) => Ok(i),
            Err(_) => Err(ProtocolError::StringParseError),
        }
    }
    /// Read Unsigned Short as u16
    fn read_unsigned_short(&mut self) -> Result<u16, ProtocolError> {
        match self.read_bytes(2)?.try_into() {
            Ok(i) => Ok(u16::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    /// Read Boolean
    fn read_boolean(&mut self) -> Result<bool, ProtocolError> {
        Ok(self.read_byte()? == 0x01)
    }
    /// Read Short as i16
    fn read_short(&mut self) -> Result<i16, ProtocolError> {
        match self.read_bytes(2)?.try_into() {
            Ok(i) => Ok(i16::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    /// Read Long as i64
    fn read_long(&mut self) -> Result<i64, ProtocolError> {
        match self.read_bytes(8)?.try_into() {
            Ok(i) => Ok(i64::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    /// Read Float as f32
    fn read_float(&mut self) -> Result<f32, ProtocolError> {
        match self.read_bytes(4)?.try_into() {
            Ok(i) => Ok(f32::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    /// Read Double as f64
    fn read_double(&mut self) -> Result<f64, ProtocolError> {
        match self.read_bytes(8)?.try_into() {
            Ok(i) => Ok(f64::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    /// Read Int as i32
    fn read_int(&mut self) -> Result<i32, ProtocolError> {
        match self.read_bytes(4)?.try_into() {
            Ok(i) => Ok(i32::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    /// Read UUID
    fn read_uuid(&mut self) -> Result<Uuid, ProtocolError> {
        match self.read_bytes(16)?.try_into() {
            Ok(i) => Ok(Uuid::from_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }

    /// Read VarInt as usize with size in bytes (varint, size)
    fn read_usize_varint_size(&mut self) -> Result<(usize, usize), ProtocolError> {
        size_varint!(usize, self)
    }
    /// Read VarInt as u8 with size in bytes (varint, size)
    fn read_u8_varint_size(&mut self) -> Result<(u8, usize), ProtocolError> {
        size_varint!(u8, self)
    }
    /// Read VarInt as u16 with size in bytes (varint, size)
    fn read_u16_varint_size(&mut self) -> Result<(u16, usize), ProtocolError> {
        size_varint!(u16, self)
    }
    /// Read VarInt as u32 with size in bytes (varint, size)
    fn read_u32_varint_size(&mut self) -> Result<(u32, usize), ProtocolError> {
        size_varint!(u32, self)
    }
    /// Read VarInt as u64 with size in bytes (varint, size)
    fn read_u64_varint_size(&mut self) -> Result<(u64, usize), ProtocolError> {
        size_varint!(u64, self)
    }
    /// Read VarInt as u128 with size in bytes (varint, size)
    fn read_u128_varint_size(&mut self) -> Result<(u128, usize), ProtocolError> {
        size_varint!(u128, self)
    }

    /// Read VarInt as isize with size in bytes (varint, size)
    fn read_isize_varint_size(&mut self) -> Result<(isize, usize), ProtocolError> {
        Ok({
            let i = self.read_usize_varint_size()?;
            (i.0.zigzag(), i.1)
        })
    }
    /// Read VarInt as i8 with size in bytes (varint, size)
    fn read_i8_varint_size(&mut self) -> Result<(i8, usize), ProtocolError> {
        Ok({
            let i = self.read_u8_varint_size()?;
            (i.0.zigzag(), i.1)
        })
    }
    /// Read VarInt as i16 with size in bytes (varint, size)
    fn read_i16_varint_size(&mut self) -> Result<(i16, usize), ProtocolError> {
        Ok({
            let i = self.read_u16_varint_size()?;
            (i.0.zigzag(), i.1)
        })
    }
    /// Read VarInt as i32 with size in bytes (varint, size)
    fn read_i32_varint_size(&mut self) -> Result<(i32, usize), ProtocolError> {
        Ok({
            let i = self.read_u32_varint_size()?;
            (i.0.zigzag(), i.1)
        })
    }
    /// Read VarInt as i64 with size in bytes (varint, size)
    fn read_i64_varint_size(&mut self) -> Result<(i64, usize), ProtocolError> {
        Ok({
            let i = self.read_u64_varint_size()?;
            (i.0.zigzag(), i.1)
        })
    }
    /// Read VarInt as i128 with size in bytes (varint, size)
    fn read_i128_varint_size(&mut self) -> Result<(i128, usize), ProtocolError> {
        Ok({
            let i = self.read_u128_varint_size()?;
            (i.0.zigzag(), i.1)
        })
    }

    /// Read VarInt as usize
    fn read_usize_varint(&mut self) -> Result<usize, ProtocolError> {
        read_varint!(usize, self)
    }
    /// Read VarInt as u8
    fn read_u8_varint(&mut self) -> Result<u8, ProtocolError> {
        read_varint!(u8, self)
    }
    /// Read VarInt as u16
    fn read_u16_varint(&mut self) -> Result<u16, ProtocolError> {
        read_varint!(u16, self)
    }
    /// Read VarInt as u32
    fn read_u32_varint(&mut self) -> Result<u32, ProtocolError> {
        read_varint!(u32, self)
    }
    /// Read VarInt as u64
    fn read_u64_varint(&mut self) -> Result<u64, ProtocolError> {
        read_varint!(u64, self)
    }
    /// Read VarInt as u128
    fn read_u128_varint(&mut self) -> Result<u128, ProtocolError> {
        read_varint!(u128, self)
    }

    /// Read VarInt as isize
    fn read_isize_varint(&mut self) -> Result<isize, ProtocolError> {
        Ok(self.read_usize_varint()?.zigzag())
    }
    /// Read VarInt as i8
    fn read_i8_varint(&mut self) -> Result<i8, ProtocolError> {
        Ok(self.read_u8_varint()?.zigzag())
    }
    /// Read VarInt as i16
    fn read_i16_varint(&mut self) -> Result<i16, ProtocolError> {
        Ok(self.read_u16_varint()?.zigzag())
    }
    /// Read VarInt as i32
    fn read_i32_varint(&mut self) -> Result<i32, ProtocolError> {
        Ok(self.read_u32_varint()?.zigzag())
    }
    /// Read VarInt as i64
    fn read_i64_varint(&mut self) -> Result<i64, ProtocolError> {
        Ok(self.read_u64_varint()?.zigzag())
    }
    /// Read VarInt as i128
    fn read_i128_varint(&mut self) -> Result<i128, ProtocolError> {
        Ok(self.read_u128_varint()?.zigzag())
    }
}

impl<R: Read> DataReader for R {
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = vec![0; size];
        match self.read(&mut buf) {
            Ok(i) => if i == size {
                Ok(buf)
            } else if i == 0 {
                Err(ProtocolError::ConnectionClosedError)
            } else {
                buf.truncate(i);
                buf.append(&mut self.read_bytes(size-i)?);
                Ok(buf)
            },
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
}
