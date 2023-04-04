use std::io::{Read, Write};

use super::PacketError;

pub trait PacketReader: Read + Send + Unpin {
    fn consume_bytes(&mut self) -> Result<(), PacketError> {
        let mut len = self.read_u32()? as usize;

        let mut buf = vec![0; 16];
        while len > 0 {
            let to_read = std::cmp::min(len, buf.len());
            self.read_exact(&mut buf[..to_read])?;
            len -= to_read;
        }
        Ok(())
    }

    fn read_packet_size(&mut self) -> Result<u32, PacketError> {
        self.read_u32()
    }

    fn read_bytes_fixed<const N: usize>(&mut self) -> Result<[u8; N], PacketError> {
        let mut res = [0; N];
        self.read_exact(&mut res)?;
        Ok(res)
    }

    fn read_bytes(&mut self) -> Result<Vec<u8>, PacketError> {
        let mut buf = vec![];

        let len = self.read_u32()?;

        let mut chunk =
            self.take(u64::try_from(len).map_err(|_| PacketError::InsufficientDataError)?);
        chunk.read_to_end(&mut buf)?;

        Ok(buf)
    }

    fn read_str_lit(&mut self, lit: &str) -> Result<(), PacketError> {
        let mut buf = vec![];

        let mut chunk =
            self.take(u64::try_from(lit.len()).map_err(|_| PacketError::InsufficientDataError)?);
        chunk.read_to_end(&mut buf)?;

        if buf == lit.as_bytes() {
            Ok(())
        } else {
            Err(PacketError::FormatError)
        }
    }

    fn read_i8(&mut self) -> Result<i8, PacketError> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0] as i8)
    }

    fn read_u8(&mut self) -> Result<u8, PacketError> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_i16(&mut self) -> Result<i16, PacketError> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    fn read_u16(&mut self) -> Result<u16, PacketError> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_u32(&mut self) -> Result<u32, PacketError> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }
}

impl<T: Read + Send + Unpin> PacketReader for T {}

pub trait PacketWriter: Write + Send + Unpin {
    // fn write_u32(&mut self, data: u32) -> Result<(), PacketError> {
    //     Ok(WriteBytesExt::write_u32::<BigEndian>(self, data)?)
    // }
    //
    // fn write_u16(&mut self, data: u16) -> Result<(), PacketError> {
    //     Ok(WriteBytesExt::write_u16::<BigEndian>(self, data)?)
    // }

    fn write_str(&mut self, data: &str) -> Result<(), PacketError> {
        self.write_u32(data.len() as u32)?;
        self.write_all(data.as_bytes())?;
        Ok(())
    }

    fn write_u16(&mut self, data: u16) -> Result<(), PacketError> {
        Ok(self.write_all(&data.to_be_bytes())?)
    }

    fn write_u32(&mut self, data: u32) -> Result<(), PacketError> {
        Ok(self.write_all(&data.to_be_bytes())?)
    }
}

impl<T: Write + Send + Unpin> PacketWriter for T {}
