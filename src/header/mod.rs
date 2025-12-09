//! Header parsing and manipulation.
//!
//! This module handles reading and parsing of WFDB header files (.hea).

mod metadata;
mod parser;
mod signal_info;

pub use metadata::Metadata;
pub use parser::*;
pub use signal_info::SignalInfo;
