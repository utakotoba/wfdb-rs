use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::signal::FormatDecoder;
use crate::{Error, Result, Sample, SignalInfo};

/// Reader for a single signal with three-level API.
///
/// Provides three ways to read signal data:
/// 1. **Buffer API** - `read_samples_buf()` for zero-copy performance
/// 2. **Ergonomic API** - `read_samples()` for simple Vec-based reading
/// 3. **Iterator API** - `samples()` for lazy evaluation and composition
///
/// # Interleaved Signals
///
/// When multiple signals share the same file, they are stored interleaved
/// (one frame = one sample from each signal). This reader automatically
/// handles de-interleaving by reading entire frames and extracting only
/// the requested signal's samples.
///
/// # Examples
///
/// ## Buffer API (highest performance)
///
/// ```no_run
/// use wfdb::Record;
///
/// # fn main() -> wfdb::Result<()> {
/// let record = Record::open("data/100")?;
/// let mut reader = record.signal_reader(0)?;
///
/// let mut buffer = vec![0; 1000];
/// loop {
///     let n = reader.read_samples_buf(&mut buffer)?;
///     if n == 0 { break; }
///     // Process samples in buffer[..n]
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Ergonomic API (simplest)
///
/// ```no_run
/// use wfdb::Record;
///
/// # fn main() -> wfdb::Result<()> {
/// let record = Record::open("data/100")?;
/// let mut reader = record.signal_reader(0)?;
///
/// // Read ADC values
/// let adc_values = reader.read_samples(1000)?;
///
/// // Read physical values (mV)
/// let physical_values = reader.read_physical(1000)?;
/// # Ok(())
/// # }
/// ```
///
/// ## Iterator API (lazy evaluation)
///
/// ```no_run
/// use wfdb::Record;
///
/// # fn main() -> wfdb::Result<()> {
/// let record = Record::open("data/100")?;
/// let mut reader = record.signal_reader(0)?;
///
/// // Collect first 1000 samples
/// let samples: Vec<_> = reader
///     .samples()
///     .take(1000)
///     .collect::<wfdb::Result<Vec<_>>>()?;
///
/// // Process with iterator combinators
/// let max = reader
///     .samples()
///     .filter_map(Result::ok)
///     .take(1000)
///     .max();
/// # Ok(())
/// # }
/// ```
pub struct SignalReader {
    /// Format decoder for this signal.
    decoder: Box<dyn FormatDecoder>,
    /// Buffered reader for the signal file.
    reader: BufReader<File>,
    /// Signal information (for physical units conversion).
    signal_info: SignalInfo,
    /// Index of this signal within its file group (for interleaved reading).
    signal_index_in_file: usize,
    /// Total number of signals sharing this file (for interleaved reading).
    signals_in_file: usize,
    /// Buffer for reading interleaved frames.
    frame_buffer: Vec<Sample>,
    /// Current sample position (for interleaved seeking).
    current_sample: u64,
    /// Bytes per sample for this format (for seeking).
    bytes_per_sample: usize,
    /// Initial file offset (to calculate absolute positions).
    initial_offset: u64,
}

impl SignalReader {
    /// Create a new signal reader.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Signal file cannot be opened
    /// - Signal format is not supported
    pub(crate) fn new(
        base_path: &Path,
        signal_info: &SignalInfo,
        all_signals: &[SignalInfo],
        signal_index: usize,
    ) -> Result<Self> {
        // Resolve signal file path
        let signal_path = base_path.join(&signal_info.file_name);

        // Open signal file
        let file = File::open(&signal_path).map_err(|e| {
            Error::InvalidPath(format!(
                "Failed to open signal file '{}': {}",
                signal_path.display(),
                e
            ))
        })?;

        let mut reader = BufReader::new(file);

        // Create decoder for this signal's format
        let initial_value = signal_info.initial_value.unwrap_or(0);
        let decoder = crate::signal::get_decoder(signal_info.format, initial_value)?;

        // Get bytes per sample for seeking
        let bytes_per_sample = decoder.bytes_per_sample().unwrap_or(0);

        // Handle byte offset if specified
        let initial_offset = signal_info.byte_offset.unwrap_or(0);
        if initial_offset > 0 {
            use std::io::Seek;
            reader.seek(std::io::SeekFrom::Start(initial_offset))?;
        }

        // Determine interleaving: count how many signals share this file
        let mut signal_index_in_file = 0;
        let mut signals_in_file = 0;
        for (idx, sig) in all_signals.iter().enumerate() {
            if sig.file_name == signal_info.file_name && sig.format == signal_info.format {
                if idx == signal_index {
                    signal_index_in_file = signals_in_file;
                }
                signals_in_file += 1;
            }
        }

        // Create frame buffer if signals are interleaved
        let frame_buffer = if signals_in_file > 1 {
            vec![0; signals_in_file]
        } else {
            Vec::new()
        };

        Ok(Self {
            decoder,
            reader,
            signal_info: signal_info.clone(),
            signal_index_in_file,
            signals_in_file,
            frame_buffer,
            current_sample: 0,
            bytes_per_sample,
            initial_offset,
        })
    }

    // [Raw ADC value reading]

    /// Read samples into a provided buffer (raw ADC values).
    ///
    /// This is the zero-copy buffer API. The caller provides the buffer,
    /// and this method fills it with as many samples as possible.
    ///
    /// For interleaved signals (multiple signals sharing one file), this
    /// automatically reads frames and extracts only this signal's samples.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the signal file fails.
    pub fn read_samples_buf(&mut self, buffer: &mut [Sample]) -> Result<usize> {
        if self.signals_in_file <= 1 {
            // Non-interleaved: read directly
            let count = self.decoder.decode_buf(&mut self.reader, buffer)?;
            self.current_sample += count as u64;
            Ok(count)
        } else if self.bytes_per_sample == 0 {
            // Interleaved with stateful format (e.g., Format212, Format310, Format311)
            // These formats pack multiple samples into non-aligned byte sequences
            // WARNING: This only works correctly if all signal readers for this file
            // are created fresh or properly coordinated. For best results, use
            // MultiSignalReader for interleaved stateful formats.

            use std::io::Seek;

            let mut count = 0;
            for sample in buffer.iter_mut() {
                // Reset decoder state before reading frame to ensure consistency
                self.decoder.reset();

                // Calculate byte position for this frame
                let frame_number = self.current_sample;

                // Use the decoder's bytes_per_frame method (format-specific logic)
                let bytes_per_frame = self.decoder.bytes_per_frame(self.signals_in_file)
                    .ok_or_else(|| Error::InvalidHeader(
                        "Format does not support frame size calculation for interleaved reading".to_string()
                    ))?;

                let byte_offset = self.initial_offset + frame_number * bytes_per_frame as u64;
                self.reader.seek(std::io::SeekFrom::Start(byte_offset))?;

                // Read one frame sequentially
                let n = self
                    .decoder
                    .decode_buf(&mut self.reader, &mut self.frame_buffer)?;
                if n == 0 {
                    break; // EOF
                }
                if n < self.signals_in_file {
                    // Incomplete frame - shouldn't happen
                    return Err(Error::InvalidHeader(
                        "Incomplete frame in interleaved signal file".to_string(),
                    ));
                }

                // Extract this signal's sample from the frame
                *sample = self.frame_buffer[self.signal_index_in_file];
                self.current_sample += 1;
                count += 1;
            }
            Ok(count)
        } else {
            // Interleaved with fixed-size format - can seek for each frame
            use std::io::Seek;

            let mut count = 0;
            for sample in buffer.iter_mut() {
                // Calculate byte position for this frame
                // Each frame contains signals_in_file samples
                let frame_number = self.current_sample;
                let byte_offset = self.initial_offset
                    + frame_number * (self.signals_in_file * self.bytes_per_sample) as u64;

                // Seek to the frame position
                self.reader.seek(std::io::SeekFrom::Start(byte_offset))?;

                // Read one frame
                let n = self
                    .decoder
                    .decode_buf(&mut self.reader, &mut self.frame_buffer)?;
                if n == 0 {
                    break; // EOF
                }
                if n < self.signals_in_file {
                    // Incomplete frame - shouldn't happen
                    return Err(Error::InvalidHeader(
                        "Incomplete frame in interleaved signal file".to_string(),
                    ));
                }

                // Extract this signal's sample from the frame
                *sample = self.frame_buffer[self.signal_index_in_file];
                self.current_sample += 1;
                count += 1;
            }
            Ok(count)
        }
    }

    /// Read a specified number of samples (raw ADC values).
    ///
    /// This is the ergonomic API that returns a `Vec<Sample>`.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the signal file fails.
    pub fn read_samples(&mut self, count: usize) -> Result<Vec<Sample>> {
        let mut buffer = vec![0; count];
        let n = self.read_samples_buf(&mut buffer)?;
        buffer.truncate(n);
        Ok(buffer)
    }

    // [Physical units reading]

    /// Read samples into a provided buffer (physical values).
    ///
    /// Converts raw ADC values to physical units using the signal's gain
    /// and baseline parameters.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer to fill with physical values
    ///
    /// # Returns
    ///
    /// The number of samples actually read (may be less than buffer length
    /// if end of file is reached).
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the signal file fails.
    pub fn read_physical_buf(&mut self, buffer: &mut [f64]) -> Result<usize> {
        // Use a temporary buffer for ADC values
        let mut adc_buffer = vec![0i32; buffer.len()];
        let n = self.read_samples_buf(&mut adc_buffer)?;

        // Convert ADC values to physical values
        for i in 0..n {
            buffer[i] = self.to_physical(adc_buffer[i]);
        }

        Ok(n)
    }

    /// Read a specified number of samples (physical values).
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the signal file fails.
    pub fn read_physical(&mut self, count: usize) -> Result<Vec<f64>> {
        let adc_values = self.read_samples(count)?;
        Ok(adc_values.iter().map(|&v| self.to_physical(v)).collect())
    }

    // [Conversion utilities]

    /// Convert an ADC value to physical units.
    #[must_use]
    pub fn to_physical(&self, adc_value: Sample) -> f64 {
        let baseline = f64::from(self.signal_info.baseline());
        let gain = self.signal_info.adc_gain();
        (f64::from(adc_value) - baseline) / gain
    }

    /// Convert a physical value to ADC units.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn to_adc(&self, physical_value: f64) -> Sample {
        let baseline = f64::from(self.signal_info.baseline());
        let gain = self.signal_info.adc_gain();
        physical_value.mul_add(gain, baseline).round() as Sample
    }

    // [Iterator API]

    /// Create an iterator over samples from this signal.
    ///
    /// Returns an iterator that lazily reads samples one at a time.
    /// This is useful for processing samples with Rust's iterators
    /// (`map`, `filter`, `take`, `collect`, etc.).
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
    /// // Collect first 1000 positive samples
    /// let positive: Vec<_> = reader
    ///     .samples()
    ///     .take(10000)
    ///     .filter_map(Result::ok)
    ///     .filter(|&s| s > 0)
    ///     .take(1000)
    ///     .collect();
    /// # Ok(())
    /// # }
    /// ```
    pub const fn samples(&mut self) -> SampleIterator<'_> {
        SampleIterator {
            reader: self,
            buffer: [0; 1],
            done: false,
        }
    }

    // [Accessors]

    /// Get the signal information for this reader.
    #[must_use]
    pub const fn signal_info(&self) -> &SignalInfo {
        &self.signal_info
    }

    /// Get the signal description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.signal_info.description.as_deref()
    }

    /// Get the physical units for this signal.
    #[must_use]
    pub fn units(&self) -> &str {
        self.signal_info.units()
    }

    /// Get the ADC gain (ADC units per physical unit).
    #[must_use]
    pub fn gain(&self) -> f64 {
        self.signal_info.adc_gain()
    }

    /// Get the baseline (ADC value at 0 physical units).
    #[must_use]
    pub fn baseline(&self) -> i32 {
        self.signal_info.baseline()
    }

    // [Seeking support]

    /// Seek to a specific sample number (0-indexed).
    ///
    /// Returns the actual sample position after seeking.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Seeking is not supported for this format
    /// - The seek operation fails
    ///
    /// # Performance Note
    ///
    /// For interleaved signals, seeking requires calculating frame boundaries.
    /// For differential formats (Format 8), seeking resets the decoder state.
    pub fn seek_to_sample(&mut self, sample: u64) -> Result<u64> {
        use std::io::Seek;

        if self.signals_in_file <= 1 {
            // Non-interleaved: calculate byte position directly
            if self.bytes_per_sample > 0 {
                let byte_offset = self.initial_offset + sample * self.bytes_per_sample as u64;
                self.reader.seek(std::io::SeekFrom::Start(byte_offset))?;
                self.decoder.reset();
                self.current_sample = sample;
                Ok(sample)
            } else {
                Err(Error::InvalidHeader(
                    "Seeking not supported for this signal format".to_string(),
                ))
            }
        } else {
            // Interleaved: seek to frame containing the sample
            if self.bytes_per_sample == 0 {
                // Stateful format - need bytes_per_frame
                let bytes_per_frame = self
                    .decoder
                    .bytes_per_frame(self.signals_in_file)
                    .ok_or_else(|| {
                        Error::InvalidHeader(
                            "Seeking not supported for this signal format".to_string(),
                        )
                    })?;
                let byte_offset = self.initial_offset + sample * bytes_per_frame as u64;
                self.reader.seek(std::io::SeekFrom::Start(byte_offset))?;
            } else {
                // Fixed-size format
                let byte_offset = self.initial_offset
                    + sample * (self.signals_in_file * self.bytes_per_sample) as u64;
                self.reader.seek(std::io::SeekFrom::Start(byte_offset))?;
            }
            self.decoder.reset();
            self.current_sample = sample;
            Ok(sample)
        }
    }

    /// Get current sample position.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.current_sample
    }

    /// Seek to a specific time in the record.
    ///
    /// Converts the time to a sample number using the signal's sampling frequency
    /// and then seeks to that sample.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Seeking is not supported for this format
    /// - The seek operation fails
    ///
    /// # Note
    ///
    /// This method requires access to the record's sampling frequency,
    /// which should be stored in the signal info. Currently this is a placeholder.
    #[allow(dead_code)]
    pub fn seek_to_time(&mut self, _seconds: f64) -> Result<u64> {
        // TODO: Implement when we have access to sampling frequency
        // let sample = (seconds * sampling_frequency).round() as u64;
        // self.seek_to_sample(sample)
        Err(Error::InvalidHeader(
            "Time-based seeking not yet implemented".to_string(),
        ))
    }
}

/// Iterator over samples from a `SignalReader`.
///
/// Created by calling [`SignalReader::samples()`].
///
/// # Performance
///
/// For non-interleaved signals, this iterator efficiently reads samples
/// sequentially. For interleaved signals, each call to `next()` may involve
/// seeking and reading a full frame, which can be slower.
pub struct SampleIterator<'a> {
    reader: &'a mut SignalReader,
    buffer: [Sample; 1],
    done: bool,
}

impl Iterator for SampleIterator<'_> {
    type Item = Result<Sample>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.reader.read_samples_buf(&mut self.buffer) {
            Ok(0) => {
                self.done = true;
                None
            }
            Ok(_) => Some(Ok(self.buffer[0])),
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
        }
    }
}
