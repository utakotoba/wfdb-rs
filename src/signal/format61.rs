use crate::signal::common::{FormatDecoder, INVALID_SAMPLE};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 61 (16-bit two's complement, big-endian).
#[derive(Debug, Clone, Default)]
pub struct Format61Decoder;

impl Format61Decoder {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl FormatDecoder for Format61Decoder {
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        let mut count = 0;
        let mut buf = [0u8; 2];

        for sample in output.iter_mut() {
            match reader.read_exact(&mut buf) {
                Ok(()) => {
                    // Big-endian: MSB first
                    let value = i16::from_be_bytes(buf);
                    if value == i16::MIN {
                        *sample = INVALID_SAMPLE;
                    } else {
                        *sample = i32::from(value);
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
