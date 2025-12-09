//! Header parsing and manipulation.
//!
//! This module handles reading and parsing of WFDB header files (.hea).

mod common;
mod metadata;
mod segment_info;
mod signal_info;

pub use common::{Header, Specifications};
pub use metadata::Metadata;
pub use segment_info::SegmentInfo;
pub use signal_info::SignalInfo;
