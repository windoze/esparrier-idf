use std::{io::Write, net::TcpStream, time::Duration};

use log::{debug, info};

use crate::barrier::packet_stream::ReadTimeout;

use super::{Actuator, ConnectionError, Packet, PacketReader, PacketStream, PacketWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardStage {
    None,
    Mark1,
    Mark2(usize),
    Mark3,
}

pub fn start<A: Actuator>(
    addr: &str,
    port: u16,
    device_name: &str,
    actor: &mut A,
) -> Result<(), ConnectionError> {
    let screen_size: (u16, u16) = actor.get_screen_size();

    let mut stream = TcpStream::connect((addr, port))?;
    // Turn off Nagle, this may not be available on ESP-IDF, so ignore the error.
    stream.set_nodelay(true).ok();
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();

    let _size = stream.read_packet_size()?;
    stream.read_str_lit("Barrier")?;
    let major = stream.read_u16()?;
    let minor = stream.read_u16()?;
    debug!("Got hello {major}:{minor}");

    stream.write_u32("Barrier".len() as u32 + 2 + 2 + 4 + device_name.bytes().len() as u32)?;
    stream.write_all(b"Barrier")?;
    stream.write_u16(1)?;
    stream.write_u16(6)?;
    stream.write_str(device_name)?;

    actor.connected();

    let mut clipboard_stage = ClipboardStage::None;
    let mut packet_stream = PacketStream::new(stream);
    while let Ok(packet) = packet_stream.read(&mut clipboard_stage) {
        match packet {
            Packet::QueryInfo => {
                packet_stream
                    .write(Packet::DeviceInfo {
                        x: 0,
                        y: 0,
                        w: screen_size.0,
                        h: screen_size.1,
                        _dummy: 0,
                        mx: 0,
                        my: 0,
                    })
                    .map_err(|e| {
                        actor.disconnected();
                        e
                    })?;
            }
            Packet::KeepAlive => {
                packet_stream.write(Packet::KeepAlive).map_err(|e| {
                    actor.disconnected();
                    e
                })?;
            }
            Packet::MouseMoveAbs { x, y } => {
                let abs_x = ((x as f32) * (0x7fff as f32 / (screen_size.0 as f32))).ceil() as u16;
                let abs_y = ((y as f32) * (0x7fff as f32 / (screen_size.1 as f32))).ceil() as u16;
                actor.set_cursor_position(abs_x, abs_y);
            }
            Packet::KeyUp { id, mask, button } => {
                actor.key_up(id, mask, button);
            }
            Packet::KeyDown { id, mask, button } => {
                actor.key_down(id, mask, button);
            }
            Packet::KeyRepeat {
                id,
                mask,
                button,
                count,
            } => {
                actor.key_repeat(id, mask, button, count);
            }
            Packet::MouseDown { id } => {
                actor.mouse_down(id);
            }
            Packet::MouseUp { id } => {
                actor.mouse_up(id);
            }
            Packet::MouseWheel { x_delta, y_delta } => {
                actor.mouse_wheel(x_delta, y_delta);
            }
            Packet::InfoAck => { //Ignore
            }
            Packet::ResetOptions => {
                actor.reset_options();
            }
            Packet::SetDeviceOptions { heartbeat } => {
                info!("Set Heartbeat Interval: {heartbeat}");
                // Set the read timeout to twice the heartbeat interval
                packet_stream
                    .set_read_timeout(Some(Duration::from_millis(heartbeat as u64 * 2)))
                    .ok();
                actor.set_options(heartbeat);
            }
            Packet::CursorEnter { .. } => {
                actor.enter();
            }
            Packet::CursorLeave => {
                actor.leave();
            }
            Packet::SetClipboard {
                id: _,
                seq_num: _,
                mark: _,
                data,
            } => {
                if let Some(data) = data {
                    actor.set_clipboard(data)
                }
            }
            Packet::GrabClipboard { .. } => {}
            Packet::DeviceInfo { .. } | Packet::ErrorUnknownDevice | Packet::ClientNoOp => {
                // Server only packets
            }
            Packet::Unknown(_) => {}
        }
    }
    panic!("Connection closed.");
}
