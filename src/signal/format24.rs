use crate::signal::common::{FormatDecoder, INVALID_SAMPLE};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 24 (24-bit two's complement, little-endian).
#[derive(Debug, Clone, Default)]
pub struct Format24Decoder;

impl Format24Decoder {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl FormatDecoder for Format24Decoder {
    #[allow(clippy::cast_possible_wrap)]
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        let mut count = 0;
        let mut buf = [0u8; 3];

        for sample in output.iter_mut() {
            match reader.read_exact(&mut buf) {
                Ok(()) => {
                    // Construct 24-bit value (little-endian)
                    let value =
                        i32::from(buf[0]) | (i32::from(buf[1]) << 8) | (i32::from(buf[2]) << 16);

                    // Sign extend from bit 23
                    let value = if value & 0x80_0000 != 0 {
                        value | 0xFF00_0000_u32 as i32
                    } else {
                        value & 0x00FF_FFFF
                    };

                    if value == (-1 << 23) {
                        *sample = INVALID_SAMPLE;
                    } else {
                        *sample = value;
                    }
                    count += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(count)
    }

    fn reset(&mut self) {}

    fn bytes_per_sample(&self) -> Option<usize> {
        Some(3)
    }
}
