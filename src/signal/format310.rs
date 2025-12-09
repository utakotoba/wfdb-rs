use crate::signal::common::{FormatDecoder, INVALID_SAMPLE, sign_extend};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 310 (packed 10-bit samples).
///
/// Three 10-bit samples are bit-packed into 4 bytes.
#[derive(Debug, Clone)]
pub struct Format310Decoder {
    /// Buffer for reading sample groups
    buffer: [u16; 2],
    /// Current position in the group (0, 1, or 2)
    position: u8,
}

impl Default for Format310Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Format310Decoder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: [0; 2],
            position: 0,
        }
    }
}

impl FormatDecoder for Format310Decoder {
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        let mut count = 0;

        for sample in output.iter_mut() {
            match self.position {
                0 => {
                    // Read first 16-bit word
                    let mut buf = [0u8; 2];
                    match reader.read_exact(&mut buf) {
                        Ok(()) => {
                            self.buffer[0] = u16::from_le_bytes(buf);
                            // Sample 0: bits 1-10 of first word (discard bit 0)
                            let raw = (self.buffer[0] >> 1) & 0x3FF;
                            let value = sign_extend(u32::from(raw), 10);

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
                }
                1 => {
                    // Read second 16-bit word
                    let mut buf = [0u8; 2];
                    match reader.read_exact(&mut buf) {
                        Ok(()) => {
                            self.buffer[1] = u16::from_le_bytes(buf);
                            // Sample 1: bits 1-10 of second word (discard bit 0)
                            let raw = (self.buffer[1] >> 1) & 0x3FF;
                            let value = sign_extend(u32::from(raw), 10);

                            *sample = if value == (-1 << 9) {
                                INVALID_SAMPLE
                            } else {
                                value
                            };

                            self.position = 2;
                            count += 1;
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                            self.position = 0;
                            break;
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                _ => {
                    // Sample 2: bits 11-15 from first word, bits 11-15 from second word
                    let high0 = (self.buffer[0] >> 11) & 0x1F;
                    let high1 = (self.buffer[1] >> 11) & 0x1F;
                    let raw = (high1 << 5) | high0;
                    let value = sign_extend(u32::from(raw), 10);

                    *sample = if value == (-1 << 9) {
                        INVALID_SAMPLE
                    } else {
                        value
                    };

                    self.position = 0;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    fn reset(&mut self) {
        self.buffer = [0; 2];
        self.position = 0;
    }

    fn bytes_per_sample(&self) -> Option<usize> {
        // Variable: 4/3 bytes per sample on average
        None
    }

    fn bytes_per_frame(&self, num_signals: usize) -> Option<usize> {
        // Format310: 4 bytes per 3 samples
        // For N signals: ceil(N/3) * 4 bytes per frame
        Some(num_signals.div_ceil(3) * 4)
    }
}
