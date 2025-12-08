//! Header parsing and manipulation.
//!
//! This module handles reading and parsing of WFDB header files (.hea).

mod metadata;
mod parser;

pub use metadata::Metadata;
pub use parser::*;
