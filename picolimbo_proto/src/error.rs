use std::string::FromUtf8Error;

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
}

pub type Result<V> = std::result::Result<V, ProtoError>;
