use bytebuffer::ByteBuffer;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::{
    error::Error,
    fmt,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use uuid::Uuid;

#[cfg(test)]
mod tests;

pub trait Zigzag<T> {
    fn zigzag(&self) -> T;
}
impl Zigzag<u8> for i8 {
    fn zigzag(&self) -> u8 {
        ((self << 1) ^ (self >> 7)) as u8
    }
}
impl Zigzag<i8> for u8 {
    fn zigzag(&self) -> i8 {
        ((self >> 1) as i8) ^ (-((self & 1) as i8))
    }
}
impl Zigzag<u16> for i16 {
    fn zigzag(&self) -> u16 {
        ((self << 1) ^ (self >> 15)) as u16
    }
}
impl Zigzag<i16> for u16 {
    fn zigzag(&self) -> i16 {
        ((self >> 1) as i16) ^ (-((self & 1) as i16))
    }
}
impl Zigzag<u32> for i32 {
    fn zigzag(&self) -> u32 {
        ((self << 1) ^ (self >> 31)) as u32
    }
}
impl Zigzag<i32> for u32 {
    fn zigzag(&self) -> i32 {
        ((self >> 1) as i32) ^ (-((self & 1) as i32))
    }
}
impl Zigzag<u64> for i64 {
    fn zigzag(&self) -> u64 {
        ((self << 1) ^ (self >> 63)) as u64
    }
}
impl Zigzag<i64> for u64 {
    fn zigzag(&self) -> i64 {
        ((self >> 1) as i64) ^ (-((self & 1) as i64))
    }
}
impl Zigzag<u128> for i128 {
    fn zigzag(&self) -> u128 {
        ((self << 1) ^ (self >> 127)) as u128
    }
}
impl Zigzag<i128> for u128 {
    fn zigzag(&self) -> i128 {
        ((self >> 1) as i128) ^ (-((self & 1) as i128))
    }
}
impl Zigzag<usize> for isize {
    fn zigzag(&self) -> usize {
        ((self << 1) ^ (self >> std::mem::size_of::<usize>() - 1)) as usize
    }
}
impl Zigzag<isize> for usize {
    fn zigzag(&self) -> isize {
        ((self >> 1) as isize) ^ (-((self & 1) as isize))
    }
}

/// Minecraft protocol error
#[derive(Debug)]
pub enum ProtocolError {
    AddressParseError,
    DataRanOutError,
    StringParseError,
    StreamConnectError,
    VarIntError,
    ReadError,
    WriteError,
    ZlibError,
    UnsignedShortError,
    CloneError,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An protocol error occured")
    }
}

impl Error for ProtocolError {}

/// Minecraft packet
#[derive(Debug, Clone)]
pub struct Packet {
    id: u8,
    buffer: ByteBuffer,
}

macro_rules! size_varint {
    ($type:ty, $self:expr) => {{
        let mut shift: $type = 0;
        let mut decoded: $type = 0;
        let mut size: $type = 0;

        loop {
            let next = DataBufferReader::read_byte($self).or(Err(ProtocolError::VarIntError))?;
            size += 1;

            if shift >= (std::mem::size_of::<$type>() * 8) as $type {
                return Err(ProtocolError::VarIntError);
            }

            decoded |= ((next & 0b01111111) as $type) << shift;

            if next & 0b10000000 == 0b10000000 {
                shift += 7;
            } else {
                return Ok((decoded, size));
            }
        }
    }};
}

macro_rules! read_varint {
    ($type:ty, $self:expr) => {{
        let mut shift: $type = 0;
        let mut decoded: $type = 0;

        loop {
            let next = DataBufferReader::read_byte($self).or(Err(ProtocolError::VarIntError))?;

            if shift >= (std::mem::size_of::<$type>() * 8) as $type {
                return Err(ProtocolError::VarIntError);
            }

            decoded |= ((next & 0b01111111) as $type) << shift;

            if next & 0b10000000 == 0b10000000 {
                shift += 7;
            } else {
                return Ok(decoded);
            }
        }
    }};
}

macro_rules! write_varint {
    ($type:ty, $self:expr, $value:expr) => {{
        let mut value: $type = $value;

        if value == 0 {
            DataBufferWriter::write_byte($self, 0).or(Err(ProtocolError::VarIntError))
        } else {
            while value >= 0b10000000 {
                let next: u8 = ((value & 0b01111111) as u8) | 0b10000000;
                value >>= 7;

                DataBufferWriter::write_byte($self, next).or(Err(ProtocolError::VarIntError))?;
            }

            DataBufferWriter::write_byte($self, (value & 0b01111111) as u8)
                .or(Err(ProtocolError::VarIntError))
        }
    }};
}

/// Packet data reader trait
pub trait DataBufferReader {
    /// Read bytes
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError>;

    /// Read byte
    fn read_byte(&mut self) -> Result<u8, ProtocolError> {
        Ok(self.read_bytes(1)?[0])
    }
    /// Read [`ByteBuffer`](ByteBuffer)
    fn read_buffer(&mut self, size: usize) -> Result<ByteBuffer, ProtocolError> {
        Ok(ByteBuffer::from_vec(self.read_bytes(size)?))
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
    fn read_u8_varint_size(&mut self) -> Result<(u8, u8), ProtocolError> {
        size_varint!(u8, self)
    }
    /// Read VarInt as u16 with size in bytes (varint, size)
    fn read_u16_varint_size(&mut self) -> Result<(u16, u16), ProtocolError> {
        size_varint!(u16, self)
    }
    /// Read VarInt as u32 with size in bytes (varint, size)
    fn read_u32_varint_size(&mut self) -> Result<(u32, u32), ProtocolError> {
        size_varint!(u32, self)
    }
    /// Read VarInt as u64 with size in bytes (varint, size)
    fn read_u64_varint_size(&mut self) -> Result<(u64, u64), ProtocolError> {
        size_varint!(u64, self)
    }
    /// Read VarInt as u128 with size in bytes (varint, size)
    fn read_u128_varint_size(&mut self) -> Result<(u128, u128), ProtocolError> {
        size_varint!(u128, self)
    }

    /// Read VarInt as isize with size in bytes (varint, size)
    fn read_isize_varint_size(&mut self) -> Result<(isize, isize), ProtocolError> {
        Ok({
            let i = self.read_usize_varint_size()?;
            (i.0.zigzag(), i.1.zigzag())
        })
    }
    /// Read VarInt as i8 with size in bytes (varint, size)
    fn read_i8_varint_size(&mut self) -> Result<(i8, i8), ProtocolError> {
        Ok({
            let i = self.read_u8_varint_size()?;
            (i.0.zigzag(), i.1.zigzag())
        })
    }
    /// Read VarInt as i16 with size in bytes (varint, size)
    fn read_i16_varint_size(&mut self) -> Result<(i16, i16), ProtocolError> {
        Ok({
            let i = self.read_u16_varint_size()?;
            (i.0.zigzag(), i.1.zigzag())
        })
    }
    /// Read VarInt as i32 with size in bytes (varint, size)
    fn read_i32_varint_size(&mut self) -> Result<(i32, i32), ProtocolError> {
        Ok({
            let i = self.read_u32_varint_size()?;
            (i.0.zigzag(), i.1.zigzag())
        })
    }
    /// Read VarInt as i64 with size in bytes (varint, size)
    fn read_i64_varint_size(&mut self) -> Result<(i64, i64), ProtocolError> {
        Ok({
            let i = self.read_u64_varint_size()?;
            (i.0.zigzag(), i.1.zigzag())
        })
    }
    /// Read VarInt as i128 with size in bytes (varint, size)
    fn read_i128_varint_size(&mut self) -> Result<(i128, i128), ProtocolError> {
        Ok({
            let i = self.read_u128_varint_size()?;
            (i.0.zigzag(), i.1.zigzag())
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

/// Packet data writer trait
pub trait DataBufferWriter {
    /// Write bytes
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ProtocolError>;

    /// Write byte
    fn write_byte(&mut self, byte: u8) -> Result<(), ProtocolError> {
        self.write_bytes(&[byte])
    }
    /// Write [`ByteBuffer`](ByteBuffer)
    fn write_buffer(&mut self, buffer: &ByteBuffer) -> Result<(), ProtocolError> {
        self.write_bytes(buffer.as_bytes())
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

impl<R: Read> DataBufferReader for R {
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = vec![0; size];
        match self.read_exact(&mut buf) {
            Ok(_) => Ok(buf),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
}

impl<W: Write> DataBufferWriter for W {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        match self.write_all(bytes) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::WriteError),
        }
    }
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
    pub fn buffer(&mut self) -> &mut ByteBuffer {
        &mut self.buffer
    }

    /// Set packet buffer
    pub fn set_buffer(&mut self, buffer: ByteBuffer) {
        self.buffer = buffer;
    }

    /// Get buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
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

/// Minecraft connection, wrapper for stream with compression
pub struct MinecraftConnection<T: Read + Write> {
    stream: T,
    compression: Arc<AtomicUsize>,
    compression_type: u32,
}

impl MinecraftConnection<TcpStream> {
    /// Connect to Minecraft Server with TcpStream
    pub fn connect(addr: &str) -> Result<MinecraftConnection<TcpStream>, ProtocolError> {
        let addr = match addr.to_socket_addrs() {
            Ok(mut i) => match i.next() {
                Some(i) => i,
                None => return Err(ProtocolError::AddressParseError),
            },
            Err(_) => return Err(ProtocolError::AddressParseError),
        };

        let stream: TcpStream = match TcpStream::connect(&addr) {
            Ok(i) => i,
            Err(_) => return Err(ProtocolError::StreamConnectError),
        };

        Ok(MinecraftConnection {
            stream,
            compression: Arc::new(AtomicUsize::new(usize::MAX)),
            compression_type: 1,
        })
    }

    /// Close TcpStream
    pub fn close(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }

    /// Try clone MinecraftConnection with compression and stream
    pub fn try_clone(&mut self) -> Result<MinecraftConnection<TcpStream>, ProtocolError> {
        match self.stream.try_clone() {
            Ok(stream) => Ok(MinecraftConnection {
                stream,
                compression: self.compression.clone(),
                compression_type: 1,
            }),
            _ => Err(ProtocolError::CloneError),
        }
    }
}

impl<T: Read + Write> DataBufferReader for MinecraftConnection<T> {
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = vec![0; size];
        match self.stream.read_exact(&mut buf) {
            Ok(_) => Ok(buf),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
}

impl<T: Read + Write> DataBufferWriter for MinecraftConnection<T> {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        match self.stream.write_all(bytes) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::WriteError),
        }
    }
}

impl<T: Read + Write> MinecraftConnection<T> {
    /// Create new MinecraftConnection from stream
    pub fn new(stream: T) -> MinecraftConnection<T> {
        MinecraftConnection {
            stream,
            compression: Arc::new(AtomicUsize::new(usize::MAX)),
            compression_type: 1,
        }
    }

    /// Set compression threshold
    pub fn set_compression(&mut self, threshold: Option<usize>) {
        self.compression = Arc::new(AtomicUsize::new(match threshold {
            Some(t) => t,
            None => usize::MAX,
        }));
    }

    /// Get compression threshold
    pub fn compression(&self) -> Option<usize> {
        let threshold = self.compression.load(Ordering::Relaxed);
        if threshold == usize::MAX {
            None
        } else {
            Some(threshold)
        }
    }

    /// Set compression type
    ///
    /// `compression_type` is integer from 0 (none) to 9 (longest)
    /// 1 is fast compression
    /// 6 is normal compression
    pub fn set_compression_type(&mut self, compression_type: u32) {
        self.compression_type = compression_type;
    }

    /// Get compression type
    ///
    /// `compression_type` is integer from 0 (none) to 9 (longest)
    /// 1 is fast compression
    /// 6 is normal compression
    pub fn compression_type(&self) -> u32 {
        self.compression_type
    }

    /// Get mutable reference of stream
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.stream
    }

    /// Get immutable reference of stream
    pub fn get_ref(&self) -> &T {
        &self.stream
    }

    /// Read [`Packet`](Packet) from connection
    pub fn read_packet(&mut self) -> Result<Packet, ProtocolError> {
        read_packet_atomic(
            &mut self.stream,
            self.compression.clone(),
            Ordering::Relaxed,
        )
    }

    /// Write [`Packet`](Packet) to connection
    pub fn write_packet(&mut self, packet: &Packet) -> Result<(), ProtocolError> {
        write_packet_atomic(
            &mut self.stream,
            self.compression.clone(),
            Ordering::Relaxed,
            self.compression_type,
            packet,
        )
    }
}

impl<T: Read + Write + Clone> MinecraftConnection<T> {
    /// Clone MinecraftConnection with compression and stream
    pub fn clone(&mut self) -> MinecraftConnection<T> {
        MinecraftConnection {
            stream: self.stream.clone(),
            compression: self.compression.clone(),
            compression_type: self.compression_type,
        }
    }
}

fn compress_zlib(bytes: &[u8], compression: u32) -> Result<Vec<u8>, ProtocolError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(compression));
    encoder.write_all(bytes).or(Err(ProtocolError::ZlibError))?;
    encoder.finish().or(Err(ProtocolError::ZlibError))
}

fn decompress_zlib(bytes: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut output = Vec::new();
    decoder
        .read_to_end(&mut output)
        .or(Err(ProtocolError::ZlibError))?;
    Ok(output)
}

/// MinecraftConnection shorter alias
pub type MCConn<T> = MinecraftConnection<T>;

/// MinecraftConnection\<TcpStream\> shorter alias
pub type MCConnTcp = MinecraftConnection<TcpStream>;

/// Read [`Packet`](Packet) from stream
///
/// `compression` here is atomic usize
/// usize::MAX means that compression is disabled
///
/// `ordering` is order how to load atomic
pub fn read_packet_atomic<T: Read>(
    stream: &mut T,
    compression: Arc<AtomicUsize>,
    ordering: Ordering,
) -> Result<Packet, ProtocolError> {
    let mut data: Vec<u8>;

    let packet_length = stream.read_usize_varint_size()?;

    let compress_threashold = compression.load(ordering);

    if compress_threashold != usize::MAX {
        let data_length = stream.read_usize_varint_size()?;

        data = stream.read_bytes(packet_length.0 - data_length.1)?;

        if data_length.0 != 0 {
            data = decompress_zlib(&data)?;
        }
    } else {
        data = stream.read_bytes(packet_length.0)?;
    }

    Ok(Packet::from_data(&data)?)
}

/// Write [`Packet`](Packet) to stream
///
/// `compression` here is atomic usize
/// usize::MAX means that compression is disabled
///
/// `ordering` is order how to load atomic
///
/// `compression_type` is integer from 0 (none) to 9 (longest)
/// 1 is fast compression
/// 6 is normal compression
pub fn write_packet_atomic<T: Write>(
    stream: &mut T,
    compression: Arc<AtomicUsize>,
    ordering: Ordering,
    compression_type: u32,
    packet: &Packet,
) -> Result<(), ProtocolError> {
    let mut buf = ByteBuffer::new();

    let mut data_buf = ByteBuffer::new();
    data_buf.write_u8_varint(packet.id)?;
    data_buf.write_buffer(&packet.buffer)?;

    let compress_threshold = compression.load(ordering);

    if compress_threshold != usize::MAX {
        let mut packet_buf = ByteBuffer::new();

        if data_buf.len() >= compress_threshold {
            let compressed_data = compress_zlib(data_buf.as_bytes(), compression_type)?;
            packet_buf.write_usize_varint(data_buf.len())?;
            packet_buf
                .write_all(&compressed_data)
                .or(Err(ProtocolError::WriteError))?;
        } else {
            packet_buf.write_usize_varint(0)?;
            packet_buf.write_buffer(&data_buf)?;
        }

        buf.write_usize_varint(packet_buf.len())?;
        buf.write_buffer(&packet_buf)?;
    } else {
        buf.write_usize_varint(data_buf.len())?;
        buf.write_buffer(&data_buf)?;
    }

    stream.write_buffer(&buf)?;

    Ok(())
}
