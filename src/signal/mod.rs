//! Signal reading and decoding.
//!
//! This module provides low-level signal format decoders that work with any
//! `BufRead` source. Each format decoder is responsible for reading raw bytes
//! and converting them to `Sample` values according to the WFDB specification.
//!
//! # Hybrid APIs
//!
//! The module provides three levels of API for different use cases:
//!
//! 1. **Low-level buffer API** - [`FormatDecoder::decode_buf()`]
//!    - Zero-copy performance
//!    - Caller manages buffer allocation
//!    - Best for tight loops or custom allocation strategies
//!
//! 2. **High-level ergonomic API** - [`FormatDecoder::decode()`]
//!    - Returns `Vec<Sample>`
//!    - Simple and idiomatic Rust
//!    - Best for most use cases
//!
//! 3. **Iterator API** - [`FormatDecoder::samples()`]
//!    - Lazy evaluation
//!    - Integrates with Rust's iterator ecosystem
//!    - Best for streaming, filtering, or processing one sample at a time
//!
//! # Format Support
//!
//! Each WFDB signal format has a dedicated decoder:
//! - Format 0: Null signals (no data)
//! - Format 8: 8-bit first differences
//! - Format 16: 16-bit two's complement (little-endian)
//! - Format 24: 24-bit two's complement (little-endian)
//! - Format 32: 32-bit two's complement (little-endian)
//! - Format 61: 16-bit two's complement (big-endian)
//! - Format 80: 8-bit offset binary
//! - Format 160: 16-bit offset binary
//! - Format 212: Packed 12-bit samples (2 samples in 3 bytes)
//! - Format 310: Packed 10-bit samples (3 samples in 4 bytes)
//! - Format 311: Packed 10-bit samples (alternative layout)
//!
//! > _FLAC-based formats are not supported yet._
//!
//! # Examples
//!
//! ## Buffer API
//!
//! ```no_run
//! use wfdb::signal::{Format16Decoder, FormatDecoder};
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! # fn main() -> wfdb::Result<()> {
//! let file = File::open("data/100.dat")?;
//! let mut reader = BufReader::new(file);
//! let mut decoder = Format16Decoder::new();
//!
//! // Reuse buffer for performance
//! let mut buffer = vec![0i32; 1000];
//! loop {
//!     let n = decoder.decode_buf(&mut reader, &mut buffer)?;
//!     if n == 0 { break; }
//!     // Process samples in buffer[..n]
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Ergonomic API
//!
//! ```no_run
//! use wfdb::signal::{Format16Decoder, FormatDecoder};
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! # fn main() -> wfdb::Result<()> {
//! let file = File::open("data/100.dat")?;
//! let mut reader = BufReader::new(file);
//! let mut decoder = Format16Decoder::new();
//!
//! // Simple and clean
//! let samples = decoder.decode(&mut reader, 1000)?;
//! println!("Read {} samples", samples.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Iterator API
//!
//! ```no_run
//! use wfdb::signal::{Format16Decoder, FormatDecoder};
//! use std::io::BufReader;
//! use std::fs::File;
//!
//! # fn main() -> wfdb::Result<()> {
//! let file = File::open("data/100.dat")?;
//! let reader = BufReader::new(file);
//! let mut decoder = Format16Decoder::new();
//!
//! // Use iterator
//! let positive_samples: Vec<_> = decoder.samples(reader)
//!     .take(1000)
//!     .filter_map(Result::ok)
//!     .filter(|&s| s > 0)
//!     .collect();
//! # Ok(())
//! # }
//! ```

mod common;
mod format0;
mod format16;
mod format160;
mod format212;
mod format24;
mod format310;
mod format311;
mod format32;
mod format61;
mod format8;
mod format80;

pub use common::{DecoderConfig, FormatDecoder, INVALID_SAMPLE, SampleIter, sign_extend};
pub use format0::Format0Decoder;
pub use format8::Format8Decoder;
pub use format16::Format16Decoder;
pub use format24::Format24Decoder;
pub use format32::Format32Decoder;
pub use format61::Format61Decoder;
pub use format80::Format80Decoder;
pub use format160::Format160Decoder;
pub use format212::Format212Decoder;
pub use format310::Format310Decoder;
pub use format311::Format311Decoder;

use crate::{Error, Result, Sample, SignalFormat};

/// Create a decoder for given signal format.
///
/// # Errors
///
/// Returns `Error::UnsupportedSignalFormat` if the format is not supported.
pub fn get_decoder(format: SignalFormat, initial_value: Sample) -> Result<Box<dyn FormatDecoder>> {
    match format {
        SignalFormat::Format0 => Ok(Box::new(Format0Decoder::new())),
        SignalFormat::Format8 => Ok(Box::new(Format8Decoder::new(initial_value))),
        SignalFormat::Format16 => Ok(Box::new(Format16Decoder::new())),
        SignalFormat::Format24 => Ok(Box::new(Format24Decoder::new())),
        SignalFormat::Format32 => Ok(Box::new(Format32Decoder::new())),
        SignalFormat::Format61 => Ok(Box::new(Format61Decoder::new())),
        SignalFormat::Format80 => Ok(Box::new(Format80Decoder::new())),
        SignalFormat::Format160 => Ok(Box::new(Format160Decoder::new())),
        SignalFormat::Format212 => Ok(Box::new(Format212Decoder::new())),
        SignalFormat::Format310 => Ok(Box::new(Format310Decoder::new())),
        SignalFormat::Format311 => Ok(Box::new(Format311Decoder::new())),
        _ => Err(Error::UnsupportedSignalFormat(u16::from(format))),
    }
}
