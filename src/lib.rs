//! __WFDB__ (Waveform Database) library for pure Rust.
//!
//! This library provides _decoding_ and _encoding_ support for
//! PhysioNet [WFDB](https://physionet.org/content/wfdb) format files.

mod error;
mod signal_format;

pub use error::Error;
pub use signal_format::SignalFormat;

/// A specialized `Result` type for the WFDB library with its `Error` enum.
///
/// > This type is a shorthand for `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// A single **sample value** from a waveform signal.
///
/// > This type uses an __`i32`__ to accommodate the maximum size and range of all
/// > signal data formats defined in the __WFDB__ (Waveform Database) standard.
///
/// > All signal data read will be stored internally as a `Sample`. When the data is
/// > encoded back into bytes for a specific WFDB format, the value will be
/// > __downcast__ to the actual required size (e.g., `i16` or `i8`).
pub type Sample = i32;

/// A single **time value** from a waveform signal.
///
/// Only the magnitude is significant; the sign of a
/// `Time` variable indicates how it is to be printed.
pub type Time = i64;
