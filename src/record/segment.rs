use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::{Error, Header, Result, SegmentInfo, SignalInfo};

/// State of a segment in a multi-segment record.
#[derive(Debug, Clone)]
enum SegmentState {
    /// Segment not yet loaded.
    NotLoaded,
    /// Segment loaded successfully.
    Loaded(Box<SegmentData>),
    /// Null segment (name is "~").
    Null,
}

/// Loaded segment data.
#[derive(Debug, Clone)]
pub struct SegmentData {
    /// Segment header.
    pub header: Header,
    /// Base path for signal files.
    pub base_path: PathBuf,
}

/// Multi-segment record manager.
///
/// Handles lazy loading of segment headers and coordinates segment switching.
pub struct SegmentManager {
    /// Base path for the multi-segment record.
    base_path: PathBuf,
    /// Segment specifications from the master header.
    segments: Vec<SegmentInfo>,
    /// Current state of each segment.
    states: Vec<SegmentState>,
    /// Currently active segment index.
    current_segment: usize,
    /// Cumulative sample counts (for seeking across segments).
    #[allow(dead_code)]
    cumulative_samples: Vec<u64>,
}

impl SegmentManager {
    /// Create a new segment manager.
    pub fn new(base_path: PathBuf, segments: Vec<SegmentInfo>) -> Self {
        let num_segments = segments.len();
        let states = vec![SegmentState::NotLoaded; num_segments];

        // Calculate cumulative sample counts
        let mut cumulative_samples = Vec::with_capacity(num_segments + 1);
        cumulative_samples.push(0);
        let mut total = 0u64;
        for segment in &segments {
            total += segment.num_samples;
            cumulative_samples.push(total);
        }

        Self {
            base_path,
            segments,
            states,
            current_segment: 0,
            cumulative_samples,
        }
    }

    /// Load a segment header.
    pub fn load_segment(&mut self, index: usize) -> Result<&SegmentData> {
        if index >= self.segments.len() {
            return Err(Error::InvalidHeader(format!(
                "Segment index {} out of bounds (record has {} segments)",
                index,
                self.segments.len()
            )));
        }

        // Check if already loaded
        if matches!(&self.states[index], SegmentState::Loaded(_)) {
            match &self.states[index] {
                SegmentState::Loaded(data) => return Ok(data),
                _ => unreachable!(),
            }
        }

        if matches!(&self.states[index], SegmentState::Null) {
            return Err(Error::InvalidHeader("Cannot load null segment".to_string()));
        }

        let segment_info = &self.segments[index];

        // Check for null segment
        if segment_info.record_name == "~" {
            self.states[index] = SegmentState::Null;
            return Err(Error::InvalidHeader(
                "Segment is null (missing data)".to_string(),
            ));
        }

        // Load segment header
        let segment_header_path = self
            .base_path
            .join(format!("{}.hea", segment_info.record_name));

        let file = File::open(&segment_header_path).map_err(|e| {
            Error::InvalidPath(format!(
                "Failed to open segment header '{}': {}",
                segment_header_path.display(),
                e
            ))
        })?;

        let mut reader = BufReader::new(file);
        let header = Header::from_reader(&mut reader)?;

        // Validate segment
        if header.specifications.is_multi_segment() {
            return Err(Error::InvalidHeader(
                "Nested multi-segment records are not supported".to_string(),
            ));
        }

        let segment_base_path = segment_header_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        let data = SegmentData {
            header,
            base_path: segment_base_path,
        };

        self.states[index] = SegmentState::Loaded(Box::new(data));

        match &self.states[index] {
            SegmentState::Loaded(data) => Ok(data),
            _ => unreachable!(),
        }
    }

    /// Get current segment data.
    pub fn current_segment(&mut self) -> Result<&SegmentData> {
        self.load_segment(self.current_segment)
    }

    /// Get signals for current segment.
    pub fn current_signals(&mut self) -> Result<&[SignalInfo]> {
        let data = self.current_segment()?;
        data.header
            .specifications
            .signals()
            .ok_or_else(|| Error::InvalidHeader("Segment has no signals".to_string()))
    }

    /// Get base path for current segment.
    pub fn current_base_path(&mut self) -> Result<&Path> {
        let data = self.current_segment()?;
        Ok(&data.base_path)
    }

    /// Check if current segment is null.
    #[allow(dead_code)]
    pub fn is_current_null(&self) -> bool {
        matches!(self.states[self.current_segment], SegmentState::Null)
    }

    /// Get current segment index.
    #[must_use]
    pub const fn current_index(&self) -> usize {
        self.current_segment
    }

    /// Get total number of segments.
    #[must_use]
    pub const fn num_segments(&self) -> usize {
        self.segments.len()
    }

    /// Get segment info.
    #[must_use]
    pub fn segment_info(&self, index: usize) -> Option<&SegmentInfo> {
        self.segments.get(index)
    }
}
