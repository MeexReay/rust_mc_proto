//! `DataReader` and `DataWriter` traits for reading and writing primitive types in the Minecraft protocol

pub mod reader;
pub mod varint;
pub mod writer;

pub use reader::*;
pub use writer::*;
