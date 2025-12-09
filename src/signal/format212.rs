use crate::signal::common::{FormatDecoder, INVALID_SAMPLE, sign_extend};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 212 (packed 12-bit samples).
///
/// Two 12-bit samples are bit-packed into 3 bytes:
/// - Bytes 0-1 (little-endian): Contains sample 0 in bits 0-11, sample 1's high bits in 12-15
/// - Byte 2: Contains sample 1's low 8 bits
///
/// The value 0x800 (-2048 in 12-bit two's complement) indicates an invalid sample.
#[derive(Debug, Clone)]
pub struct Format212Decoder {
    /// Buffer for partial data when reading pairs
    buffer: Option<u16>,
    /// Whether we're reading the first or second sample of a pair
    is_second: bool,
}

impl Default for Format212Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Format212Decoder {
    /// Create a new Format 212 decoder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: None,
            is_second: false,
        }
    }
}

impl FormatDecoder for Format212Decoder {
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        let mut count = 0;

        for sample in output.iter_mut() {
            if self.is_second {
                // Read second sample of pair (need 1 byte)
                let mut buf = [0u8; 1];
                match reader.read_exact(&mut buf) {
                    Ok(()) => {
                        let Some(word) = self.buffer else {
                            // Should not happen - reset state and skip
                            self.is_second = false;
                            continue;
                        };
                        // Sample 1: high 4 bits from word (bits 12-15), low 8 bits from new byte
                        let high_bits = (word >> 12) & 0x0F;
                        let low_bits = u16::from(buf[0]);
                        let raw_value = (high_bits << 8) | low_bits;
                        let value = sign_extend(u32::from(raw_value), 12);

                        if value == (-1 << 11) {
                            *sample = INVALID_SAMPLE;
                        } else {
                            *sample = value;
                        }

                        self.buffer = None;
                        self.is_second = false;
                        count += 1;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        // Partial pair - reset state
                        self.buffer = None;
                        self.is_second = false;
                        break;
                    }
                    Err(e) => return Err(e.into()),
                }
            } else {
                // Read first sample of pair (need 2 bytes)
                let mut buf = [0u8; 2];
                match reader.read_exact(&mut buf) {
                    Ok(()) => {
                        let word = u16::from_le_bytes(buf);
                        // Sample 0: bits 0-11
                        let value = sign_extend(u32::from(word & 0x0FFF), 12);

                        if value == (-1 << 11) {
                            *sample = INVALID_SAMPLE;
                        } else {
                            *sample = value;
                        }

                        // Save bits 12-15 for second sample
                        self.buffer = Some(word);
                        self.is_second = true;
                        count += 1;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        break;
                    }
                    Err(e) => return Err(e.into()),
                }
            }
        }

        Ok(count)
    }

    fn reset(&mut self) {
        self.buffer = None;
        self.is_second = false;
    }

    fn bytes_per_sample(&self) -> Option<usize> {
        // Variable: 1.5 bytes per sample on average (3 bytes per 2 samples)
        None
    }

    fn bytes_per_frame(&self, num_signals: usize) -> Option<usize> {
        // Format212: 3 bytes per 2 samples
        // For N signals: ceil(N/2) * 3 bytes per frame
        Some(num_signals.div_ceil(2) * 3)
    }
}
