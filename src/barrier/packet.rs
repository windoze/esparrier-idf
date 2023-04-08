use std::{collections::HashMap, io::Write};

use super::{PacketError, PacketWriter};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Packet {
    QueryInfo,
    DeviceInfo {
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        _dummy: u16,
        mx: u16, // x position of the mouse on the secondary screen
        my: u16, // y position of the mouse on the secondary screen
    },
    InfoAck,
    KeepAlive,
    ResetOptions,
    ClientNoOp,
    SetDeviceOptions(HashMap<String, u32>),
    ErrorUnknownDevice,
    GrabClipboard {
        id: u8,
        seq_num: u32,
    },
    SetClipboard {
        id: u8,
        seq_num: u32,
        mark: u8,
        data: Option<Vec<u8>>,
    },
    CursorEnter {
        x: u16,
        y: u16,
        seq_num: u32,
        mask: u16,
    },
    MouseUp {
        id: i8,
    },
    MouseDown {
        id: i8,
    },
    KeyUp {
        id: u16,
        mask: u16,
        button: u16,
    },
    KeyDown {
        id: u16,
        mask: u16,
        button: u16,
    },
    KeyRepeat {
        id: u16,
        mask: u16,
        button: u16,
        count: u16,
    },
    MouseWheel {
        x_delta: i16,
        y_delta: i16,
    },
    CursorLeave,
    MouseMoveAbs {
        x: u16,
        y: u16,
    },
    Unknown(String),
}

impl Packet {
    pub fn write_wire<W: Write + Send + Unpin>(self, mut out: W) -> Result<(), PacketError> {
        match self {
            Packet::QueryInfo => {
                out.write_str("QINF")?;
                Ok(())
            }
            Packet::DeviceInfo {
                x,
                y,
                w,
                h,
                _dummy,
                mx,
                my,
            } => {
                out.write_u32(2 * 7 + 4)?;
                out.write_all(b"DINF")?;
                out.write_u16(x)?;
                out.write_u16(y)?;
                out.write_u16(w)?;
                out.write_u16(h)?;
                out.write_u16(0)?;
                out.write_u16(mx)?;
                out.write_u16(my)?;
                Ok(())
            }
            Packet::ClientNoOp => {
                out.write_str("CNOP")?;
                Ok(())
            }
            Packet::Unknown(_) => {
                unimplemented!()
            }
            Packet::InfoAck => {
                out.write_str("CIAK")?;
                Ok(())
            }
            Packet::KeepAlive => {
                out.write_str("CALV")?;
                Ok(())
            }
            Packet::ErrorUnknownDevice => {
                out.write_str("EUNK")?;
                Ok(())
            }
            Packet::MouseMoveAbs { x, y } => {
                out.write_u32(4 + 2 + 2)?;
                out.write_all(b"DMMV")?;
                out.write_u16(x)?;
                out.write_u16(y)?;
                Ok(())
            }
            _ => {
                unimplemented!("{:?} not yet implemented", self)
            }
        }
    }
}
