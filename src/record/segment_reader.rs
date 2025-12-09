use crate::record::segment::SegmentManager;
use crate::{Error, MultiSignalReader, Result, Sample, SegmentInfo};
use std::path::PathBuf;

/// Reader for multi-segment records with seeking support.
///
/// Handles automatic segment switching and provides unified access to signals
/// across all segments in a multi-segment record.
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
pub struct SegmentReader {
    /// Segment manager for handling segment metadata and switching.
    segment_manager: SegmentManager,
    /// Current multi-signal reader (for current segment).
    current_reader: Option<MultiSignalReader>,
    /// Total samples read across all segments.
    samples_read: u64,
}

impl SegmentReader {
    /// Create a new segment reader.
    pub(crate) fn new(base_path: PathBuf, segments: Vec<SegmentInfo>) -> Self {
        let segment_manager = SegmentManager::new(base_path, segments);

        Self {
            segment_manager,
            current_reader: None,
            samples_read: 0,
        }
    }

    /// Read one frame (one sample from each signal).
    ///
    /// Returns `None` when all segments have been read.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A segment cannot be loaded
    /// - A frame cannot be read
    pub fn read_frame(&mut self) -> Result<Option<Vec<Sample>>> {
        loop {
            // Try to read from current reader
            if let Some(reader) = &mut self.current_reader {
                let frame = reader.read_frame()?;
                if !frame.is_empty() {
                    self.samples_read += 1;
                    return Ok(Some(frame));
                }
                // Current segment exhausted, move to next
            }

            // Move to next segment
            if !self.advance_segment()? {
                return Ok(None); // No more segments
            }
        }
    }

    /// Read multiple frames.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A segment cannot be loaded
    /// - Frames cannot be read
    pub fn read_frames(&mut self, count: usize) -> Result<Vec<Vec<Sample>>> {
        let mut frames = Vec::with_capacity(count);
        for _ in 0..count {
            match self.read_frame()? {
                Some(frame) => frames.push(frame),
                None => break,
            }
        }
        Ok(frames)
    }

    /// Seek to a specific sample number across all segments.
    ///
    /// Automatically switches to the appropriate segment and positions
    /// the reader at the target sample.
    ///
    /// Returns the actual sample position after seeking.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The segment containing the target sample cannot be loaded
    /// - Seeking within the segment fails
    pub fn seek_to_sample(&mut self, sample: u64) -> Result<u64> {
        // Find which segment contains this sample
        let segment_index = self.find_segment_for_sample(sample)?;

        // Calculate offset within segment
        let segment_start = if segment_index == 0 {
            0
        } else {
            self.segment_manager
                .segment_info(segment_index - 1)
                .map_or(0, |s| s.num_samples)
        };
        let offset_in_segment = sample - segment_start;

        // Switch to target segment
        self.switch_to_segment(segment_index)?;

        // Seek within segment
        if let Some(reader) = &mut self.current_reader {
            reader.seek_to_frame(offset_in_segment)?;
        }

        self.samples_read = sample;
        Ok(sample)
    }

    /// Get current sample position across all segments.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.samples_read
    }

    /// Get total number of samples across all segments.
    #[must_use]
    pub fn total_samples(&self) -> u64 {
        self.segment_manager
            .segment_info(self.segment_manager.num_segments() - 1)
            .map_or(0, |s| s.num_samples)
    }

    /// Get current segment index.
    #[must_use]
    pub const fn current_segment(&self) -> usize {
        self.segment_manager.current_index()
    }

    /// Get total number of segments.
    #[must_use]
    pub const fn num_segments(&self) -> usize {
        self.segment_manager.num_segments()
    }

    // [Private helper methods]

    /// Advance to the next segment.
    ///
    /// Returns `true` if successfully advanced, `false` if no more segments.
    fn advance_segment(&mut self) -> Result<bool> {
        let next_index = self.segment_manager.current_index() + 1;
        if next_index >= self.segment_manager.num_segments() {
            return Ok(false);
        }

        self.switch_to_segment(next_index)?;
        Ok(true)
    }

    /// Switch to a specific segment.
    fn switch_to_segment(&mut self, index: usize) -> Result<()> {
        // Load segment data
        self.segment_manager.load_segment(index)?;

        // Get signals and base path for this segment
        let signals = self.segment_manager.current_signals()?.to_vec();
        let base_path = self.segment_manager.current_base_path()?.to_path_buf();

        // Create new multi-signal reader for this segment
        let reader = MultiSignalReader::new(&base_path, &signals)?;

        self.current_reader = Some(reader);
        Ok(())
    }

    /// Find which segment contains a given sample number.
    fn find_segment_for_sample(&self, sample: u64) -> Result<usize> {
        let mut cumulative = 0u64;
        for i in 0..self.segment_manager.num_segments() {
            let segment_info = self
                .segment_manager
                .segment_info(i)
                .ok_or_else(|| Error::InvalidHeader("Segment not found".to_string()))?;

            if sample < cumulative + segment_info.num_samples {
                return Ok(i);
            }
            cumulative += segment_info.num_samples;
        }

        Err(Error::InvalidHeader(format!(
            "Sample {sample} is beyond the end of the record"
        )))
    }
}
