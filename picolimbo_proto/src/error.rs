use std::{str::Utf8Error, string::FromUtf8Error};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtoError {
    #[error("String is too long! ({0} > {1})")]
    StringError(i32, i32),
    #[error("Error during an IO operation: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Error during an operation with VarInts: {0}")]
    VarintError(&'static str),
    #[error("Error with string decoding: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("Error with string decoding: {0}")]
    Utf8ErrorStr(#[from] Utf8Error),
    #[error("Invalid enum index: {0}!")]
    EnumError(i32),
    #[error("Failed to serialize JSON: {0}")]
    SerializationError(String),
    #[error("Error during an operation with NBT: {0}")]
    NBTError(#[from] nbt::Error),
    #[error("Packet ID mismatch. Packet of id {0} was unexpected.")]
    InvalidPacket(i32),
}

pub type Result<V> = std::result::Result<V, ProtoError>;
