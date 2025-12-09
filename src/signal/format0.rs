use crate::signal::common::{FormatDecoder, INVALID_SAMPLE};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 0 (null signal).
///
/// This format represents a null signal - no data is actually read from the
/// input stream. All samples are returned as `WFDB_INVALID_SAMPLE`.
#[derive(Debug, Clone, Default)]
pub struct Format0Decoder;

impl Format0Decoder {
    /// Create a new Format 0 decoder.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl FormatDecoder for Format0Decoder {
    fn decode_buf(&mut self, _reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        // Fill output with invalid samples
        output.fill(INVALID_SAMPLE);
        Ok(output.len())
    }

    fn reset(&mut self) {
        // No state need to be reset
    }

    fn bytes_per_sample(&self) -> Option<usize> {
        Some(0) // No bytes per sample
    }
}
