use crate::{Error, Result, Sample, SignalFormat};

/// A trait for decoding signal data.
pub trait FormatDecoder: Send + Sync {
    /// Decodes raw bytes into samples.
    ///
    /// Returns the number of samples decoded.
    fn decode(&self, data: &[u8], output: &mut [Sample]) -> Result<usize>;
}

pub fn get_decoder(format: SignalFormat) -> Result<Box<dyn FormatDecoder>> {
    match format {
        SignalFormat::Format0 => Ok(Box::new(Format0Decoder)),
        SignalFormat::Format16 => Ok(Box::new(Format16Decoder)),
        SignalFormat::Format212 => Ok(Box::new(Format212Decoder)),
        _ => Err(Error::UnsupportedSignalFormat(format as u16)),
    }
}

pub struct Format0Decoder;

impl FormatDecoder for Format0Decoder {
    fn decode(&self, _data: &[u8], output: &mut [Sample]) -> Result<usize> {
        for sample in output.iter_mut() {
            *sample = 0;
        }
        Ok(output.len())
    }
}

pub struct Format16Decoder;

impl FormatDecoder for Format16Decoder {
    fn decode(&self, data: &[u8], output: &mut [Sample]) -> Result<usize> {
        if data.len() < 2 {
            return Ok(0);
        }

        let num_samples = (data.len() / 2).min(output.len());
        for i in 0..num_samples {
            let offset = i * 2;
            let low = data[offset] as u16;
            let high = data[offset + 1] as u16;
            let value = (high << 8) | low;
            output[i] = value as i16 as Sample;
        }

        Ok(num_samples)
    }
}

pub struct Format212Decoder;

impl FormatDecoder for Format212Decoder {
    fn decode(&self, data: &[u8], output: &mut [Sample]) -> Result<usize> {
        // Format 212 packs 2 samples into 3 bytes
        // We need at least 3 bytes to decode any samples
        if data.len() < 3 {
            return Ok(0);
        }

        // Calculate how many complete pairs we can decode
        let max_pairs_from_data = data.len() / 3;
        let max_pairs_from_output = output.len() / 2;
        let num_complete_pairs = max_pairs_from_data.min(max_pairs_from_output);

        let mut output_idx = 0;

        // Decode complete pairs
        for i in 0..num_complete_pairs {
            let offset = i * 3;

            let byte0 = data[offset] as u16;
            let byte1 = data[offset + 1] as u16;
            let byte2 = data[offset + 2] as u16;

            // Sample 1: LSBs in byte 0, MSBs in low nibble of byte 1
            let sample0_raw = byte0 | ((byte1 & 0x0F) << 8);

            // Sample 2: LSBs in byte 2, MSBs in high nibble of byte 1
            // The high nibble (bits 4-7) of byte1 contains the high 4 bits of sample1
            let sample1_raw = byte2 | ((byte1 & 0xF0) << 4);

            // Sign extension for 12-bit values
            let sample0 = if sample0_raw & 0x800 != 0 {
                (sample0_raw | 0xF000) as i16
            } else {
                sample0_raw as i16
            };

            let sample1 = if sample1_raw & 0x800 != 0 {
                (sample1_raw | 0xF000) as i16
            } else {
                sample1_raw as i16
            };

            output[output_idx] = sample0 as Sample;
            output_idx += 1;

            // Only write second sample if we have space
            if output_idx < output.len() {
                output[output_idx] = sample1 as Sample;
                output_idx += 1;
            }
        }

        // Handle partial pair: if we have 3 bytes available but only 1 output slot remaining
        // This can happen when output buffer has odd length
        if output_idx < output.len() && data.len() >= (num_complete_pairs * 3 + 3) {
            let offset = num_complete_pairs * 3;

            let byte0 = data[offset] as u16;
            let byte1 = data[offset + 1] as u16;

            // Decode only the first sample of the pair
            let sample0_raw = byte0 | ((byte1 & 0x0F) << 8);

            let sample0 = if sample0_raw & 0x800 != 0 {
                (sample0_raw | 0xF000) as i16
            } else {
                sample0_raw as i16
            };

            output[output_idx] = sample0 as Sample;
            output_idx += 1;
        }

        Ok(output_idx)
    }
}
