use thiserror::Error;

/// Errors that may occur within the Rust WFDB library.
///
/// This enum represents possible error conditions that can arise
/// when working with waveform database format files by the library.
#[derive(Error, Debug)]
pub enum Error {
    /// Indicates that the specified signal format code is not supported.
    ///
    /// > The contained value is the unsupported format code encountered.
    #[error("Unsupported signal format: {0}")]
    UnsupportedSignalFormat(u16),

    /// Indicates that an invalid annotation code was encountered.
    ///
    /// Annotation codes must be in the range 0-49. The contained value is
    /// the invalid code that was encountered.
    #[error("Invalid annotation code: {0}")]
    InvalidAnnotationCode(u8),

    /// Indicates that an unsupported annotation format was requested.
    ///
    /// The contained string describes the format that is not supported.
    #[error("Unsupported annotation format: {0}")]
    UnsupportedAnnotationFormat(String),

    /// Indicates an invalid file or path was encountered.
    ///
    /// The contained string describes the path or file that is invalid.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Wraps I/O errors that occur during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Indicates an invalid header format.
    #[error("Invalid header: {0}")]
    InvalidHeader(String),
}
