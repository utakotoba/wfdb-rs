//! Annotation format detection and parsing traits.
//!
//! This module provides traits and utilities for detecting and parsing
//! different annotation file formats (MIT and AHA).

use crate::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// Annotation file format types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationFormat {
    /// MIT format (standard WFDB annotation format).
    Mit,
    /// AHA format (American Heart Association format).
    Aha,
}

impl AnnotationFormat {
    /// Returns a human-readable description of the format.
    pub fn description(&self) -> &'static str {
        match self {
            AnnotationFormat::Mit => "MIT (standard WFDB) format",
            AnnotationFormat::Aha => "AHA (American Heart Association) format",
        }
    }
}

/// Trait for parsing annotations from a specific format.
pub trait AnnotationParser {
    /// Parses all annotations from the given reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - A reader positioned at the start of the annotation file.
    ///
    /// # Returns
    ///
    /// A vector of parsed annotations, or an error if parsing fails.
    fn parse<R: Read>(reader: R) -> Result<Vec<super::Annotation>>;
}

/// Detects the annotation format of a file.
///
/// This function attempts to detect whether an annotation file is in MIT or AHA format
/// by examining the first few bytes. The reader is reset to the beginning after detection.
///
/// # Arguments
///
/// * `reader` - A seekable reader positioned at the start of the annotation file.
///
/// # Returns
///
/// The detected format, or `AnnotationFormat::Mit` if detection fails (default assumption).
///
/// # Errors
///
/// Returns an error if the reader cannot be read or seeked.
pub fn detect_format<R: Read + Seek>(reader: &mut R) -> Result<AnnotationFormat> {
    let pos = reader.stream_position()?;
    let mut bytes = [0u8; 2];

    match reader.read_exact(&mut bytes) {
        Ok(_) => {
            reader.seek(SeekFrom::Start(pos))?;

            let [byte0, byte1] = bytes;

            // MIT format detection:
            // - The annotation type is in bits 2-7 of byte1
            // - The time difference is in bits 0-1 of byte1 and all of byte0
            // - Valid MIT format: annotation_type <= 63 and time_diff bits < 4
            let annotation_type = (byte1 >> 2) & 0x3F;
            let time_diff_bits = byte1 & 0x03;

            // AHA format detection:
            // - AHA files begin with a null byte (byte0 == 0)
            // - Followed by an ASCII character that is a legal AHA annotation code
            if byte0 == 0 && byte1 != 0 && byte1 != b'[' && byte1 != b']' {
                if (32..=126).contains(&byte1) {
                    Ok(AnnotationFormat::Aha)
                } else {
                    Ok(AnnotationFormat::Mit)
                }
            } else if annotation_type <= 63 && time_diff_bits < 4 {
                Ok(AnnotationFormat::Mit)
            } else {
                // Default to MIT if uncertain
                Ok(AnnotationFormat::Mit)
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            // Empty file, default to MIT
            Ok(AnnotationFormat::Mit)
        }
        Err(e) => Err(Error::Io(e)),
    }
}
