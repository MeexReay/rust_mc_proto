use std::io::{Write, Read};
use std::net::{TcpStream, ToSocketAddrs};

use flate2::bufread::ZlibDecoder;
use flate2::{Compress, Compression, Decompress, FlushCompress, Status, FlushDecompress};
use bytebuffer::ByteBuffer;

pub trait Zigzag<T> { fn zigzag(&self) -> T; }
impl Zigzag<u8> for i8 { fn zigzag(&self) -> u8 { ((self << 1) ^ (self >> 7)) as u8 } }
impl Zigzag<i8> for u8 { fn zigzag(&self) -> i8 { ((self >> 1) as i8) ^ (-((self & 1) as i8)) } }
impl Zigzag<u16> for i16 { fn zigzag(&self) -> u16 { ((self << 1) ^ (self >> 15)) as u16 } }
impl Zigzag<i16> for u16 { fn zigzag(&self) -> i16 { ((self >> 1) as i16) ^ (-((self & 1) as i16)) } }
impl Zigzag<u32> for i32 { fn zigzag(&self) -> u32 { ((self << 1) ^ (self >> 31)) as u32 } }
impl Zigzag<i32> for u32 { fn zigzag(&self) -> i32 { ((self >> 1) as i32) ^ (-((self & 1) as i32)) } }
impl Zigzag<u64> for i64 { fn zigzag(&self) -> u64 { ((self << 1) ^ (self >> 63)) as u64 } }
impl Zigzag<i64> for u64 { fn zigzag(&self) -> i64 { ((self >> 1) as i64) ^ (-((self & 1) as i64)) } }
impl Zigzag<u128> for i128 { fn zigzag(&self) -> u128 { ((self << 1) ^ (self >> 127)) as u128 } }
impl Zigzag<i128> for u128 { fn zigzag(&self) -> i128 { ((self >> 1) as i128) ^ (-((self & 1) as i128)) } }
impl Zigzag<usize> for isize { fn zigzag(&self) -> usize { ((self << 1) ^ (self >> std::mem::size_of::<usize>()-1)) as usize } }
impl Zigzag<isize> for usize { fn zigzag(&self) -> isize { ((self >> 1) as isize) ^ (-((self & 1) as isize)) } }

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
    UnsignedShortError
}

#[derive(Debug)]
pub struct Packet {
    pub id: u8,
    pub buffer: ByteBuffer
}


// copied from varint-rs (im sorry T-T)

macro_rules! size_varint {
    ($type: ty, $self: expr) => {
        {
            let mut shift: $type = 0;
            let mut decoded: $type = 0;
            let mut next: u8 = 0;

            loop {
                match DataBufferReader::read_byte($self) {
                    Ok(value) => next = value,
                    Err(error) => Err(error)?
                }

                decoded |= ((next & 0b01111111) as $type) << shift;

                if next & 0b10000000 == 0b10000000 {
                    shift += 7;
                } else {
                    return Ok((decoded, shift / 7))
                }
            }
        }
    };
}

macro_rules! read_varint {
    ($type: ty, $self: expr) => {
        {
            let mut shift: $type = 0;
            let mut decoded: $type = 0;
            let mut next: u8 = 0;

            loop {
                match DataBufferReader::read_byte($self) {
                    Ok(value) => next = value,
                    Err(error) => Err(error)?
                }

                decoded |= ((next & 0b01111111) as $type) << shift;

                if next & 0b10000000 == 0b10000000 {
                    shift += 7;
                } else {
                    return Ok(decoded)
                }
            }
        }
    };
}

macro_rules! write_varint {
    ($type: ty, $self: expr, $value: expr) => {
        {
            let mut value: $type = $value;
    
            if value == 0 {
                DataBufferWriter::write_byte($self, 0)
            } else {
                while value >= 0b10000000 {
                    let next: u8 = ((value & 0b01111111) as u8) | 0b10000000;
                    value >>= 7;
    
                    match DataBufferWriter::write_byte($self, next) {
                        Err(error) => Err(error)?,
                        Ok(_) => ()
                    }
                }
    
                DataBufferWriter::write_byte($self, (value & 0b01111111) as u8)
            }
        }
    };
}

pub trait DataBufferReader {
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError>;
    fn read_byte(&mut self) -> Result<u8, ProtocolError>;

    fn read_string(&mut self) -> Result<String, ProtocolError> {
        let size = self.read_usize_varint()?;
        match String::from_utf8(self.read_bytes(size)?) {
            Ok(i) => Ok(i),
            Err(_) => Err(ProtocolError::StringParseError)
        }
    }
    fn read_unsigned_short(&mut self) -> Result<u16, ProtocolError> {
        match self.read_bytes(2)?.try_into() {
            Ok(i) => Ok(u16::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    fn read_boolean(&mut self) -> Result<bool, ProtocolError> {
        Ok(self.read_byte()? == 0x01)
    }
    fn read_short(&mut self) -> Result<i16, ProtocolError> {
        match self.read_bytes(2)?.try_into() {
            Ok(i) => Ok(i16::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    fn read_long(&mut self) -> Result<i64, ProtocolError> {
        match self.read_bytes(8)?.try_into() {
            Ok(i) => Ok(i64::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    fn read_float(&mut self) -> Result<f32, ProtocolError> {
        match self.read_bytes(4)?.try_into() {
            Ok(i) => Ok(f32::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    fn read_double(&mut self) -> Result<f64, ProtocolError> {
        match self.read_bytes(8)?.try_into() {
            Ok(i) => Ok(f64::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    fn read_int(&mut self) -> Result<i32, ProtocolError> {
        match self.read_bytes(4)?.try_into() {
            Ok(i) => Ok(i32::from_be_bytes(i)),
            Err(_) => Err(ProtocolError::ReadError),
        }
    }

    fn read_usize_varint_size(&mut self) -> Result<(usize, usize), ProtocolError> { size_varint!(usize, self) }

    fn read_usize_varint(&mut self) -> Result<usize, ProtocolError> { read_varint!(usize, self) }
    fn read_u8_varint(&mut self) -> Result<u8, ProtocolError> { read_varint!(u8, self) }
    fn read_u16_varint(&mut self) -> Result<u16, ProtocolError> { read_varint!(u16, self) }
    fn read_u32_varint(&mut self) -> Result<u32, ProtocolError> { read_varint!(u32, self) }
    fn read_u64_varint(&mut self) -> Result<u64, ProtocolError> { read_varint!(u64, self) }
    fn read_u128_varint(&mut self) -> Result<u128, ProtocolError> { read_varint!(u128, self) }

    fn read_isize_varint(&mut self) -> Result<isize, ProtocolError> { Ok(self.read_usize_varint()?.zigzag()) }
    fn read_i8_varint(&mut self) -> Result<i8, ProtocolError> { Ok(self.read_u8_varint()?.zigzag()) }
    fn read_i16_varint(&mut self) -> Result<i16, ProtocolError> { Ok(self.read_u16_varint()?.zigzag()) }
    fn read_i32_varint(&mut self) -> Result<i32, ProtocolError> { Ok(self.read_u32_varint()?.zigzag()) }
    fn read_i64_varint(&mut self) -> Result<i64, ProtocolError> { Ok(self.read_u64_varint()?.zigzag()) }
    fn read_i128_varint(&mut self) -> Result<i128, ProtocolError> { Ok(self.read_u128_varint()?.zigzag()) }
}


pub trait DataBufferWriter {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ProtocolError>;
    fn write_byte(&mut self, byte: u8) -> Result<(), ProtocolError>;

    fn write_string(&mut self, val: &str) -> Result<(), ProtocolError> {
        let bytes = val.as_bytes();

        self.write_usize_varint(bytes.len())?;
        self.write_bytes(bytes)
    }
    fn write_unsigned_short(&mut self, val: u16) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    fn write_boolean(&mut self, val: bool) -> Result<(), ProtocolError> {
        match self.write_byte(if val { 0x01 } else { 0x00 }) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    fn write_short(&mut self, val: i16) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    fn write_long(&mut self, val: i64) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    fn write_float(&mut self, val: f32) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    fn write_double(&mut self, val: f64) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }
    fn write_int(&mut self, val: i32) -> Result<(), ProtocolError> {
        match self.write_bytes(&val.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }

    fn write_usize_varint(&mut self, val: usize) -> Result<(), ProtocolError> { write_varint!(usize, self, val) }
    fn write_u8_varint(&mut self, val: u8) -> Result<(), ProtocolError> { write_varint!(u8, self, val) }
    fn write_u16_varint(&mut self, val: u16) -> Result<(), ProtocolError> { write_varint!(u16, self, val) }
    fn write_u32_varint(&mut self, val: u32) -> Result<(), ProtocolError> { write_varint!(u32, self, val) }
    fn write_u64_varint(&mut self, val: u64) -> Result<(), ProtocolError> { write_varint!(u64, self, val) }
    fn write_u128_varint(&mut self, val: u128) -> Result<(), ProtocolError> { write_varint!(u128, self, val) }

    fn write_isize_varint(&mut self, val: isize) -> Result<(), ProtocolError> { self.write_usize_varint(val.zigzag()) }
    fn write_i8_varint(&mut self, val: i8) -> Result<(), ProtocolError> { self.write_u8_varint(val.zigzag()) }
    fn write_i16_varint(&mut self, val: i16) -> Result<(), ProtocolError> { self.write_u16_varint(val.zigzag()) }
    fn write_i32_varint(&mut self, val: i32) -> Result<(), ProtocolError> { self.write_u32_varint(val.zigzag()) }
    fn write_i64_varint(&mut self, val: i64) -> Result<(), ProtocolError> { self.write_u64_varint(val.zigzag()) }
    fn write_i128_varint(&mut self, val: i128) -> Result<(), ProtocolError> { self.write_u128_varint(val.zigzag()) }
}

impl<R: Read> DataBufferReader for R {
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = vec![0; size];
        match self.read(&mut buf) {
            Ok(i) => {
                if i < size {
                    Err(ProtocolError::ReadError)
                } else {
                    Ok(buf)
                }
            },
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    fn read_byte(&mut self) -> Result<u8, ProtocolError> {
        match self.read_bytes(1) {
            Ok(i) => Ok(i[0]),
            Err(i) => Err(i),
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

    fn write_byte(&mut self, byte: u8) -> Result<(), ProtocolError> {
        self.write_bytes(&[byte])
    }
}

impl Packet {
    pub fn new(id: u8, buffer: ByteBuffer) -> Packet {
        Packet {
            id,
            buffer
        }
    }

    pub fn from_bytes(id: u8, data: &[u8]) -> Packet {
        Packet {
            id,
            buffer: ByteBuffer::from_bytes(data)
        }
    }

    pub fn empty(id: u8) -> Packet {
        Packet {
            id,
            buffer: ByteBuffer::new()
        }
    }
}

impl DataBufferReader for Packet {
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = vec![0; size];
        match self.buffer.read(&mut buf) {
            Ok(i) => {
                if i < size {
                    Err(ProtocolError::ReadError)
                } else {
                    Ok(buf)
                }
            },
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
    fn read_byte(&mut self) -> Result<u8, ProtocolError> {
        match self.read_bytes(1) {
            Ok(i) => Ok(i[0]),
            Err(i) => Err(i),
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

    fn write_byte(&mut self, byte: u8) -> Result<(), ProtocolError> {
        self.write_bytes(&[byte])
    }
}

pub struct MinecraftConnection<T: Read + Write> {
    pub stream: T,
    compress: bool,
    compress_threashold: usize
}

impl MinecraftConnection<TcpStream> {
    pub fn connect(addr: &str) -> Result<MinecraftConnection<TcpStream>, ProtocolError> {
        let addr = match addr.to_socket_addrs() {
            Ok(mut i) => { match i.next() {
                Some(i) => { i },
                None => { return Err(ProtocolError::AddressParseError) },
            } },
            Err(_) => { return Err(ProtocolError::AddressParseError) },
        };
    
        let stream: TcpStream = match TcpStream::connect(&addr) {
            Ok(i) => i,
            Err(_) => { return Err(ProtocolError::StreamConnectError) },
        };
    
        Ok(MinecraftConnection {
            stream,
            compress: false,
            compress_threashold: 0
        })
    }

    pub fn close(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}

impl<T: Read + Write> DataBufferReader for MinecraftConnection<T> {
    fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError> {
        let mut buf = vec![0; size];
        match self.stream.read(&mut buf) {
            Ok(i) => {
                if i < size {
                    Err(ProtocolError::ReadError)
                } else {
                    Ok(buf)
                }
            },
            Err(err) => Err(ProtocolError::ReadError),
        }
    }
    fn read_byte(&mut self) -> Result<u8, ProtocolError> {
        match self.read_bytes(1) {
            Ok(i) => Ok(i[0]),
            Err(i) => Err(i),
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

    fn write_byte(&mut self, byte: u8) -> Result<(), ProtocolError> {
        self.write_bytes(&[byte])
    }
}

impl<T: Read + Write> MinecraftConnection<T> {
    pub fn new(stream: T) -> MinecraftConnection<T> {
        MinecraftConnection {
            stream,
            compress: false,
            compress_threashold: 0
        }
    }

    pub fn set_compression(&mut self, threashold: usize) {
        self.compress = true;
        self.compress_threashold = threashold;
    }
    pub fn read_packet(&mut self) -> Result<Packet, ProtocolError> {
        if !self.compress {
            let length = self.read_usize_varint()?;

            let packet_id = self.read_u8_varint()?;
            let mut data: Vec<u8> = vec![0; length - 1];
            match self.stream.read_exact(&mut data) {
                Ok(i) => i,
                Err(_) => { return Err(ProtocolError::ReadError) },
            };

            return Ok(Packet::from_bytes(packet_id, &data))
        }
        let packet_length = self.read_usize_varint_size()?;
        let data_length = self.read_usize_varint_size()?;

        if data_length.0 == 0 {
            let packet_id = self.read_u8_varint()?;
            let mut data: Vec<u8> = vec![0; packet_length.0 - 2];
            match self.stream.read_exact(&mut data) {
                Ok(i) => i,
                Err(_) => { return Err(ProtocolError::ReadError) },
            };

            return Ok(Packet::from_bytes(packet_id, &data))
        }

        let mut data: Vec<u8> = vec![0; packet_length.0 - packet_length.1 - data_length.1];
        match self.stream.read_exact(&mut data) {
            Ok(i) => i,
            Err(_) => { return Err(ProtocolError::ReadError) },
        };

        let mut data_buf = ByteBuffer::from_vec(decompress_zlib(&data, packet_length.0 - packet_length.1 - data_length.1)?);

        let packet_id = match data_buf.read_u8_varint() {
            Ok(i) => i,
            Err(_) => { return Err(ProtocolError::VarIntError) },
        };
        let mut data: Vec<u8> = vec![0; packet_length.0 - packet_length.1 - data_length.1 - 1];
        match data_buf.read_exact(&mut data) {
            Ok(i) => i,
            Err(_) => { return Err(ProtocolError::ReadError) },
        };

        Ok(Packet::from_bytes(packet_id, &data))
    }

    pub fn write_packet(&mut self, packet: &Packet) -> Result<(), ProtocolError> {
        let mut buf = ByteBuffer::new();

        if !self.compress {
            match buf.write_usize_varint(packet.buffer.len() + 1) {
                Ok(_) => {},
                Err(_) => { return Err(ProtocolError::WriteError) },
            };
            match buf.write_u8_varint(packet.id) {
                Ok(_) => {},
                Err(_) => { return Err(ProtocolError::WriteError) },
            };
            match buf.write_all(packet.buffer.as_bytes()) {
                Ok(_) => {},
                Err(_) => { return Err(ProtocolError::WriteError) },
            };
        } else {
            let mut pack = ByteBuffer::new();

            if packet.buffer.len() < self.compress_threashold {
                match pack.write_usize_varint(0) {
                    Ok(_) => {},
                    Err(_) => { return Err(ProtocolError::WriteError) },
                };
                match pack.write_u8_varint(packet.id) {
                    Ok(_) => {},
                    Err(_) => { return Err(ProtocolError::WriteError) },
                };
                match pack.write_all(packet.buffer.as_bytes()) {
                    Ok(_) => {},
                    Err(_) => { return Err(ProtocolError::WriteError) },
                };
            } else {
                let mut data = ByteBuffer::new();

                match data.write_u8_varint(packet.id) {
                    Ok(_) => {},
                    Err(_) => { return Err(ProtocolError::WriteError) },
                };
                match data.write_all(packet.buffer.as_bytes()) {
                    Ok(_) => {},
                    Err(_) => { return Err(ProtocolError::WriteError) },
                };

                let data = compress_zlib(data.as_bytes())?;

                match pack.write_usize_varint(packet.buffer.len() + 1) {
                    Ok(_) => {},
                    Err(_) => { return Err(ProtocolError::WriteError) },
                };
                match pack.write_all(&data) {
                    Ok(_) => {},
                    Err(_) => { return Err(ProtocolError::WriteError) },
                };
            }

            match buf.write_usize_varint(pack.len()) {
                Ok(_) => {},
                Err(_) => { return Err(ProtocolError::WriteError) },
            };
            match buf.write_all(pack.as_bytes()) {
                Ok(_) => {},
                Err(_) => { return Err(ProtocolError::WriteError) },
            };
        }

        match self.stream.write_all(buf.as_bytes()) {
            Ok(_) => {},
            Err(_) => { return Err(ProtocolError::WriteError) },
        };

        Ok(())
    }
}

fn compress_zlib(bytes: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let mut compresser = Compress::new(Compression::best(), true);
    let mut output: Vec<u8> = Vec::new();
    match compresser.compress_vec(&bytes, &mut output, FlushCompress::Finish) {
        Ok(i) => { 
            match i {
                Status::BufError => Err(ProtocolError::ZlibError),
                _ => Ok(output),
            }
        },
        Err(_) => Err(ProtocolError::ZlibError)
    }
}

fn decompress_zlib(bytes: &[u8], packet_length: usize) -> Result<Vec<u8>, ProtocolError> {
    let mut compresser = Decompress::new(true);
    let mut output: Vec<u8> = Vec::with_capacity(packet_length);
    match compresser.decompress_vec(&bytes, &mut output, FlushDecompress::Sync) {
        Ok(i) => { 
            match i {
                Status::BufError => Err(ProtocolError::ZlibError),
                _ => Ok(output),
            }
        },
        Err(_) => Err(ProtocolError::ZlibError)
    }
}


pub type MCConn<T> = MinecraftConnection<T>;
pub type MCConnTcp = MinecraftConnection<TcpStream>;
