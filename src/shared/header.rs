use chrono::{NaiveDate, NaiveTime};

/// Metadata from the WFDB header record line.
///
/// # Examples
///
/// Here are a few examples of a validated record line:
///
/// - `100 2 360 650000 12:00:00 01/01/2000` _refer to the example on
///   [WFDB website](https://wfdb.io/spec/header-files.html#record-line)_
/// - `my_record_0 12 500/100(50) 675000 14:49:37 07/06/2025`
/// - `24_record/2 4 102400` (Many fields are optional.)
#[derive(Debug, Clone, PartialEq)]
pub struct RecordMetadata {
    /// Identifier for the record (letters, digits, underscores only).
    pub name: String,
    /// If present, appended as /n. Indicates a multi-segment record.
    pub num_segments: Option<usize>,
    /// Number of signals described in the header.
    pub num_signals: usize,
    /// Samples per second (Hz) per signal.
    pub sampling_frequency: f64,
    /// Frequency (Hz) for counter (secondary clock).
    pub counter_frequency: Option<f64>,
    /// Offset value for counter.
    pub base_counter: Option<f64>,
    /// Total samples per signal.
    pub num_samples: Option<u64>,
    /// Start time of the recording (HH:MM:SS).
    pub base_time: Option<NaiveTime>,
    /// Start date of the recording (DD/MM/YYYY).
    pub base_date: Option<NaiveDate>,
}

/// Information about a segment in a multi-segment record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentInfo {
    /// Name of the segment record.
    pub name: String,
    /// Number of samples in the segment.
    pub num_samples: u64,
}

/// Parsed header content.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// Record metadata.
    pub metadata: RecordMetadata,
    /// Signal specifications.
    pub signals: Vec<super::SignalInfo>,
    /// Segment information (for multi-segment records).
    pub segments: Option<Vec<SegmentInfo>>,
    /// Info strings (comments).
    pub info_strings: Vec<String>,
}
