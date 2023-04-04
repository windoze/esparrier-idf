use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("io error")]
    IoError(#[from] io::Error),
    #[error("did not match format")]
    FormatError,
    #[error("not enough data")]
    InsufficientDataError,
    #[error("Packet too small")]
    PacketTooSmall,
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("tcp connection failed")]
    TcpError(#[from] io::Error),
    #[error("invalid data received")]
    ProtocolError(#[from] PacketError),
}
