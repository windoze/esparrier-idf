use std::cmp::min;

use super::{PacketError, PacketReader};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ClipboardFormat {
    Text = 0,
    Html = 1,
    Bitmap = 2,
}

const MAX_SIZE: usize = 1024;

pub fn parse_clipboard<T: PacketReader>(stream: &mut T) -> Result<Option<Vec<u8>>, PacketError> {
    let _sz = stream.read_u32()?;
    let mut ret = Vec::new();
    let num_formats = stream.read_u32()?;

    for _ in 0..num_formats {
        let format = stream.read_u32()?;
        let mut length = stream.read_u32()? as usize;

        let format = match format {
            0 => ClipboardFormat::Text,
            1 => ClipboardFormat::Html,
            2 => ClipboardFormat::Bitmap,
            _ => Err(PacketError::FormatError)?,
        };

        if format == ClipboardFormat::Text {
            while length > 0 {
                let mut buf = [0; 16];
                let read_length = min(length, 16);
                stream.read_exact(&mut buf[0..read_length])?;
                length -= read_length;
                if ret.len() < MAX_SIZE {
                    ret.extend_from_slice(&buf[0..min(read_length, 16)]);
                }
            }
        } else {
            stream.discard_exact(length)?;
        }
    }
    if ret.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ret))
    }
}
