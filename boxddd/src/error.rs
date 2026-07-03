use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type ApiResult<T> = Result<T>;
pub type ApiError = Error;

#[non_exhaustive]
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    #[error("boxddd API called from a Box3D callback; Box3D world is locked")]
    InCallback,
    #[error("failed to create Box3D world")]
    CreateWorldFailed,
    #[error("failed to create Box3D body")]
    CreateBodyFailed,
    #[error("failed to create Box3D shape")]
    CreateShapeFailed,
    #[error("failed to create Box3D recording")]
    CreateRecordingFailed,
    #[error("failed to create Box3D dynamic tree")]
    CreateDynamicTreeFailed,
    #[error("failed to create Box3D replay player")]
    CreateRecPlayerFailed,
    #[error("failed to load or save a Box3D recording")]
    RecordingIoFailed,
    #[error("invalid argument")]
    InvalidArgument,
    #[error("invalid WorldId")]
    InvalidWorldId,
    #[error("invalid BodyId")]
    InvalidBodyId,
    #[error("invalid ShapeId")]
    InvalidShapeId,
    #[error("invalid JointId")]
    InvalidJointId,
    #[error("invalid ContactId")]
    InvalidContactId,
    #[error("wrong joint type for this API")]
    WrongJointType,
    #[error("index out of range for this API")]
    IndexOutOfRange,
    #[error("string contains an interior NUL byte")]
    NulByteInString,
    #[error("native resource lifetime would be violated")]
    ResourceLifetimeViolation,
    #[error("this API is not supported on the current WASM target")]
    UnsupportedOnWasm,
    #[error("Rust callback panicked and native traversal was stopped")]
    CallbackPanicked,
    #[error("no callback slot is available")]
    CallbackSlotsExhausted,
}
