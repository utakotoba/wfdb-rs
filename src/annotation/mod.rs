//! Annotation reading and parsing.
//!
//! This module provides functionality for reading and parsing WFDB annotation files.
//! It supports both MIT (standard) and AHA formats, with automatic format detection.
//!
//! # Example
//!
//! ```no_run
//! use wfdb::annotation::AnnotationReader;
//!
//! // Read annotations from a record
//! let mut reader = AnnotationReader::open("100", "atr")?;
//!
//! // Iterate over annotations
//! for annotation_result in reader {
//!     let annotation = annotation_result?;
//!     println!("Time: {}, Code: {:?}", annotation.time, annotation.code);
//! }
//!
//! // Or access all annotations at once
//! let reader = AnnotationReader::open("100", "atr")?;
//! for annotation in reader.annotations() {
//!     println!("Time: {}, Code: {:?}", annotation.time, annotation.code);
//! }
//! # Ok::<(), wfdb::Error>(())
//! ```

mod aha;
mod format;
mod mit;
mod reader;
mod types;

pub use aha::AhaParser;
pub use format::{AnnotationFormat, AnnotationParser, detect_format};
pub use mit::MitParser;
pub use reader::AnnotationReader;
pub use types::{ACMAX, Annotation, AnnotationCode};
