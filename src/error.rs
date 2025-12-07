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
}
