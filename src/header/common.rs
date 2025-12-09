use std::io::BufRead;

use crate::{Error, Result};

use super::{Metadata, SegmentInfo, SignalInfo};

/// Header specifications containing either signal or segment data.
///
/// A WFDB header contains either signal specifications (for single-segment)
/// or segment specifications (for multi-segment). These two are mutually exclusive.
#[derive(Debug, Clone, PartialEq)]
pub enum Specifications {
    /// Single-segment record with signal specifications.
    SingleSegment {
        /// Signal specifications for each signal in the record.
        signals: Vec<SignalInfo>,
    },
    /// Multi-segment record with segment specifications.
    MultiSegment {
        /// Segment specifications for each segment in the record.
        segments: Vec<SegmentInfo>,
    },
}

impl Specifications {
    /// Get the signal specifications if this is a single-segment record.
    ///
    /// Returns `Some` for single-segment records, `None` for multi-segment records.
    #[must_use]
    pub fn signals(&self) -> Option<&[SignalInfo]> {
        if let Self::SingleSegment { signals } = self {
            Some(signals)
        } else {
            None
        }
    }

    /// Get the segment specifications if this is a multi-segment record.
    ///
    /// Returns `Some` for multi-segment records, `None` for single-segment records.
    #[must_use]
    pub fn segments(&self) -> Option<&[SegmentInfo]> {
        if let Self::MultiSegment { segments } = self {
            Some(segments)
        } else {
            None
        }
    }

    /// Check if this is a multi-segment record.
    #[must_use]
    pub const fn is_multi_segment(&self) -> bool {
        matches!(self, Self::MultiSegment { .. })
    }

    /// Check if this is a single-segment record.
    #[must_use]
    pub const fn is_single_segment(&self) -> bool {
        matches!(self, Self::SingleSegment { .. })
    }
}

/// Parsed WFDB header file content.
///
/// A header file specifies the record metadata, and either signal specifications
/// (for single-segment records) or segment specifications (for multi-segment records).
/// These two types are mutually exclusive.
///
/// # Examples
///
/// Here are a few examples of header file structures:
///
/// ## Single-segment record (MIT-BIH Database record 100):
/// ```text
/// 100 2 360 650000 0:0:0 0/0/0
/// 100.dat 212 200 11 1024 995 -22131 0 MLII
/// 100.dat 212 200 11 1024 1011 20052 0 V5
/// # 69 M 1085 1629 x1
/// # Aldomet, Inderal
/// ```
///
/// ## Multi-segment record:
/// ```text
/// multi/3 2 360 45000
/// 100s 21600
/// null 1800
/// 100s 21600
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// Record metadata from the record line.
    pub metadata: Metadata,
    /// Specifications (signals for single-segment, segments for multi-segment).
    pub specifications: Specifications,
    /// Info strings (comments following signal/segment specifications).
    ///
    /// Each string represents the content of one comment line (without the '#' prefix).
    pub info_strings: Vec<String>,
}

impl Header {
    // [Header decoding functions]

    /// Parse a WFDB header from a buffered reader.
    ///
    /// # Format
    ///
    /// Header files contain ASCII text with the following structure:
    /// 1. Optional comment lines (starting with '#')
    /// 2. Record line (required)
    /// 3. Signal specification lines (for single-segment records) OR
    ///    Segment specification lines (for multi-segment records)
    /// 4. Optional info strings (comment lines after specifications)
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// - The record line is missing or invalid
    /// - Signal/segment specifications are missing or invalid
    /// - The number of specifications doesn't match the record line
    pub fn from_reader<R: BufRead>(reader: &mut R) -> Result<Self> {
        // Use iterator-based approach with proper line handling
        let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<String>>>()?;

        Self::from_lines(&lines)
    }

    /// Parse a WFDB header from a slice of lines.
    ///
    /// This is the internal parsing function used by `from_reader`.
    fn from_lines(lines: &[String]) -> Result<Self> {
        // Find the first non-empty, non-comment line (record line)
        let record_line_idx = lines
            .iter()
            .position(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#')
            })
            .ok_or_else(|| Error::InvalidHeader("Missing record line in header".to_string()))?;

        // Parse the record line
        let metadata = Metadata::from_record_line(&lines[record_line_idx])?;
        let mut line_idx = record_line_idx + 1;

        // Determine if this is a multi-segment record
        let is_multi_segment = metadata.num_segments.is_some();

        // Parse signal or segment specifications
        let (signals, segments) = if is_multi_segment {
            // Parse segment specifications
            #[allow(clippy::expect_used)]
            let num_segments = metadata.num_segments.expect("num_segments should be Some");
            let mut segment_specs = Vec::new();

            while line_idx < lines.len() && segment_specs.len() < num_segments {
                let line = lines[line_idx].trim();

                // Skip comments (but they shouldn't appear before all segments are read)
                if line.is_empty() || line.starts_with('#') {
                    line_idx += 1;
                    continue;
                }

                segment_specs.push(SegmentInfo::from_segment_line(line)?);
                line_idx += 1;
            }

            if segment_specs.len() < num_segments {
                return Err(Error::InvalidHeader(format!(
                    "Expected {} segment specifications, found {}",
                    num_segments,
                    segment_specs.len()
                )));
            }

            (None, Some(segment_specs))
        } else {
            // Parse signal specifications
            let num_signals = metadata.num_signals;
            let mut signal_specs = Vec::new();

            while line_idx < lines.len() && signal_specs.len() < num_signals {
                let line = lines[line_idx].trim();

                // Skip comments (but they shouldn't appear before all signals are read)
                if line.is_empty() || line.starts_with('#') {
                    line_idx += 1;
                    continue;
                }

                signal_specs.push(SignalInfo::from_signal_line(line)?);
                line_idx += 1;
            }

            if signal_specs.len() < num_signals {
                return Err(Error::InvalidHeader(format!(
                    "Expected {} signal specifications, found {}",
                    num_signals,
                    signal_specs.len()
                )));
            }

            (Some(signal_specs), None)
        };

        // Parse info strings (trailing comment lines)
        let mut info_strings = Vec::new();

        while line_idx < lines.len() {
            let line = lines[line_idx].trim();

            if line.starts_with('#') {
                // Remove the '#' prefix and collect as info string
                let info = line.trim_start_matches('#').to_string();
                info_strings.push(info);
            }

            line_idx += 1;
        }

        #[allow(clippy::expect_used)]
        let specifications = match (signals, segments) {
            (Some(signals), None) => Specifications::SingleSegment { signals },
            (None, Some(segments)) => Specifications::MultiSegment { segments },
            _ => unreachable!("Either signals or segments should be Some, but not both"),
        };

        Ok(Self {
            metadata,
            specifications,
            info_strings,
        })
    }

    // [Accessors]

    /// Get the record metadata.
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Get the specifications (signals or segments).
    #[must_use]
    pub const fn specifications(&self) -> &Specifications {
        &self.specifications
    }

    /// Get the signal specifications.
    ///
    /// Returns `Some` for single-segment records, `None` for multi-segment records.
    #[must_use]
    pub fn signals(&self) -> Option<&[SignalInfo]> {
        self.specifications.signals()
    }

    /// Get the segment specifications.
    ///
    /// Returns `Some` for multi-segment records, `None` for single-segment records.
    #[must_use]
    pub fn segments(&self) -> Option<&[SegmentInfo]> {
        self.specifications.segments()
    }

    /// Get the info strings.
    #[must_use]
    pub fn info_strings(&self) -> &[String] {
        &self.info_strings
    }

    /// Check if this is a multi-segment record.
    #[must_use]
    pub const fn is_multi_segment(&self) -> bool {
        self.specifications.is_multi_segment()
    }

    /// Check if this is a single-segment record.
    #[must_use]
    pub const fn is_single_segment(&self) -> bool {
        self.specifications.is_single_segment()
    }

    /// Get the number of signals.
    #[must_use]
    pub const fn num_signals(&self) -> usize {
        self.metadata.num_signals
    }

    /// Get the number of segments (for multi-segment records).
    ///
    /// Returns `Some` for multi-segment records, `None` for single-segment records.
    #[must_use]
    pub const fn num_segments(&self) -> Option<usize> {
        self.metadata.num_segments
    }
}
