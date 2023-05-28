use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ProtoError {
    #[error("String is too long! ({0} > {1})")]
    StringError(i32, i32),
}

pub type Result<V> = std::result::Result<V, ProtoError>;
