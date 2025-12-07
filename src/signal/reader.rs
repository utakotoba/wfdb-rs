use crate::signal::formats::{FormatDecoder, get_decoder};
use crate::{Error, Header, Result, Sample, SignalFormat};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// A reader for WFDB signal files.
pub struct SignalReader {
    /// The parsed header containing signal metadata.
    header: Header,
    /// Groups of signals that share the same file and format.
    file_groups: Vec<FileGroup>,
    /// Indices of format 0 (null) signals that don't have associated files.
    null_signals: Vec<usize>,
    /// Current time position in samples.
    current_time: u64,
}

/// A group of signals that share the same file and format.
struct FileGroup {
    /// Buffered file reader for the signal file.
    reader: BufReader<File>,
    /// Decoder for the signal format.
    decoder: Box<dyn FormatDecoder>,
    /// Indices of signals in this group (relative to header.signals).
    signal_indices: Vec<usize>,
    /// Buffer for raw bytes read from the file.
    raw_buffer: Vec<u8>,
    /// Buffer for decoded samples (interleaved).
    sample_buffer: Vec<Sample>,
    /// Current position in the sample buffer.
    buffer_pos: usize,
    // /// Signal format for this group.
    // format: SignalFormat,
    /// Number of bytes per frame for all signals in this group.
    bytes_per_frame: usize,
}

impl SignalReader {
    /// Creates a new SignalReader from a parsed header and a base directory.
    pub fn new(header: Header, base_dir: &Path) -> Result<Self> {
        let mut file_groups = Vec::new();
        let mut format0_signals = Vec::new();
        let mut filename_to_signals: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();

        for (i, sig) in header.signals.iter().enumerate() {
            if sig.format == SignalFormat::Format0 {
                format0_signals.push(i);
            } else {
                filename_to_signals
                    .entry(sig.file_name.clone())
                    .or_default()
                    .push(i);
            }
        }

        for (filename, indices) in filename_to_signals {
            let first_sig = &header.signals[indices[0]];
            let format = first_sig.format;
            for &idx in &indices[1..] {
                if header.signals[idx].format != format {
                    return Err(Error::UnsupportedSignalFormat(
                        header.signals[idx].format as u16,
                    ));
                }
            }

            let file_path = base_dir.join(&filename);
            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);
            let decoder = get_decoder(format)?;

            let bytes_per_frame = match format {
                SignalFormat::Format16 => indices.len() * 2,
                SignalFormat::Format212 => {
                    let pairs = (indices.len() + 1) / 2;
                    pairs * 3
                }
                SignalFormat::Format0 => 0,
                _ => return Err(Error::UnsupportedSignalFormat(format as u16)),
            };

            let frames_to_buffer = 1024;
            let buffer_size = bytes_per_frame * frames_to_buffer;
            let num_signals_in_group = indices.len();

            file_groups.push(FileGroup {
                reader,
                decoder,
                signal_indices: indices,
                raw_buffer: vec![0u8; buffer_size],
                sample_buffer: Vec::with_capacity(frames_to_buffer * num_signals_in_group),
                buffer_pos: 0,
                // format,
                bytes_per_frame,
            });
        }

        Ok(Self {
            header,
            file_groups,
            null_signals: format0_signals,
            current_time: 0,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Reads the next vector of samples (one sample per signal).
    /// Returns `None` if end of record is reached.
    pub fn read_frame(&mut self) -> Result<Option<Vec<Sample>>> {
        let num_signals = self.header.metadata.num_signals;
        let mut frame = vec![0; num_signals];

        if let Some(ns) = self.header.metadata.num_samples {
            if self.current_time >= ns {
                return Ok(None);
            }
        }

        for group in &mut self.file_groups {
            if group.buffer_pos >= group.sample_buffer.len() {
                let frames_to_read = group.raw_buffer.len() / group.bytes_per_frame;
                let bytes_to_read = frames_to_read * group.bytes_per_frame;
                let chunk = &mut group.raw_buffer[0..bytes_to_read];

                let n = group.reader.read(chunk)?;
                if n == 0 {
                    return Ok(None);
                }

                let valid_bytes = (n / group.bytes_per_frame) * group.bytes_per_frame;
                if valid_bytes == 0 {
                    return Ok(None);
                }

                let num_frames = valid_bytes / group.bytes_per_frame;
                let num_samples_total = num_frames * group.signal_indices.len();
                group.sample_buffer.resize(num_samples_total, 0);

                group
                    .decoder
                    .decode(&group.raw_buffer[0..valid_bytes], &mut group.sample_buffer)?;
                group.buffer_pos = 0;
            }

            for (i, &sig_idx) in group.signal_indices.iter().enumerate() {
                if group.buffer_pos + i < group.sample_buffer.len() {
                    frame[sig_idx] = group.sample_buffer[group.buffer_pos + i];
                } else {
                    return Ok(None);
                }
            }

            group.buffer_pos += group.signal_indices.len();
        }

        for &sig_idx in &self.null_signals {
            frame[sig_idx] = 0;
        }

        self.current_time += 1;
        Ok(Some(frame))
    }
}
