mod actuator;
mod client;
mod error;
mod packet;
mod packet_io;
mod packet_stream;
mod clipboard;
mod thread_act;
mod take;

pub use error::{ConnectionError, PacketError};
pub use packet::Packet;
pub use packet_io::{PacketReader, PacketWriter};
pub use packet_stream::PacketStream;
pub use actuator::Actuator;
pub use client::start;
pub use thread_act::{ActMsg, ThreadedActuator};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
