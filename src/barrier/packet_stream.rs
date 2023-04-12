use std::{cmp::min, net::TcpStream};

use log::debug;

use crate::barrier::clipboard::Clipboard;

use super::{Packet, PacketError, PacketReader, PacketWriter};

pub struct PacketStream<S: PacketReader + PacketWriter> {
    stream: S,
}

impl<S: PacketReader + PacketWriter> PacketStream<S> {
    pub fn new(stream: S) -> Self {
        Self { stream }
    }

    pub fn read(&mut self) -> Result<Packet, PacketError> {
        let size = self.stream.read_packet_size()?;
        if size < 4 {
            let mut vec = Vec::new();
            self.stream.read_to_end(&mut vec)?;
            return Err(PacketError::PacketTooSmall);
        }
        let code: [u8; 4] = self.stream.read_bytes_fixed()?;

        let packet = match code.as_ref() {
            b"QINF" => Packet::QueryInfo,
            b"CIAK" => Packet::InfoAck,
            b"CALV" => Packet::KeepAlive,
            // We don't really have any option to set and reset
            // b"CROP" => Packet::ResetOptions,
            b"DSOP" => {
                let num_items = self.stream.read_u32()?;
                let num_opts = num_items / 2;
                let mut heartbeat: u32 = 5000;
                // Currently only HBRT(Heartbeat interval) is supported
                for _ in 0..num_opts {
                    let opt: [u8; 4] = self.stream.read_bytes_fixed()?;
                    let val = self.stream.read_u32()?;
                    if &opt == b"HBRT" {
                        heartbeat = val;
                    }
                }
                Packet::SetDeviceOptions { heartbeat }
            }
            b"EUNK" => Packet::ErrorUnknownDevice,
            b"DMMV" => {
                let x = self.stream.read_u16()?;
                let y = self.stream.read_u16()?;
                Packet::MouseMoveAbs { x, y }
            }
            b"CINN" => {
                let x = self.stream.read_u16()?;
                let y = self.stream.read_u16()?;
                let seq_num = self.stream.read_u32()?;
                let mask = self.stream.read_u16()?;
                Packet::CursorEnter {
                    x,
                    y,
                    seq_num,
                    mask,
                }
            }
            b"COUT" => Packet::CursorLeave,
            b"CCLP" => {
                let id = self.stream.read_u8()?;
                let seq_num = self.stream.read_u32()?;
                Packet::GrabClipboard { id, seq_num }
            }
            b"DCLP" => {
                let id = self.stream.read_u8()?;
                let seq_num = self.stream.read_u32()?;
                let mark = self.stream.read_u8()?;

                // mark 1 is a length string in ASCII
                // mark 2 is the actual data
                // mark 3 is an empty chunk
                let data = if mark == 2 {
                    let mut c = Clipboard::default();
                    let mut sz = self.stream.read_u32()? as usize;
                    let mut buf: [u8; 16] = [0; 16];
                    while sz > 0 {
                        let l = self.stream.read(&mut buf[0..min(16, sz)])?;
                        c.feed(&buf[0..l]);
                        sz -= l;
                    }
                    debug!(
                        "ClipboardStash State: {:?}, NumFormats: {}, CurrentIndex: {}",
                        c.state, c.num_format, c.current_index
                    );
                    c.into_data()
                } else {
                    self.stream.consume_bytes()?;
                    None
                };

                Packet::SetClipboard {
                    id,
                    seq_num,
                    mark,
                    data,
                }
            }
            b"DMUP" => {
                let id = self.stream.read_i8()?;
                Packet::MouseUp { id }
            }
            b"DMDN" => {
                let id = self.stream.read_i8()?;
                Packet::MouseDown { id }
            }
            b"DKUP" => {
                let id = self.stream.read_u16()?;
                let mask = self.stream.read_u16()?;
                let button = self.stream.read_u16()?;
                Packet::KeyUp { id, mask, button }
            }
            b"DKDN" => {
                let id = self.stream.read_u16()?;
                let mask = self.stream.read_u16()?;
                let button = self.stream.read_u16()?;
                Packet::KeyDown { id, mask, button }
            }
            b"DKRP" => {
                let id = self.stream.read_u16()?;
                let mask = self.stream.read_u16()?;
                let count = self.stream.read_u16()?;
                let button = self.stream.read_u16()?;
                Packet::KeyRepeat {
                    id,
                    mask,
                    button,
                    count,
                }
            }
            b"DMWM" => {
                let x_delta = self.stream.read_i16()?;
                let y_delta = self.stream.read_i16()?;
                Packet::MouseWheel { x_delta, y_delta }
            }
            _ => {
                let mut s = size - 4;
                while s > 0 {
                    let _: [u8; 1] = self.stream.read_bytes_fixed()?;
                    s -= 1;
                }
                Packet::Unknown(code)
            }
        };

        Ok(packet)
    }

    pub fn write(&mut self, packet: Packet) -> Result<(), PacketError> {
        packet.write_wire(&mut self.stream)?;
        Ok(())
    }
}

pub trait ReadTimeout {
    fn set_read_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result<(), std::io::Error>;
}

impl ReadTimeout for PacketStream<TcpStream> {
    fn set_read_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result<(), std::io::Error> {
        self.stream.set_read_timeout(timeout)
    }
}