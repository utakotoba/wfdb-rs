//! __WFDB__ (Waveform Database) library for pure Rust.
//!
//! This library provides _decoding_ ~~and _encoding_~~(maybe in the future) support for
//! `PhysioNet`'s  [WFDB](https://physionet.org/content/wfdb) format files.

// pub mod annotation;
pub mod header;
pub mod record;
pub mod signal;

// Internal module declaration
mod common;
mod error;

pub use common::*;
pub use error::Error;
pub use header::{Header, Metadata, SegmentInfo, SignalInfo};
pub use record::{MultiSignalReader, Record, SignalReader};
