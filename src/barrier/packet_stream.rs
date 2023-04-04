use std::collections::HashMap;

use log::debug;

use super::{PacketReader, PacketWriter, PacketError, Packet};

pub struct PacketStream<S: PacketReader + PacketWriter> {
    stream: S,
}

impl<S: PacketReader + PacketWriter> PacketStream<S> {
    pub fn new(stream: S) -> Self {
        Self { stream }
    }

    pub fn read(&mut self) -> Result<Packet, PacketError> {
        // let mut x = [0; 4];
        // self.stream.read(&mut x)?;
        // let size = u32::from_be_bytes(x);
        let size = self.stream.read_packet_size()?;
        if size < 4 {
            let mut vec = Vec::new();
            self.stream.read_to_end(&mut vec)?;
            return Err(PacketError::PacketTooSmall);
        }
        let code: [u8; 4] = self.stream.read_bytes_fixed()?;
        let code = String::from_utf8_lossy(&code).into_owned();

        let packet = match code.as_ref() {
            "QINF" => Packet::QueryInfo,
            "CIAK" => Packet::InfoAck,
            "CALV" => Packet::KeepAlive,
            "CROP" => Packet::ResetOptions,
            "DSOP" => {
                let num_items = self.stream.read_u32()?;
                let num_opts = num_items / 2;
                let mut opts = HashMap::new();
                for _ in 0..num_opts {
                    let opt: [u8; 4] = self.stream.read_bytes_fixed()?;
                    let opt = String::from_utf8_lossy(&opt).into_owned();
                    let val = self.stream.read_u32()?;
                    debug!("Read option: {opt} with value {val:?}");
                    opts.insert(opt, val);
                }
                Packet::SetDeviceOptions(opts)
            }
            "EUNK" => Packet::ErrorUnknownDevice,
            "DMMV" => {
                let x = self.stream.read_u16()?;
                let y = self.stream.read_u16()?;
                Packet::MouseMoveAbs { x, y }
            }
            "CINN" => {
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
            "COUT" => Packet::CursorLeave,
            "CCLP" => {
                let id = self.stream.read_u8()?;
                let seq_num = self.stream.read_u32()?;
                Packet::GrabClipboard { id, seq_num }
            }
            "DCLP" => {
                let id = self.stream.read_u8()?;
                let seq_num = self.stream.read_u32()?;
                let mark = self.stream.read_u8()?;

                // NOTE: Do not save clipboard content as MCU doesn't have enough memory
                // let data = self.stream.read_bytes()?;
                self.stream.consume_bytes()?;
                let data = Vec::new();
                
                Packet::SetClipboard {
                    id,
                    seq_num,
                    mark,
                    data,
                }
            }
            "DMUP" => {
                let id = self.stream.read_i8()?;
                Packet::MouseUp { id }
            }
            "DMDN" => {
                let id = self.stream.read_i8()?;
                Packet::MouseDown { id }
            }
            "DKUP" => {
                let id = self.stream.read_u16()?;
                let mask = self.stream.read_u16()?;
                let button = self.stream.read_u16()?;
                Packet::KeyUp { id, mask, button }
            }
            "DKDN" => {
                let id = self.stream.read_u16()?;
                let mask = self.stream.read_u16()?;
                let button = self.stream.read_u16()?;
                Packet::KeyDown { id, mask, button }
            }
            "DKRP" => {
                let id = self.stream.read_u16()?;
                let mask = self.stream.read_u16()?;
                let count = self.stream.read_u16()?;
                let button = self.stream.read_u16()?;
                Packet::KeyRepeat { id, mask, button, count }
            }
            "DMWM" => {
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
