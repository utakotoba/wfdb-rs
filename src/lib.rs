//! __WFDB__ (Waveform Database) library for pure Rust.
//!
//! This library provides _decoding_ and _encoding_ support for
//! `PhysioNet` [WFDB](https://physionet.org/content/wfdb) format files.

pub mod annotation;
mod error;
pub mod header;
mod record;
pub mod signal;

pub use error::Error;
use header::Header;
pub use record::{Record, open};
use signal::SignalFormat;
use signal::SignalReader;

/// A specialized `Result` type for the WFDB library with its `Error` enum.
pub type Result<T> = std::result::Result<T, Error>;

/// A single **sample value** from a waveform signal.
///
/// This type uses an `i32` to accommodate the maximum size and range of all
/// signal data formats defined in the WFDB standard.
///
/// All signal data read will be stored internally as a `Sample`. When the data is
/// encoded back into bytes for a specific WFDB format, the value will be
/// downcast to the actual required size.
pub type Sample = i32;

/// A single **time value** from a waveform signal.
pub type Time = i64;
