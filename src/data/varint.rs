macro_rules! size_varint {
    ($type:ty, $self:expr) => {{
        let mut shift: $type = 0;
        let mut decoded: $type = 0;
        let mut size: usize = 0;

        loop {
            let next = DataReader::read_byte($self)?;
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
            let next = DataReader::read_byte($self)?;

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
            DataWriter::write_byte($self, 0)
        } else {
            while value >= 0b10000000 {
                let next: u8 = ((value & 0b01111111) as u8) | 0b10000000;
                value >>= 7;

                DataWriter::write_byte($self, next)?;
            }

            DataWriter::write_byte($self, (value & 0b01111111) as u8)
        }
    }};
}

pub(crate) use read_varint;
pub(crate) use size_varint;
pub(crate) use write_varint;
