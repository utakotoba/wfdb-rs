//! Header parsing and manipulation.
//!
//! This module handles reading and parsing of WFDB header files (.hea).

mod parser;
mod types;

pub use parser::parse_header;
pub use types::{Header, RecordMetadata, SegmentInfo, SignalInfo};
