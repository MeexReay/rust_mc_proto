use std::io::{Write, Read};
use std::net::{TcpStream, ToSocketAddrs};
use bytebuffer::ByteBuffer;
use varint_rs::{VarintWriter, VarintReader};
use flate2::{Compress, Compression, Decompress, FlushCompress, Status, FlushDecompress};

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

impl VarintWriter for Packet {
    type Error = ProtocolError;
 
    fn write(&mut self, byte: u8) -> Result<(), Self::Error> {
        match Write::write(&mut self.buffer, &[byte]) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::WriteError)
        }
    }
}

impl VarintReader for Packet {
    type Error = ProtocolError;
 
    fn read(&mut self) -> Result<u8, Self::Error> {
        let mut buf = vec![0; 1];

        match Read::read(&mut self.buffer, &mut buf) {
            Ok(i) => {
                if i == 1 { 
                    Ok(buf[0]) 
                } else { 
                    Err(ProtocolError::ReadError) 
                }
            },
            Err(_) => Err(ProtocolError::ReadError),
        }
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

    pub fn write_string(&mut self, s: String) -> Result<(), ProtocolError> {
        let bytes = s.as_bytes();

        self.write_usize_varint(bytes.len())?;
        self.write_bytes(bytes)
    }

    pub fn read_string(&mut self) -> Result<String, ProtocolError> {
        let size = self.read_usize_varint()?;
        match String::from_utf8(self.read_bytes(size)?) {
            Ok(i) => Ok(i),
            Err(_) => Err(ProtocolError::StringParseError)
        }
    }

    pub fn write_unsigned_short(&mut self, short: u16) -> Result<(), ProtocolError> {
        match self.buffer.write_all(&short.to_be_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::UnsignedShortError),
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        match Write::write_all(&mut self.buffer, bytes) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::WriteError),
        }
    }

    pub fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, ProtocolError> {
        let mut bytes = vec![0; size];
        match Read::read(&mut self.buffer, &mut bytes) {
            Ok(_) => Ok(bytes),
            Err(_) => Err(ProtocolError::WriteError)
        }
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

            Ok(Packet::from_bytes(packet_id, &data))
        } else {
            let packet_length = self.read_usize_varint()?;
            let data_length = self.read_usize_varint()?;

            if data_length == 0 {
                let mut data: Vec<u8> = vec![0; packet_length - 1];
                match self.stream.read_exact(&mut data) {
                    Ok(i) => i,
                    Err(_) => { return Err(ProtocolError::ReadError) },
                };

                let mut data_buf = ByteBuffer::from_vec(decompress_zlib(&data)?);

                let packet_id = match data_buf.read_u8_varint() {
                    Ok(i) => i,
                    Err(_) => { return Err(ProtocolError::VarIntError) },
                };
                let mut data: Vec<u8> = vec![0; data_length - 1];
                match data_buf.read_exact(&mut data) {
                    Ok(i) => i,
                    Err(_) => { return Err(ProtocolError::ReadError) },
                };

                Ok(Packet::from_bytes(packet_id, &data))
            } else {
                let packet_id = self.read_u8_varint()?;
                let mut data: Vec<u8> = vec![0; data_length - 1];
                match self.stream.read_exact(&mut data) {
                    Ok(i) => i,
                    Err(_) => { return Err(ProtocolError::ReadError) },
                };

                Ok(Packet::from_bytes(packet_id, &data))
            }
        }
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

impl<T: Read + Write> VarintWriter for MinecraftConnection<T> {
    type Error = ProtocolError;

    fn write(&mut self, byte: u8) -> Result<(), Self::Error> {
        match Write::write(&mut self.stream, &[byte]) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProtocolError::WriteError),
        }
    }
}

impl<T: Read + Write> VarintReader for MinecraftConnection<T> {
    type Error = ProtocolError;

    fn read(&mut self) -> Result<u8, Self::Error> {
        let mut buf = vec![0; 1];

        match Read::read(&mut self.stream, &mut buf) {
            Ok(i) => {
                if i == 1 { 
                    Ok(buf[0]) 
                } else { 
                    Err(ProtocolError::ReadError) 
                }
            },
            Err(_) => Err(ProtocolError::ReadError),
        }
    }
}

fn compress_zlib(bytes: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let mut compresser = Compress::new(Compression::best(), true);
    let mut output: Vec<u8> = Vec::new();
    match compresser.compress_vec(&bytes, &mut output, FlushCompress::Finish) {
        Ok(i) => { 
            match i {
                Status::Ok => Ok(output),
                _ => Err(ProtocolError::ZlibError)
            }
        },
        Err(_) => Err(ProtocolError::ZlibError)
    }
}

fn decompress_zlib(bytes: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let mut compresser = Decompress::new(true);
    let mut output: Vec<u8> = Vec::new();
    match compresser.decompress_vec(&bytes, &mut output, FlushDecompress::Finish) {
        Ok(i) => { 
            match i {
                Status::Ok => Ok(output),
                _ => Err(ProtocolError::ZlibError)
            }
        },
        Err(_) => Err(ProtocolError::ZlibError)
    }
}

pub type MCConn<T> = MinecraftConnection<T>;