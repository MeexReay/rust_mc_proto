#[cfg(test)]
mod tests;

pub mod data_buffer;
pub mod packet;
pub mod zigzag;

pub use crate::{
    data_buffer::{DataBufferReader, DataBufferWriter},
    packet::Packet,
};

use bytebuffer::ByteBuffer;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::{
    error::Error, fmt, io::{Read, Write}, net::{TcpStream, ToSocketAddrs}, sync::atomic::AtomicBool, usize
};

#[cfg(feature = "atomic_clone")]
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

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
    ConnectionClosedError
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An protocol error occured")
    }
}

impl Error for ProtocolError {}

/// Minecraft connection, wrapper for stream with compression
pub struct MinecraftConnection<T: Read + Write> {
    stream: T,
    #[cfg(feature = "atomic_clone")]
    compression: Arc<AtomicUsize>,
    #[cfg(not(feature = "atomic_clone"))]
    compression: Option<usize>,
    compression_type: u32,
    #[cfg(feature = "atomic_clone")]
    is_alive: Arc<AtomicBool>,
    #[cfg(not(feature = "atomic_clone"))]
    is_alive: bool,
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
            #[cfg(feature = "atomic_clone")]
            compression: Arc::new(AtomicUsize::new(usize::MAX)),
            #[cfg(not(feature = "atomic_clone"))]
            compression: None,
            #[cfg(feature = "atomic_clone")]
            is_alive: Arc::new(AtomicBool::new(true)),
            #[cfg(not(feature = "atomic_clone"))]
            is_alive: true,
            compression_type: 1,
        })
    }

    pub fn set_nonblocking(&mut self, state: bool) {
        self.stream.set_nonblocking(state).unwrap();
    }

    /// Close TcpStream
    #[cfg(not(feature = "atomic_clone"))]
    pub fn close(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
        self.is_alive = false;
    }

    /// Close TcpStream
    #[cfg(feature = "atomic_clone")]
    pub fn close(&self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
        self.is_alive.store(false, Ordering::Relaxed);
    }

    /// Try clone MinecraftConnection with compression and stream
    pub fn try_clone(&self) -> Result<MinecraftConnection<TcpStream>, ProtocolError> {
        match self.stream.try_clone() {
            Ok(stream) => Ok(MinecraftConnection {
                stream,
                is_alive: self.is_alive.clone(),
                compression: self.compression.clone(),
                compression_type: self.compression_type,
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
            #[cfg(feature = "atomic_clone")]
            compression: Arc::new(AtomicUsize::new(usize::MAX)),
            #[cfg(not(feature = "atomic_clone"))]
            compression: None,
            #[cfg(feature = "atomic_clone")]
            is_alive: Arc::new(AtomicBool::new(true)),
            #[cfg(not(feature = "atomic_clone"))]
            is_alive: true,
            compression_type: 1,
        }
    }

    /// Set alive state
    #[cfg(not(feature = "atomic_clone"))]
    pub fn set_alive(&mut self, state: bool) {
        self.is_alive = state;
    }

    /// Set alive state
    #[cfg(feature = "atomic_clone")]
    pub fn set_alive(&self, state: bool) {
        self.is_alive.store(state, Ordering::Relaxed);
    }

    /// Is connection alive
    #[cfg(not(feature = "atomic_clone"))]
    pub fn set_alive(&self) -> bool {
        self.is_alive
    }

    /// Is connection alive
    #[cfg(feature = "atomic_clone")]
    pub fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::Relaxed)
    }

    /// Set compression threshold
    pub fn set_compression(&mut self, threshold: Option<usize>) {
        #[cfg(feature = "atomic_clone")]
        self.compression.store(
            match threshold {
                Some(t) => t,
                None => usize::MAX,
            },
            Ordering::Relaxed,
        );
        #[cfg(not(feature = "atomic_clone"))]
        {
            self.compression = threshold;
        }
    }

    /// Get compression threshold
    pub fn compression(&self) -> Option<usize> {
        #[cfg(feature = "atomic_clone")]
        {
            let threshold = self.compression.load(Ordering::Relaxed);
            if threshold == usize::MAX {
                return None
            } else {
                return Some(threshold)
            }
        }
        #[cfg(not(feature = "atomic_clone"))]
        {
            self.compression
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
        if !self.is_alive() {
            return Err(ProtocolError::ConnectionClosedError);
        }

        #[cfg(feature = "atomic_clone")]
        {
            return match read_packet_atomic(
                            &mut self.stream,
                            self.compression.clone(),
                            Ordering::Relaxed,
                        ) {
                Err(ProtocolError::ConnectionClosedError) => {
                    self.set_alive(false);
                    Err(ProtocolError::ConnectionClosedError)
                },
                i => i
            };
        }

        #[cfg(not(feature = "atomic_clone"))]
        match read_packet(&mut self.stream, self.compression) {
            Err(ProtocolError::ConnectionClosedError) => {
                self.set_alive(false);
                Err(ProtocolError::ConnectionClosedError)
            },
            i => i
        }
    }

    /// Write [`Packet`](Packet) to connection
    pub fn write_packet(&mut self, packet: &Packet) -> Result<(), ProtocolError> {
        if !self.is_alive() {
            return Err(ProtocolError::ConnectionClosedError);
        }
        
        #[cfg(feature = "atomic_clone")]
        {
            return write_packet_atomic(
                &mut self.stream,
                self.compression.clone(),
                Ordering::Relaxed,
                self.compression_type,
                packet,
            )
        }

        #[cfg(not(feature = "atomic_clone"))]
        {
            write_packet(&mut self.stream, self.compression, self.compression_type, packet)
        }
    }
}

impl<T: Read + Write + Clone> MinecraftConnection<T> {
    /// Clone MinecraftConnection with compression and stream
    pub fn clone(&mut self) -> MinecraftConnection<T> {
        MinecraftConnection {
            stream: self.stream.clone(),
            compression: self.compression.clone(),
            is_alive: self.is_alive.clone(),
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
pub fn read_packet<T: Read>(
    stream: &mut T,
    compression: Option<usize>
) -> Result<Packet, ProtocolError> {
    let mut data: Vec<u8>;

    let packet_length = stream.read_usize_varint_size()?;

    if compression.is_some() {
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
/// `compression` here is usize
/// usize::MAX means that compression is disabled
///
/// `ordering` is order how to load atomic
///
/// `compression_type` is integer from 0 (none) to 9 (longest)
/// 1 is fast compression
/// 6 is normal compression
pub fn write_packet<T: Write>(
    stream: &mut T,
    compression: Option<usize>,
    compression_type: u32,
    packet: &Packet,
) -> Result<(), ProtocolError> {
    let mut buf = ByteBuffer::new();

    let mut data_buf = ByteBuffer::new();
    data_buf.write_u8_varint(packet.id())?;
    data_buf.write_buffer(packet.buffer())?;

    if let Some(compression) = compression {
        let mut packet_buf = ByteBuffer::new();

        if data_buf.len() >= compression {
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

/// Read [`Packet`](Packet) from stream
///
/// `compression` here is atomic usize
/// usize::MAX means that compression is disabled
///
/// `ordering` is order how to load atomic
#[cfg(feature = "atomic_clone")]
pub fn read_packet_atomic<T: Read>(
    stream: &mut T,
    compression: Arc<AtomicUsize>,
    ordering: Ordering,
) -> Result<Packet, ProtocolError> {
    read_packet(stream, match compression.load(ordering) {
        usize::MAX => None,
        i => Some(i),
    })
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
#[cfg(feature = "atomic_clone")]
pub fn write_packet_atomic<T: Write>(
    stream: &mut T,
    compression: Arc<AtomicUsize>,
    ordering: Ordering,
    compression_type: u32,
    packet: &Packet,
) -> Result<(), ProtocolError> {
    write_packet(stream, match compression.load(ordering) {
        usize::MAX => None,
        i => Some(i),
    }, compression_type, packet)
}
