use crate::shared::SignalFormat;
use crate::{Error, Result, Sample};

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
        for (sample, bytes) in output.iter_mut().zip(data.chunks_exact(2)) {
            let low = u16::from(bytes[0]);
            let high = u16::from(bytes[1]);
            let value = (high << 8) | low;
            // TODO: Handle error
            *sample = Sample::from(i16::try_from(value).ok().unwrap_or(0));
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

            let byte0 = u16::from(data[offset]);
            let byte1 = u16::from(data[offset + 1]);
            let byte2 = u16::from(data[offset + 2]);

            // Sample 1: LSBs in byte 0, MSBs in low nibble of byte 1
            let sample0_raw = byte0 | ((byte1 & 0x0F) << 8);

            // Sample 2: LSBs in byte 2, MSBs in high nibble of byte 1
            // The high nibble (bits 4-7) of byte1 contains the high 4 bits of sample1
            let sample1_raw = byte2 | ((byte1 & 0xF0) << 4);

            // Sign extension for 12-bit values
            // TODO: Handle error
            let sample0 = if sample0_raw & 0x800 != 0 {
                i16::try_from(sample0_raw | 0xF000).ok().unwrap_or(0)
            } else {
                i16::try_from(sample0_raw).ok().unwrap_or(0)
            };

            let sample1 = if sample1_raw & 0x800 != 0 {
                i16::try_from(sample1_raw | 0xF000).ok().unwrap_or(0)
            } else {
                i16::try_from(sample1_raw).ok().unwrap_or(0)
            };

            output[output_idx] = Sample::from(sample0);
            output_idx += 1;

            // Only write second sample if we have space
            if output_idx < output.len() {
                output[output_idx] = Sample::from(sample1);
                output_idx += 1;
            }
        }

        // Handle partial pair: if we have 3 bytes available but only 1 output slot remaining
        // This can happen when output buffer has odd length
        if output_idx < output.len() && data.len() >= (num_complete_pairs * 3 + 3) {
            let offset = num_complete_pairs * 3;

            let byte0 = u16::from(data[offset]);
            let byte1 = u16::from(data[offset + 1]);

            // Decode only the first sample of the pair
            let sample0_raw = byte0 | ((byte1 & 0x0F) << 8);

            // TODO: Handle error
            let sample0 = if sample0_raw & 0x800 != 0 {
                i16::try_from(sample0_raw | 0xF000).ok().unwrap_or(0)
            } else {
                i16::try_from(sample0_raw).ok().unwrap_or(0)
            };

            output[output_idx] = Sample::from(sample0);
            output_idx += 1;
        }

        Ok(output_idx)
    }
}
