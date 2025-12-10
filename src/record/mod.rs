//! High-level API for reading WFDB records.
//!
//! This module provides the primary interface for working with WFDB records,
//! combining header parsing with signal reading in an ergonomic, modern Rust API.
//!
//! # Examples
//!
//! ## Reading a single signal
//!
//! ```no_run
//! use wfdb::Record;
//!
//! # fn main() -> wfdb::Result<()> {
//! // Open record (loads and parses header)
//! let record = Record::open("data/100")?;
//!
//! // Access header information
//! println!("Record: {}", record.metadata().name());
//! println!("Signals: {}", record.signal_count());
//! println!("Sampling frequency: {} Hz", record.metadata().sampling_frequency());
//!
//! // Create reader for first signal
//! let mut reader = record.signal_reader(0)?;
//!
//! // Read 1000 samples as raw ADC values
//! let adc_samples = reader.read_samples(1000)?;
//!
//! // Read 1000 samples as physical values (mV)
//! let physical_samples = reader.read_physical(1000)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Reading multiple signals together (frames)
//!
//! ```no_run
//! use wfdb::Record;
//!
//! # fn main() -> wfdb::Result<()> {
//! let record = Record::open("data/100")?;
//! let mut reader = record.multi_signal_reader()?;
//!
//! // Read 1000 frames (each frame contains one sample per signal)
//! for _ in 0..1000 {
//!     let frame = reader.read_frame()?;
//!     // frame is Vec<Sample>, one per signal
//!     println!("Signal 0: {}, Signal 1: {}", frame[0], frame[1]);
//! }
//! # Ok(())
//! # }
//! ```

mod multi_signal_reader;
pub(crate) mod segment;
mod segment_reader;
mod signal_reader;

pub use multi_signal_reader::MultiSignalReader;
pub use segment_reader::SegmentReader;
pub use signal_reader::SignalReader;

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::{Error, Header, Metadata, Result, SegmentInfo, SignalInfo};

/// High-level API for working with WFDB records.
///
/// The `Record` itself is lightweight—it only holds the parsed header and
/// base path. Signal files are opened lazily when readers are created.
///
/// # Examples
///
/// ```no_run
/// use wfdb::Record;
///
/// # fn main() -> wfdb::Result<()> {
/// let record = Record::open("data/100")?;
///
/// // Check if multi-segment
/// if record.is_multi_segment() {
///     println!("Multi-segment record with {} segments", record.segment_count());
/// } else {
///     println!("Single-segment record with {} signals", record.signal_count());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Record {
    /// Parsed header containing metadata and specifications.
    header: Header,
    /// Base directory path for resolving signal files.
    base_path: PathBuf,
}

impl Record {
    // [Constructors]

    /// Open a WFDB record from a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The header file is not found
    /// - The header cannot be parsed
    /// - The header file cannot be opened
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Resolve header file path (add .hea if not present)
        let header_path = if path.extension().is_some_and(|ext| ext == "hea") {
            path.to_path_buf()
        } else {
            path.with_extension("hea")
        };

        // Verify header file exists
        if !header_path.exists() {
            return Err(Error::InvalidPath(format!(
                "Header file not found: {}",
                header_path.display()
            )));
        }

        // Open and parse header
        let file = File::open(&header_path)?;
        let mut reader = BufReader::new(file);
        let header = Header::from_reader(&mut reader)?;

        // Store base directory for resolving signal files
        let base_path = header_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        Ok(Self { header, base_path })
    }

    /// Create a Record from a parsed header and base path.
    ///
    /// This is primarily for testing purposes.
    #[must_use]
    pub const fn from_header(header: Header, base_path: PathBuf) -> Self {
        Self { header, base_path }
    }

    // [Accessors]

    /// Get the record metadata.
    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.header.metadata
    }

    /// Get the header specifications (signals or segments).
    #[must_use]
    pub const fn specifications(&self) -> &crate::header::Specifications {
        &self.header.specifications
    }

    /// Get signal specifications for single-segment records.
    ///
    /// Returns `None` for multi-segment records.
    #[must_use]
    pub fn signal_info(&self) -> Option<&[SignalInfo]> {
        self.header.specifications.signals()
    }

    /// Get segment specifications for multi-segment records.
    ///
    /// Returns `None` for single-segment records.
    #[must_use]
    pub fn segment_info(&self) -> Option<&[SegmentInfo]> {
        self.header.specifications.segments()
    }

    /// Get info strings (comments) from the header.
    #[must_use]
    pub fn info_strings(&self) -> &[String] {
        &self.header.info_strings
    }

    /// Check if this is a multi-segment record.
    #[must_use]
    pub const fn is_multi_segment(&self) -> bool {
        self.header.specifications.is_multi_segment()
    }

    /// Get the number of signals (for single-segment records).
    ///
    /// Returns 0 for multi-segment records (use `segment_count()` instead).
    #[must_use]
    pub fn signal_count(&self) -> usize {
        self.signal_info().map_or(0, <[SignalInfo]>::len)
    }

    /// Get the number of segments (for multi-segment records).
    ///
    /// Returns 0 for single-segment records (use `signal_count()` instead).
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segment_info().map_or(0, <[SegmentInfo]>::len)
    }

    /// Get the base directory path for this record.
    ///
    /// This is used internally to resolve signal file paths.
    #[must_use]
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    // [Reader creation methods]

    /// Create a reader for a single signal.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - This is a multi-segment record (not yet supported for single signal readers)
    /// - The signal index is out of bounds
    /// - The signal file cannot be opened
    /// - The signal format is not supported
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wfdb::Record;
    ///
    /// # fn main() -> wfdb::Result<()> {
    /// let record = Record::open("data/100")?;
    /// let mut reader = record.signal_reader(0)?;
    ///
    /// // Read samples as raw ADC values
    /// let adc_values = reader.read_samples(1000)?;
    ///
    /// // Read samples as physical values (mV)
    /// let physical_values = reader.read_physical(1000)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn signal_reader(&self, signal_index: usize) -> Result<SignalReader> {
        if self.is_multi_segment() {
            return Err(Error::InvalidHeader(
                "Single signal readers not yet supported for multi-segment records".to_string(),
            ));
        }

        let signals = self.signal_info().ok_or_else(|| {
            Error::InvalidHeader("No signal specifications in header".to_string())
        })?;

        if signal_index >= signals.len() {
            return Err(Error::InvalidHeader(format!(
                "Signal index {} out of bounds (record has {} signals)",
                signal_index,
                signals.len()
            )));
        }

        let sampling_frequency = Some(self.metadata().sampling_frequency());

        SignalReader::new(
            &self.base_path,
            &signals[signal_index],
            signals,
            signal_index,
            sampling_frequency,
        )
    }

    /// Create a reader for all signals (frame-based reading).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - This is a multi-segment record (not yet supported)
    /// - Signal files cannot be opened
    /// - Signal formats are not supported
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wfdb::Record;
    ///
    /// # fn main() -> wfdb::Result<()> {
    /// let record = Record::open("data/100")?;
    /// let mut reader = record.multi_signal_reader()?;
    ///
    /// // Read 1000 frames (one sample per signal per frame)
    /// for _ in 0..1000 {
    ///     let frame = reader.read_frame()?;
    ///     // Process frame...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn multi_signal_reader(&self) -> Result<MultiSignalReader> {
        if self.is_multi_segment() {
            return Err(Error::InvalidHeader(
                "Multi-signal readers not yet supported for multi-segment records".to_string(),
            ));
        }

        let signals = self.signal_info().ok_or_else(|| {
            Error::InvalidHeader("No signal specifications in header".to_string())
        })?;

        MultiSignalReader::new(&self.base_path, signals)
    }

    /// Create a reader for multi-segment records.
    ///
    /// This reader handles segment switching and provides unified access
    /// to signals across all segments.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - This is not a multi-segment record
    /// - Segment headers cannot be loaded
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wfdb::Record;
    ///
    /// # fn main() -> wfdb::Result<()> {
    /// let record = Record::open("data/multi_segment_record")?;
    /// let mut reader = record.segment_reader()?;
    ///
    /// // Read frames across all segments
    /// while let Some(frame) = reader.read_frame()? {
    ///     // Process frame...
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn segment_reader(&self) -> Result<SegmentReader> {
        if !self.is_multi_segment() {
            return Err(Error::InvalidHeader(
                "Segment readers only supported for multi-segment records".to_string(),
            ));
        }

        let segments = self.segment_info().ok_or_else(|| {
            Error::InvalidHeader("No segment specifications in header".to_string())
        })?;

        Ok(SegmentReader::new(
            self.base_path.clone(),
            segments.to_vec(),
        ))
    }
}
