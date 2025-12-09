use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::signal::FormatDecoder;
use crate::{Error, Result, Sample, SignalInfo};

/// Signal group - signals that share the same file.
struct SignalGroup {
    /// Format decoder for this group.
    decoder: Box<dyn FormatDecoder>,
    /// Buffered reader for the signal file.
    reader: BufReader<File>,
    /// Indices of signals in this group (into the original signals array).
    signal_indices: Vec<usize>,
    /// Signal info for each signal in this group.
    signal_infos: Vec<SignalInfo>,
}

/// Reader for multiple signals (frame-based).
///
/// Reads one frame at a time, where each frame contains one sample from each signal.
/// Handles signals in different files and with different formats.
pub struct MultiSignalReader {
    /// Signal groups (one per unique file).
    groups: Vec<SignalGroup>,
    /// Total number of signals.
    num_signals: usize,
    /// Mapping from signal index to (`group_index`, `index_within_group`).
    signal_to_group: Vec<(usize, usize)>,
    /// Current frame position.
    current_frame: u64,
}

impl MultiSignalReader {
    /// Create a new multi-signal reader.
    pub(crate) fn new(base_path: &Path, signals: &[SignalInfo]) -> Result<Self> {
        if signals.is_empty() {
            return Err(Error::InvalidHeader("No signals to read".to_string()));
        }

        // Group signals by file name
        let mut file_groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, signal) in signals.iter().enumerate() {
            file_groups
                .entry(signal.file_name.clone())
                .or_default()
                .push(idx);
        }

        // Create signal groups
        let mut groups = Vec::new();
        let mut signal_to_group = vec![(0, 0); signals.len()];

        for (file_name, signal_indices) in file_groups {
            let group_index = groups.len();

            // Get first signal in group for decoder setup
            let first_signal = &signals[signal_indices[0]];

            // Open signal file
            let signal_path = base_path.join(&file_name);
            let file = File::open(&signal_path).map_err(|e| {
                Error::InvalidPath(format!(
                    "Failed to open signal file '{}': {}",
                    signal_path.display(),
                    e
                ))
            })?;

            let mut reader = BufReader::new(file);

            // Handle byte offset if specified
            if let Some(offset) = first_signal.byte_offset {
                use std::io::Seek;
                reader.seek(std::io::SeekFrom::Start(offset))?;
            }

            // Create decoder
            let initial_value = first_signal.initial_value.unwrap_or(0);
            let decoder = crate::signal::get_decoder(first_signal.format, initial_value)?;

            // Collect signal infos for this group
            let signal_infos: Vec<SignalInfo> = signal_indices
                .iter()
                .map(|&idx| signals[idx].clone())
                .collect();

            // Update signal_to_group mapping
            for (within_group_idx, &signal_idx) in signal_indices.iter().enumerate() {
                signal_to_group[signal_idx] = (group_index, within_group_idx);
            }

            groups.push(SignalGroup {
                decoder,
                reader,
                signal_indices: signal_indices.clone(),
                signal_infos,
            });
        }

        Ok(Self {
            groups,
            num_signals: signals.len(),
            signal_to_group,
            current_frame: 0,
        })
    }

    /// Read one frame (one sample from each signal).
    ///
    /// Returns a vector with `num_signals` samples, ordered by signal index.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The frame cannot be read
    /// - The frame is incomplete
    pub fn read_frame(&mut self) -> Result<Vec<Sample>> {
        let mut frame = vec![0; self.num_signals];

        // Read from each group
        for group in &mut self.groups {
            let mut group_samples = vec![0; group.signal_indices.len()];
            let n = group
                .decoder
                .decode_buf(&mut group.reader, &mut group_samples)?;

            if n == 0 {
                return Ok(vec![]); // EOF
            }

            if n != group.signal_indices.len() {
                return Err(Error::InvalidHeader(
                    "Incomplete frame read from signal group".to_string(),
                ));
            }

            // Place samples in correct positions
            for (within_group_idx, &signal_idx) in group.signal_indices.iter().enumerate() {
                frame[signal_idx] = group_samples[within_group_idx];
            }
        }

        self.current_frame += 1;
        Ok(frame)
    }

    /// Read multiple frames.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The frames cannot be read
    /// - The frames are incomplete
    pub fn read_frames(&mut self, count: usize) -> Result<Vec<Vec<Sample>>> {
        let mut frames = Vec::with_capacity(count);
        for _ in 0..count {
            let frame = self.read_frame()?;
            if frame.is_empty() {
                break;
            }
            frames.push(frame);
        }
        Ok(frames)
    }

    /// Read frames as physical values.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The frames cannot be read
    /// - The frames are incomplete
    pub fn read_frames_physical(&mut self, count: usize) -> Result<Vec<Vec<f64>>> {
        let frames = self.read_frames(count)?;

        // Convert each frame to physical values
        Ok(frames
            .into_iter()
            .map(|frame| self.frame_to_physical(&frame))
            .collect())
    }

    /// Convert a frame of ADC values to physical values.
    fn frame_to_physical(&self, adc_frame: &[Sample]) -> Vec<f64> {
        adc_frame
            .iter()
            .enumerate()
            .map(|(signal_idx, &adc_value)| {
                let (group_idx, within_group_idx) = self.signal_to_group[signal_idx];
                let signal_info = &self.groups[group_idx].signal_infos[within_group_idx];

                let baseline = f64::from(signal_info.baseline());
                let gain = signal_info.adc_gain();
                (f64::from(adc_value) - baseline) / gain
            })
            .collect()
    }

    /// Get number of signals.
    #[must_use]
    pub const fn num_signals(&self) -> usize {
        self.num_signals
    }

    // [Seeking support]

    /// Seek all signals to a specific frame (sample) number.
    ///
    /// All signals are positioned atomically to the same frame.
    /// Returns the actual frame position after seeking.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Seeking is not supported for any signal format
    /// - The seek operation fails
    pub fn seek_to_frame(&mut self, frame: u64) -> Result<u64> {
        use std::io::Seek;

        // Seek each group to the appropriate position
        for group in &mut self.groups {
            let num_signals = group.signal_indices.len();
            let first_signal = &group.signal_infos[0];
            let initial_offset = first_signal.byte_offset.unwrap_or(0);

            // Calculate byte position for this frame
            if let Some(bytes_per_frame) = group.decoder.bytes_per_frame(num_signals) {
                let byte_offset = initial_offset + frame * bytes_per_frame as u64;
                group.reader.seek(std::io::SeekFrom::Start(byte_offset))?;
            } else {
                return Err(Error::InvalidHeader(
                    "Seeking not supported for this signal format".to_string(),
                ));
            }

            // Reset decoder state
            group.decoder.reset();
        }

        self.current_frame = frame;
        Ok(frame)
    }

    /// Get current frame position.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.current_frame
    }
}
