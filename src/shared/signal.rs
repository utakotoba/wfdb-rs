use crate::SignalFormat;

/// Essential information to resolve a single signal.
///
/// > Note that the `skew` and `byte_offset` are _not included_ in
/// > the public API, since they are only used as internal state.
///
/// # Examples
///
/// Here are a few examples of a validated signal specification line:
///
/// - `100.dat 212 200 11 1024 995 0 MLII` _refer to the example on
///   [WFDB Website](https://wfdb.io/spec/header-files.html#signal-specification-lines)_
/// - `00001_lr.dat 16 1000.0(0)/mV 16 0 -119 1508 0 I` _refer to a record in
///   [PTB-XL Dataset](https://physionet.org/content/ptb-xl)_
#[derive(Debug, Clone, PartialEq)]
pub struct SignalInfo {
    /// Name of the file containing signal data.
    pub file_name: String,
    /// Format code for the signal data.
    pub format: SignalFormat,
    // /// Number of samples per frame.
    // pub sample_per_frame: Option<i32>,
    /// ADC gain (ADC units per physical unit).
    pub gain: Option<f64>,
    /// ADC zero (ADC value corresponding to 0 physical units).
    pub baseline: Option<i32>,
    /// Physical units of the signal.
    pub units: Option<String>,
    /// ADC resolution (bits).
    pub adc_res: Option<u8>, // TODO: need to verify the type again
    /// ADC zero (ADC value corresponding to 0 Volts - usually same as baseline).
    pub adc_zero: Option<i32>,
    /// Initial value of the signal.
    pub initial_value: Option<i32>,
    /// Checksum of the signal data.
    pub checksum: Option<u16>,
    /// Block size for block-aligned formats.
    pub block_size: Option<usize>,
    /// Description of the signal.
    pub description: Option<String>,
}
