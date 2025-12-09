use crate::signal::common::{FormatDecoder, INVALID_SAMPLE};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 160 (16-bit offset binary, little-endian).
#[derive(Debug, Clone, Default)]
pub struct Format160Decoder;

impl Format160Decoder {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl FormatDecoder for Format160Decoder {
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        let mut count = 0;
        let mut buf = [0u8; 2];

        for sample in output.iter_mut() {
            match reader.read_exact(&mut buf) {
                Ok(()) => {
                    // Read as unsigned, subtract 32768
                    let unsigned = u16::from_le_bytes(buf);
                    let value = i32::from(unsigned) - 32768;
                    if value == -32768 {
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
        Some(2)
    }
}
