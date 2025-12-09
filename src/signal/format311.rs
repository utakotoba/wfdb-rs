use crate::signal::common::{FormatDecoder, INVALID_SAMPLE, sign_extend};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 311 (packed 10-bit samples, alternative layout).
///
/// Three 10-bit samples are packed into a 32-bit word in little-endian order.
#[derive(Debug, Clone)]
pub struct Format311Decoder {
    /// Buffered 32-bit word containing 3 samples
    buffer: u32,
    /// Current position in the group (0, 1, or 2)
    position: u8,
}

impl Default for Format311Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Format311Decoder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: 0,
            position: 0,
        }
    }
}

impl FormatDecoder for Format311Decoder {
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        let mut count = 0;

        for sample in output.iter_mut() {
            if self.position == 0 {
                // Read 4 bytes as little-endian 32-bit word
                let mut buf = [0u8; 4];
                match reader.read_exact(&mut buf) {
                    Ok(()) => {
                        self.buffer = u32::from_le_bytes(buf);

                        // Sample 0: bits 0-9
                        let raw = self.buffer & 0x3FF;
                        let value = sign_extend(raw, 10);

                        *sample = if value == (-1 << 9) {
                            INVALID_SAMPLE
                        } else {
                            value
                        };

                        self.position = 1;
                        count += 1;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e.into()),
                }
            } else if self.position == 1 {
                // Sample 1: bits 10-19
                let raw = (self.buffer >> 10) & 0x3FF;
                let value = sign_extend(raw, 10);

                *sample = if value == (-1 << 9) {
                    INVALID_SAMPLE
                } else {
                    value
                };

                self.position = 2;
                count += 1;
            } else {
                // Sample 2: bits 20-29
                let raw = (self.buffer >> 20) & 0x3FF;
                let value = sign_extend(raw, 10);

                *sample = if value == (-1 << 9) {
                    INVALID_SAMPLE
                } else {
                    value
                };

                self.position = 0;
                count += 1;
            }
        }

        Ok(count)
    }

    fn reset(&mut self) {
        self.buffer = 0;
        self.position = 0;
    }

    fn bytes_per_sample(&self) -> Option<usize> {
        // Variable: 4/3 bytes per sample on average
        None
    }
}
