use std::net::TcpStream;

use log::{debug, warn};

use super::{Packet, PacketError, PacketReader, PacketWriter, clipboard::parse_clipboard};

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
        let mut chunk = super::take::Take::new(&mut self.stream, size as u64);
        let code: [u8; 4] = chunk.read_bytes_fixed()?;
        if size > 2048 {
            warn!("Packet too large, discarding {} bytes", size);
            chunk.discard_all()?;
            return Ok(Packet::Unknown(code));
        }

        let packet = match code.as_ref() {
            b"QINF" => Packet::QueryInfo,
            b"CIAK" => Packet::InfoAck,
            b"CALV" => Packet::KeepAlive,
            // We don't really have any option to set and reset
            // b"CROP" => Packet::ResetOptions,
            b"DSOP" => {
                let num_items = chunk.read_u32()?;
                let num_opts = num_items / 2;
                let mut heartbeat: u32 = 5000;
                // Currently only HBRT(Heartbeat interval) is supported
                for _ in 0..num_opts {
                    let opt: [u8; 4] = chunk.read_bytes_fixed()?;
                    let val = chunk.read_u32()?;
                    if &opt == b"HBRT" {
                        heartbeat = val;
                    }
                }
                Packet::SetDeviceOptions { heartbeat }
            }
            b"EUNK" => Packet::ErrorUnknownDevice,
            b"DMMV" => {
                let x = chunk.read_u16()?;
                let y = chunk.read_u16()?;
                Packet::MouseMoveAbs { x, y }
            }
            b"CINN" => {
                let x = chunk.read_u16()?;
                let y = chunk.read_u16()?;
                let seq_num = chunk.read_u32()?;
                let mask = chunk.read_u16()?;
                Packet::CursorEnter {
                    x,
                    y,
                    seq_num,
                    mask,
                }
            }
            b"COUT" => Packet::CursorLeave,
            b"CCLP" => {
                let id = chunk.read_u8()?;
                let seq_num = chunk.read_u32()?;
                Packet::GrabClipboard { id, seq_num }
            }
            b"DCLP" => {
                let id = chunk.read_u8()?;
                let seq_num = chunk.read_u32()?;
                let mark = chunk.read_u8()?;
                debug!("DCLP chunk, size: {}, mark: {}", size, mark);
                let mut data = None;

                // mark 1 is the total length string in ASCII
                // mark 2 is the actual data
                // mark 3 is an empty chunk
                if mark==2 {
                    data = parse_clipboard(&mut chunk).unwrap_or_default();
                }

                Packet::SetClipboard {
                    id,
                    seq_num,
                    mark,
                    data,
                }
            }

            b"DMUP" => {
                let id = chunk.read_i8()?;
                Packet::MouseUp { id }
            }
            b"DMDN" => {
                let id = chunk.read_i8()?;
                Packet::MouseDown { id }
            }
            b"DKUP" => {
                let id = chunk.read_u16()?;
                let mask = chunk.read_u16()?;
                let button = chunk.read_u16()?;
                Packet::KeyUp { id, mask, button }
            }
            b"DKDN" => {
                let id = chunk.read_u16()?;
                let mask = chunk.read_u16()?;
                let button = chunk.read_u16()?;
                Packet::KeyDown { id, mask, button }
            }
            b"DKRP" => {
                let id = chunk.read_u16()?;
                let mask = chunk.read_u16()?;
                let count = chunk.read_u16()?;
                let button = chunk.read_u16()?;
                Packet::KeyRepeat {
                    id,
                    mask,
                    button,
                    count,
                }
            }
            b"DMWM" => {
                let x_delta = chunk.read_i16()?;
                let y_delta = chunk.read_i16()?;
                Packet::MouseWheel { x_delta, y_delta }
            }
            _ => Packet::Unknown(code),
        };

        // Discard the rest of the packet
        while chunk.limit() > 0 {
            warn!(
                "Discarding rest of packet, code: {:?}, size: {}",
                code,
                chunk.limit()
            );
            chunk.discard_all()?;
        }

        Ok(packet)
    }

    pub fn write(&mut self, packet: Packet) -> Result<(), PacketError> {
        packet.write_wire(&mut self.stream)?;
        Ok(())
    }
}

pub trait ReadTimeout {
    fn set_read_timeout(
        &mut self,
        timeout: Option<std::time::Duration>,
    ) -> Result<(), std::io::Error>;
}

impl ReadTimeout for PacketStream<TcpStream> {
    fn set_read_timeout(
        &mut self,
        timeout: Option<std::time::Duration>,
    ) -> Result<(), std::io::Error> {
        self.stream.set_read_timeout(timeout)
    }
}
