use thiserror::Error;

/// Result type used by the safe Box3D wrapper.
pub type Result<T> = std::result::Result<T, Error>;

/// Public alias for APIs that return a boxddd error.
pub type ApiResult<T> = Result<T>;

/// Public alias for the boxddd error type.
pub type ApiError = Error;

/// Errors returned by safe wrapper operations.
#[non_exhaustive]
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The operation was attempted while Box3D was executing a callback.
    #[error("boxddd API called from a Box3D callback; Box3D world is locked")]
    InCallback,

    /// Box3D did not return a valid world id.
    #[error("failed to create Box3D world")]
    CreateWorldFailed,

    /// Box3D did not return a valid body id.
    #[error("failed to create Box3D body")]
    CreateBodyFailed,

    /// Box3D did not return a valid shape id.
    #[error("failed to create Box3D shape")]
    CreateShapeFailed,

    /// Box3D did not return a valid recording handle.
    #[error("failed to create Box3D recording")]
    CreateRecordingFailed,

    /// Box3D did not return a valid dynamic tree handle.
    #[error("failed to create Box3D dynamic tree")]
    CreateDynamicTreeFailed,

    /// Box3D did not return a valid replay player handle.
    #[error("failed to create Box3D replay player")]
    CreateRecPlayerFailed,

    /// A native recording file operation failed.
    #[error("failed to load or save a Box3D recording")]
    RecordingIoFailed,

    /// An input value failed validation.
    #[error("invalid argument")]
    InvalidArgument,

    /// The world handle failed Box3D id validation.
    #[error("invalid WorldId")]
    InvalidWorldId,

    /// The body handle failed Box3D id validation.
    #[error("invalid BodyId")]
    InvalidBodyId,

    /// The shape handle failed Box3D id validation.
    #[error("invalid ShapeId")]
    InvalidShapeId,

    /// The joint handle failed Box3D id validation.
    #[error("invalid JointId")]
    InvalidJointId,

    /// The contact handle failed Box3D id validation.
    #[error("invalid ContactId")]
    InvalidContactId,

    /// A joint-specific API was called with a different joint type.
    #[error("wrong joint type for this API")]
    WrongJointType,

    /// The provided index is outside the range accepted by the native API.
    #[error("index out of range for this API")]
    IndexOutOfRange,

    /// A string passed to Box3D contained an interior NUL byte.
    #[error("string contains an interior NUL byte")]
    NulByteInString,

    /// A native resource would outlive the resource it depends on.
    #[error("native resource lifetime would be violated")]
    ResourceLifetimeViolation,

    /// The operation is unavailable for the current WebAssembly backend.
    #[error("this API is not supported on the current WASM target")]
    UnsupportedOnWasm,

    /// A Rust callback panicked and boxddd stopped the native traversal safely.
    #[error("Rust callback panicked and native traversal was stopped")]
    CallbackPanicked,

    /// All global callback slots are currently in use.
    #[error("no callback slot is available")]
    CallbackSlotsExhausted,

    /// A provider-mode callback bridge failed while collecting debug draw data.
    #[error("provider callback bridge failed")]
    ProviderCallbackFailed,
}
