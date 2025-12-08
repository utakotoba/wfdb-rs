//! Shared enums and data structs across the whole library

mod header;
mod signal;

pub use header::{Header, RecordMetadata, SegmentInfo};
pub use signal::{SignalFormat, SignalInfo};
