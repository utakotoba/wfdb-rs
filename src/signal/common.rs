//! Common traits and types for signal format decoders and encoders.

use crate::{Result, Sample};
use std::io::BufRead;

/// Invalid sample marker used by WFDB library.
///
/// This value indicates that a sample is missing or invalid.
/// It is typically the most negative value that can be represented.
pub const INVALID_SAMPLE: Sample = Sample::MIN;

/// Configuration for format decoders.
///
/// Parameters needed by format decoders that may vary per signal.
#[derive(Debug, Clone)]
pub struct DecoderConfig {
    /// Initial sample value (used for differential formats like Format 8).
    pub initial_value: Sample,
    /// Number of samples per frame (for multiplexed formats).
    pub samples_per_frame: usize,
    /// Byte offset from the beginning of the file to sample 0.
    pub byte_offset: u64,
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self {
            initial_value: 0,
            samples_per_frame: 1,
            byte_offset: 0,
        }
    }
}

/// Trait for decoding WFDB signal data from a byte stream.
///
/// Format decoders read raw bytes from a `BufRead` source and convert them
/// to `Sample` values according to the WFDB format specification.
pub trait FormatDecoder: Send {
    /// Decode samples into a caller-provided buffer (low-level, zero-copy).
    ///
    /// Reads raw bytes from `reader` and writes decoded samples to `output`.
    /// Returns the number of samples successfully decoded.
    ///
    /// This is the low-level API for performance-critical code that needs
    /// to manage its own allocations. Most code should use [`decode()`](FormatDecoder::decode) instead.
    ///
    /// # Errors
    ///
    /// Returns an error if the input data is malformed or I/O fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wfdb::signal::{Format16Decoder, FormatDecoder};
    /// # use std::io::BufReader;
    /// # use std::fs::File;
    /// # fn main() -> wfdb::Result<()> {
    /// let mut decoder = Format16Decoder::new();
    /// let file = File::open("data.dat")?;
    /// let mut reader = BufReader::new(file);
    ///
    /// // Reuse buffer in a loop for performance
    /// let mut buffer = vec![0; 1000];
    /// loop {
    ///     let n = decoder.decode_buf(&mut reader, &mut buffer)?;
    ///     if n == 0 { break; }
    ///     // Process samples in buffer[..n]
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize>;

    /// Decode samples and return them as an owned `Vec` (high-level, ergonomic).
    ///
    /// This is the recommended API for most use cases. It allocates a new `Vec`
    /// and returns the decoded samples.
    ///
    /// # Errors
    ///
    /// Returns an error if the input data is malformed or I/O fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wfdb::signal::{Format16Decoder, FormatDecoder};
    /// # use std::io::BufReader;
    /// # use std::fs::File;
    /// # fn main() -> wfdb::Result<()> {
    /// let mut decoder = Format16Decoder::new();
    /// let file = File::open("data.dat")?;
    /// let mut reader = BufReader::new(file);
    ///
    /// // Simple and ergonomic
    /// let samples = decoder.decode(&mut reader, 1000)?;
    /// println!("Decoded {} samples", samples.len());
    /// # Ok(())
    /// # }
    /// ```
    fn decode(&mut self, reader: &mut dyn BufRead, count: usize) -> Result<Vec<Sample>> {
        let mut output = vec![0; count];
        let n = self.decode_buf(reader, &mut output)?;
        output.truncate(n);
        Ok(output)
    }

    /// Reset the decoder to its initial state.
    ///
    /// This should clear any internal buffers or state. Useful when seeking
    /// to a new position in the input stream.
    fn reset(&mut self);

    /// Get the number of bytes required to decode one sample.
    ///
    /// Returns `None` for variable-size formats or formats where the size
    /// depends on the number of signals being multiplexed.
    fn bytes_per_sample(&self) -> Option<usize> {
        None
    }

    /// Get the number of bytes required for one frame of interleaved signals.
    ///
    /// A frame contains one sample from each of `num_signals` signals stored
    /// in an interleaved format. This is used for seeking in files with
    /// multiple interleaved signals.
    ///
    /// Default implementation works for fixed-size formats by multiplying
    /// `bytes_per_sample()` by `num_signals`. Variable-size formats should
    /// override this method.
    fn bytes_per_frame(&self, num_signals: usize) -> Option<usize> {
        self.bytes_per_sample().map(|bps| bps * num_signals)
    }

    /// Create an iterator over samples from this decoder (most flexible API).
    ///
    /// Returns an iterator that lazily decodes samples one at a time from the reader.
    /// This is the most flexible API, allowing integration with Rust's iterator
    /// ecosystem (map, filter, take, collect, etc.).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wfdb::signal::{Format16Decoder, FormatDecoder};
    /// # use std::io::BufReader;
    /// # use std::fs::File;
    /// # fn main() -> wfdb::Result<()> {
    /// let mut decoder = Format16Decoder::new();
    /// let file = File::open("data.dat")?;
    /// let reader = BufReader::new(file);
    ///
    /// // Use iterator combinators
    /// let positive_samples: Vec<_> = decoder.samples(reader)
    ///     .take(1000)
    ///     .filter_map(|r| r.ok())
    ///     .filter(|&s| s > 0)
    ///     .collect();
    /// # Ok(())
    /// # }
    /// ```
    fn samples<R: BufRead>(&mut self, reader: R) -> SampleIter<'_, Self, R>
    where
        Self: Sized,
    {
        SampleIter::new(self, reader)
    }
}

/// Sign-extend a value from a specific bit position.
///
/// # Examples
///
/// ```
/// # use wfdb::signal::sign_extend;
/// // 12-bit value 0x7FF (2047) should remain positive
/// assert_eq!(sign_extend(0x7FF, 12), 2047);
///
/// // 12-bit value 0x800 (-2048 in 12-bit two's complement)
/// assert_eq!(sign_extend(0x800, 12), -2048);
/// ```
#[inline]
#[must_use]
#[allow(clippy::cast_possible_wrap)]
pub const fn sign_extend(value: u32, bits: u32) -> i32 {
    let sign_bit = 1 << (bits - 1);
    if value & sign_bit != 0 {
        // Negative: set all bits above the sign bit
        (value | !((1 << bits) - 1)) as i32
    } else {
        // Positive: mask to the specified number of bits
        (value & ((1 << bits) - 1)) as i32
    }
}

/// Iterator over samples from a format decoder.
///
/// # Examples
///
/// ```no_run
/// # use wfdb::signal::{Format16Decoder, FormatDecoder};
/// # use std::io::BufReader;
/// # use std::fs::File;
/// # fn main() -> wfdb::Result<()> {
/// let mut decoder = Format16Decoder::new();
/// let file = File::open("data.dat")?;
/// let reader = BufReader::new(file);
///
/// for sample in decoder.samples(reader).take(10) {
///     let sample = sample?;
///     println!("Sample: {}", sample);
/// }
/// # Ok(())
/// # }
/// ```
pub struct SampleIter<'a, D, R>
where
    D: FormatDecoder + ?Sized,
    R: BufRead,
{
    decoder: &'a mut D,
    reader: R,
    buffer: [Sample; 1],
    done: bool,
}

impl<'a, D, R> SampleIter<'a, D, R>
where
    D: FormatDecoder + ?Sized,
    R: BufRead,
{
    /// Create a new sample iterator.
    pub const fn new(decoder: &'a mut D, reader: R) -> Self {
        Self {
            decoder,
            reader,
            buffer: [0],
            done: false,
        }
    }
}

impl<D, R> Iterator for SampleIter<'_, D, R>
where
    D: FormatDecoder + ?Sized,
    R: BufRead,
{
    type Item = Result<Sample>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.decoder.decode_buf(&mut self.reader, &mut self.buffer) {
            Ok(0) => {
                self.done = true;
                None // End of stream
            }
            Ok(_) => Some(Ok(self.buffer[0])),
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
        }
    }
}
