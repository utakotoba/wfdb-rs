//! Signal reading and processing.
//!
//! This module provides functionality for reading WFDB signal files.

mod formats;
mod reader;
mod types;

pub use reader::SignalReader;
pub use types::SignalFormat;
