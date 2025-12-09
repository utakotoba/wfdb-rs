//! __WFDB__ (Waveform Database) library for pure Rust.
//!
//! This library provides _decoding_ and _encoding_ support for
//! `PhysioNet` [WFDB](https://physionet.org/content/wfdb) format files.

// pub mod annotation;
pub mod header;
pub mod signal;

// Internal module declaration
mod common;
// mod record;
mod shared;

pub use common::*;
pub use header::{Metadata, SegmentInfo, SignalInfo};
// pub use record::{Record, open};
pub use shared::*;
