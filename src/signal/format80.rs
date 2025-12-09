use crate::signal::common::{FormatDecoder, INVALID_SAMPLE};
use crate::{Result, Sample};
use std::io::BufRead;

/// Decoder for WFDB Format 80 (8-bit offset binary).
#[derive(Debug, Clone, Default)]
pub struct Format80Decoder;

impl Format80Decoder {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl FormatDecoder for Format80Decoder {
    fn decode_buf(&mut self, reader: &mut dyn BufRead, output: &mut [Sample]) -> Result<usize> {
        let mut count = 0;
        let mut buf = [0u8; 1];

        for sample in output.iter_mut() {
            match reader.read_exact(&mut buf) {
                Ok(()) => {
                    // Subtract 128 to convert from offset binary
                    let value = i32::from(buf[0]) - 128;
                    if value == -128 {
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
        Some(1)
    }
}
