use crate::signal::common::{FormatDecoder, INVALID_SAMPLE};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 16 (16-bit two's complement, little-endian).
///
/// Each sample occupies 2 bytes stored in little-endian byte order.
/// The value 0x8000 (-32768) is reserved to indicate an invalid sample.
#[derive(Debug, Clone, Default)]
pub struct Format16Decoder;

impl Format16Decoder {
    /// Create a new Format 16 decoder.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl FormatDecoder for Format16Decoder {
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        let mut count = 0;
        let mut buf = [0u8; 2];

        for sample in output.iter_mut() {
            match reader.read_exact(&mut buf) {
                Ok(()) => {
                    // Little-endian: LSB first
                    let value = i16::from_le_bytes(buf);

                    // Check for invalid sample marker
                    if value == i16::MIN {
                        *sample = INVALID_SAMPLE;
                    } else {
                        *sample = i32::from(value);
                    }
                    count += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // End of stream reached
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(count)
    }

    fn reset(&mut self) {
        // No state to reset
    }

    fn bytes_per_sample(&self) -> Option<usize> {
        Some(2)
    }
}
