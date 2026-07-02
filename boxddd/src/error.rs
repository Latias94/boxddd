use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error("failed to create Box3D world")]
    CreateWorldFailed,
    #[error("failed to create Box3D body")]
    CreateBodyFailed,
    #[error("failed to create Box3D shape")]
    CreateShapeFailed,
    #[error("invalid argument")]
    InvalidArgument,
}
