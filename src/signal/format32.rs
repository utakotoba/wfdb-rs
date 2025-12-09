use crate::signal::common::{FormatDecoder, INVALID_SAMPLE};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 32 (32-bit two's complement, little-endian).
#[derive(Debug, Clone, Default)]
pub struct Format32Decoder;

impl Format32Decoder {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl FormatDecoder for Format32Decoder {
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        let mut count = 0;
        let mut buf = [0u8; 4];

        for sample in output.iter_mut() {
            match reader.read_exact(&mut buf) {
                Ok(()) => {
                    let value = i32::from_le_bytes(buf);
                    if value == i32::MIN {
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
        Some(4)
    }
}
