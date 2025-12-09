use crate::signal::common::{FormatDecoder, INVALID_SAMPLE};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 8 (8-bit first differences).
///
/// This format stores first differences as signed 8-bit integers.
/// The actual sample value is computed by accumulating differences.
///
/// When differences exceed the representable range (-128 to +127), the maximum
/// difference is stored and subsequent differences adjust to reach the target.
#[derive(Debug, Clone)]
pub struct Format8Decoder {
    /// Current accumulated sample value
    current_value: Sample,
}

impl Format8Decoder {
    /// Create a new Format 8 decoder with the specified initial value.
    #[must_use]
    pub const fn new(initial_value: Sample) -> Self {
        Self {
            current_value: initial_value,
        }
    }
}

impl FormatDecoder for Format8Decoder {
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        let mut count = 0;
        let mut buf = [0u8; 1];

        for sample in output.iter_mut() {
            match reader.read_exact(&mut buf) {
                Ok(()) => {
                    // Read signed 8-bit difference
                    let diff = i8::from_le_bytes(buf);

                    // Accumulate the difference
                    self.current_value = self.current_value.saturating_add(i32::from(diff));

                    // Check for invalid sample marker (not typically used in format 8)
                    if diff == i8::MIN && self.current_value == i32::from(i8::MIN) {
                        *sample = INVALID_SAMPLE;
                    } else {
                        *sample = self.current_value;
                    }

                    count += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(count)
    }

    fn reset(&mut self) {
        self.current_value = 0;
    }

    fn bytes_per_sample(&self) -> Option<usize> {
        Some(1)
    }
}
