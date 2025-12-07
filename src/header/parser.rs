use super::types::{Header, RecordMetadata, SegmentInfo, SignalInfo};
use crate::{Error, Result, SignalFormat};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parses a WFDB header file.
pub fn parse_header<P: AsRef<Path>>(path: P) -> Result<Header> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Parse the first line (record line)
    let first_line = lines
        .next()
        .ok_or_else(|| Error::InvalidHeader("Empty header file".to_string()))??;
    let first_line = first_line.trim();
    if first_line.is_empty() {
        return Err(Error::InvalidHeader("Empty header file".to_string()));
    }

    let (metadata, is_multi_segment) = parse_record_line(first_line)?;
    let num_signals = metadata.num_signals;

    let mut signals = Vec::new();
    let mut segments = Vec::new();
    let mut info_strings = Vec::new();

    // Parse signal lines or segment lines
    for _ in 0..num_signals {
        if let Some(line) = lines.next() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                if !line.is_empty() {
                    info_strings.push(line.trim_start_matches('#').trim().to_string());
                }
                continue;
            }

            if is_multi_segment {
                segments.push(parse_segment_line(line)?);
            } else {
                signals.push(parse_signal_line(line)?);
            }
        } else {
            return Err(Error::InvalidHeader(
                "Unexpected end of file while parsing signals/segments".to_string(),
            ));
        }
    }

    // Parse remaining lines as info strings
    for line in lines {
        let line = line?;
        let line = line.trim();
        if line.starts_with('#') {
            info_strings.push(line.trim_start_matches('#').trim().to_string());
        }
    }

    Ok(Header {
        metadata,
        signals,
        segments: if is_multi_segment {
            Some(segments)
        } else {
            None
        },
        info_strings,
    })
}

fn parse_record_line(line: &str) -> Result<(RecordMetadata, bool)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(Error::InvalidHeader(format!("Invalid record line: {line}")));
    }

    // Record name and segments (e.g., "100/2")
    let name_part = parts[0];
    let (name, num_segments) = if let Some((n, s)) = name_part.split_once('/') {
        (
            n.to_string(),
            Some(s.parse::<usize>().map_err(|_| {
                Error::InvalidHeader(format!("Invalid segment count in: {name_part}"))
            })?),
        )
    } else {
        (name_part.to_string(), None)
    };

    let num_signals = parts[1]
        .parse::<usize>()
        .map_err(|_| Error::InvalidHeader(format!("Invalid number of signals: {}", parts[1])))?;

    let sampling_frequency;
    let mut counter_frequency = None;
    let mut base_counter = None;
    let mut num_samples = None;
    let mut base_time = None;
    let mut base_date = None;

    if parts.len() > 2 {
        let freq_part = parts[2];

        // Parse sampling frequency, counter frequency, and base counter.
        // Supported formats:
        // - "freq"
        // - "freq/counter_freq"
        // - "freq(base_counter)"
        // - "freq/counter_freq(base_counter)"

        let (samp_part, counter_part) = if let Some((s, c)) = freq_part.split_once('/') {
            (s, Some(c))
        } else {
            (freq_part, None)
        };

        // Parse sampling frequency.
        // If the base counter is attached to the sampling frequency (e.g. "360(0)"), strip it.
        let samp_val_str = if let Some((s, _)) = samp_part.split_once('(') {
            s
        } else {
            samp_part
        };

        sampling_frequency = samp_val_str.parse().map_err(|_| {
            Error::InvalidHeader(format!("Invalid sampling frequency: {samp_val_str}"))
        })?;

        // Parse counter frequency part
        if let Some(c_part) = counter_part {
            let (c_val_str, c_base) = if let Some((c, b)) = c_part.split_once('(') {
                (c, Some(b.trim_end_matches(')')))
            } else {
                (c_part, None)
            };

            counter_frequency = Some(c_val_str.parse().map_err(|_| {
                Error::InvalidHeader(format!("Invalid counter frequency: {c_val_str}"))
            })?);

            // If base counter found in counter part
            if let Some(b) = c_base {
                base_counter = Some(
                    b.parse()
                        .map_err(|_| Error::InvalidHeader(format!("Invalid base counter: {b}")))?,
                );
            }
        }
    } else {
        sampling_frequency = 250.0;
    }

    if parts.len() > 3 {
        num_samples = Some(parts[3].parse().map_err(|_| {
            Error::InvalidHeader(format!("Invalid number of samples: {}", parts[3]))
        })?);
    }

    if parts.len() > 4 {
        base_time = Some(parts[4].to_string());
    }

    if parts.len() > 5 {
        base_date = Some(parts[5].to_string());
    }

    let is_multi_segment = num_segments.is_some();

    Ok((
        RecordMetadata {
            name,
            num_signals,
            sampling_frequency,
            counter_frequency,
            base_counter,
            num_samples,
            base_time,
            base_date,
        },
        is_multi_segment,
    ))
}

fn parse_signal_line(line: &str) -> Result<SignalInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(Error::InvalidHeader(format!("Invalid signal line: {line}")));
    }

    let file_name = parts[0].to_string();
    let format_str = parts[1];

    let (format, block_size) = if let Some((f, b)) = format_str.split_once('x') {
        let format_code = f
            .parse::<u16>()
            .map_err(|_| Error::InvalidHeader(format!("Invalid format code: {f}")))?;
        let b_size = b
            .parse::<usize>()
            .map_err(|_| Error::InvalidHeader(format!("Invalid block size: {b}")))?;
        (SignalFormat::from_code(format_code)?, b_size)
    } else {
        let format_code = format_str
            .parse::<u16>()
            .map_err(|_| Error::InvalidHeader(format!("Invalid format code: {format_str}")))?;
        (SignalFormat::from_code(format_code)?, 0)
    };

    let (gain, baseline, units) = if parts.len() > 2 {
        parse_gain_baseline_units(parts[2])?
    } else {
        (200.0, 0, "mV".to_string())
    };

    let mut adc_res = 12; // Default?
    let mut adc_zero = 0;
    let mut init_value = 0;
    let mut checksum = 0;
    let mut description = None;

    if parts.len() > 3 {
        adc_res = parts[3]
            .parse()
            .map_err(|_| Error::InvalidHeader(format!("Invalid ADC resolution: {}", parts[3])))?;
    }

    if parts.len() > 4 {
        adc_zero = parts[4]
            .parse()
            .map_err(|_| Error::InvalidHeader(format!("Invalid ADC zero: {}", parts[4])))?;
    }

    if parts.len() > 5 {
        init_value = parts[5]
            .parse()
            .map_err(|_| Error::InvalidHeader(format!("Invalid initial value: {}", parts[5])))?;
    }

    if parts.len() > 6 {
        // Checksum can be unsigned or signed (e.g. -1 for unknown, or just negative 16-bit value)
        checksum = if let Ok(val) = parts[6].parse::<u16>() {
            val
        } else if let Ok(val) = parts[6].parse::<i16>() {
            val as u16
        } else {
            return Err(Error::InvalidHeader(format!(
                "Invalid checksum: {}",
                parts[6]
            )));
        };
    }

    // Field 7 is optionally block size. Field 8+ is description.
    // If field 7 is not an integer, it's considered the start of the description.

    let mut final_block_size = block_size;
    let mut desc_start_index = 8;

    if parts.len() > 7 {
        let maybe_block_size = parts[7];
        if let Ok(bs) = maybe_block_size.parse::<usize>() {
            // It's a valid integer, likely block size
            if block_size == 0 {
                final_block_size = bs;
            }
        } else {
            // Not an integer, must be start of description
            desc_start_index = 7;
        }
    }

    if parts.len() > desc_start_index {
        description = Some(parts[desc_start_index..].join(" "));
    }

    Ok(SignalInfo {
        file_name,
        format,
        gain,
        baseline,
        units,
        adc_res,
        adc_zero,
        init_value,
        checksum,
        block_size: final_block_size,
        description,
    })
}

fn parse_gain_baseline_units(gain_part: &str) -> Result<(f64, i32, String)> {
    if let Some((g, rest)) = gain_part.split_once('(') {
        let gain = g
            .parse()
            .map_err(|_| Error::InvalidHeader(format!("Invalid gain: {g}")))?;

        if let Some((b, u_part)) = rest.split_once(')') {
            let baseline = b
                .parse()
                .map_err(|_| Error::InvalidHeader(format!("Invalid baseline: {b}")))?;

            let units = u_part
                .strip_prefix('/')
                .map_or_else(|| "mV".to_string(), ToString::to_string);

            Ok((gain, baseline, units))
        } else {
            Err(Error::InvalidHeader(format!(
                "Invalid gain/baseline format: {gain_part}"
            )))
        }
    } else if let Some((g, u)) = gain_part.split_once('/') {
        // Handle "gain/units" format
        let gain = g
            .parse()
            .map_err(|_| Error::InvalidHeader(format!("Invalid gain: {g}")))?;
        let units = u.to_string();
        Ok((gain, 0, units))
    } else {
        let gain = gain_part
            .parse()
            .map_err(|_| Error::InvalidHeader(format!("Invalid gain: {gain_part}")))?;
        Ok((gain, 0, "mV".to_string()))
    }
}

fn parse_segment_line(line: &str) -> Result<SegmentInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(Error::InvalidHeader(format!(
            "Invalid segment line: {line}"
        )));
    }

    let name = parts[0].to_string();
    let num_samples = parts[1].parse().map_err(|_| {
        Error::InvalidHeader(format!("Invalid sample count in segment: {}", parts[1]))
    })?;

    Ok(SegmentInfo { name, num_samples })
}
