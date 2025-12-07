/// Information about a single signal.
#[derive(Debug, Clone, PartialEq)]
pub struct SignalInfo {
    /// File name where signal data is stored.
    pub file_name: String,
    /// Format code for the signal data.
    pub format: crate::SignalFormat,
    /// ADC gain (ADC units per physical unit).
    pub gain: f64,
    /// ADC zero (ADC value corresponding to 0 physical units).
    pub baseline: i32,
    /// Physical units of the signal.
    pub units: String,
    /// ADC resolution (bits).
    pub adc_res: u8,
    /// ADC zero (ADC value corresponding to 0 Volts - usually same as baseline).
    pub adc_zero: i32,
    /// Initial value of the signal.
    pub init_value: i32,
    /// Checksum of the signal data.
    pub checksum: u16,
    /// Block size for block-aligned formats.
    pub block_size: usize,
    /// Description of the signal.
    pub description: Option<String>,
}

/// Metadata for a WFDB record.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordMetadata {
    /// Name of the record.
    pub name: String,
    /// Number of signals in the record.
    pub num_signals: usize,
    /// Sampling frequency (Hz).
    pub sampling_frequency: f64,
    /// Counter frequency (Hz) - usually same as sampling frequency.
    pub counter_frequency: Option<f64>,
    /// Base counter value.
    pub base_counter: Option<f64>,
    /// Number of samples per signal.
    pub num_samples: Option<u64>,
    /// Base time (HH:MM:SS).
    pub base_time: Option<String>,
    /// Base date (DD/MM/YYYY).
    pub base_date: Option<String>,
}

/// Information about a segment in a multi-segment record.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentInfo {
    /// Name of the segment record.
    pub name: String,
    /// Number of samples in the segment.
    pub num_samples: u64,
}

/// Parsed header content.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// Record metadata.
    pub metadata: RecordMetadata,
    /// Signal specifications.
    pub signals: Vec<SignalInfo>,
    /// Segment information (for multi-segment records).
    pub segments: Option<Vec<SegmentInfo>>,
    /// Info strings (comments).
    pub info_strings: Vec<String>,
}

impl Default for SignalInfo {
    fn default() -> Self {
        Self {
            file_name: String::new(),
            format: crate::SignalFormat::Format16, // Default to 16-bit
            gain: 200.0,                           // Default gain
            baseline: 0,
            units: "mV".to_string(),
            adc_res: 12, // Default resolution
            adc_zero: 0,
            init_value: 0,
            checksum: 0,
            block_size: 0,
            description: None,
        }
    }
}
