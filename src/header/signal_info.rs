use crate::{Error, Result, SignalFormat};

/// Parsed format field components.
///
/// __INTERNAL USE ONLY__
type FormatFieldComponents = (SignalFormat, Option<u32>, Option<u32>, Option<u64>);

/// Return type for parsed optional fields from a WFDB signal specification line.
///
/// __INTERNAL USE ONLY__
struct OptionalFields {
    adc_gain: Option<f64>,
    baseline: Option<i32>,
    units: Option<String>,
    adc_resolution: Option<u8>,
    adc_zero: Option<i32>,
    initial_value: Option<i32>,
    checksum: Option<i16>,
    block_size: Option<i32>,
    description: Option<String>,
}

/// Type of optional field detected by format.
///
/// __INTERNAL USE ONLY__
#[derive(Debug, Clone, Copy, PartialEq)]
enum FieldType {
    Gain,
    Resolution,
    AdcZero,
    InitialValue,
    Checksum,
    BlockSize,
    Description,
}

/// Parsing state for optional fields in signal specification lines.
///
/// __INTERNAL USE ONLY__
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum ParseState {
    Start,
    AfterGain,
    AfterResolution,
    AfterZero,
    AfterInitial,
    AfterChecksum,
    AfterBlockSize,
}

/// Signal specification from a WFDB header signal line.
///
/// # Examples
///
/// Here are a few examples of a validated signal specification line:
///
/// - `100.dat 212 200 11 1024 995 -22131 0 MLII` _from MIT-BIH Database_
/// - `data0 8 100 10 0 -53 -1279 0 ECG signal 0` _from AHA Database_
/// - `- 16` _minimal specification (standard I/O, format 16)_
/// - `sig.dat 16x2:100+512 200(500)/uV 12 2048 0 0 0 Channel A`
///   _with samples per frame, skew, byte offset, and full ADC specifications_
#[derive(Debug, Clone, PartialEq)]
pub struct SignalInfo {
    /// Name of the file containing the signal samples.
    pub file_name: String,
    /// Storage format for the signal.
    pub format: SignalFormat,
    /// Number of samples per frame (default: 1).
    pub samples_per_frame: Option<u32>,
    /// Number of samples of skew relative to sample 0.
    pub skew: Option<u32>,
    /// Byte offset from beginning of file to sample 0.
    pub byte_offset: Option<u64>,
    /// ADC gain in ADC units per physical unit.
    pub adc_gain: Option<f64>,
    /// Baseline value in ADC units corresponding to 0 physical units.
    pub baseline: Option<i32>,
    /// Physical unit name (e.g., "mV", "uV").
    pub units: Option<String>,
    /// ADC resolution in bits.
    pub adc_resolution: Option<u8>,
    /// ADC zero value (center of ADC range).
    pub adc_zero: Option<i32>,
    /// Initial sample value (for difference formats).
    pub initial_value: Option<i32>,
    /// 16-bit checksum of all samples.
    pub checksum: Option<i16>,
    /// Block size in bytes for special files (usually 0).
    pub block_size: Option<i32>,
    /// Human-readable description of the signal.
    pub description: Option<String>,
}

impl SignalInfo {
    // [Default values for optional fields]

    /// Default ADC gain (ADC units per physical unit) when omitted.
    pub const DEFAULT_ADC_GAIN: f64 = 200.0;

    /// Default ADC resolution (bits) for amplitude-format signals.
    pub const DEFAULT_ADC_RESOLUTION_AMPLITUDE: u8 = 12;

    /// Default ADC resolution (bits) for difference-format signals.
    pub const DEFAULT_ADC_RESOLUTION_DIFFERENCE: u8 = 10;

    /// Default ADC zero value when omitted.
    pub const DEFAULT_ADC_ZERO: i32 = 0;

    /// Default baseline value when omitted (equals ADC zero).
    pub const DEFAULT_BASELINE: i32 = Self::DEFAULT_ADC_ZERO;

    /// Default physical units when omitted.
    pub const DEFAULT_UNITS: &'static str = "mV";

    /// Default samples per frame when omitted.
    pub const DEFAULT_SAMPLES_PER_FRAME: u32 = 1;

    /// Default skew when omitted.
    pub const DEFAULT_SKEW: u32 = 0;

    /// Default byte offset when omitted.
    pub const DEFAULT_BYTE_OFFSET: u64 = 0;

    /// Default block size when omitted.
    pub const DEFAULT_BLOCK_SIZE: i32 = 0;

    // [Signal info decoding functions]

    /// Build signal information from a signal specification line in a WFDB header.
    ///
    /// # Errors
    ///
    /// Will return an error if the format of the signal specification line is invalid.
    pub fn from_signal_line(line: &str) -> Result<Self> {
        let line = line.trim();
        let mut parts = line.split_whitespace();

        // First field: file name (required)
        let file_name = parts
            .next()
            .ok_or_else(|| Error::InvalidHeader("Missing file name".to_string()))?
            .to_string();

        // Second field: format (required), possibly with modifiers
        let format_field = parts
            .next()
            .ok_or_else(|| Error::InvalidHeader("Missing format field".to_string()))?;

        let (format, samples_per_frame, skew, byte_offset) =
            Self::parse_format_field(format_field)?;

        // Collect remaining optional fields
        let remaining: Vec<&str> = parts.collect();

        // Parse optional fields
        let optional_fields = Self::parse_optional_fields(&remaining)?;

        Ok(Self {
            file_name,
            format,
            samples_per_frame,
            skew,
            byte_offset,
            adc_gain: optional_fields.adc_gain,
            baseline: optional_fields.baseline,
            units: optional_fields.units,
            adc_resolution: optional_fields.adc_resolution,
            adc_zero: optional_fields.adc_zero,
            initial_value: optional_fields.initial_value,
            checksum: optional_fields.checksum,
            block_size: optional_fields.block_size,
            description: optional_fields.description,
        })
    }

    /// Parse the format field and its optional modifiers.
    ///
    /// Format: `format[xsamples_per_frame][:skew][+byte_offset]`
    fn parse_format_field(field: &str) -> Result<FormatFieldComponents> {
        let mut format_str = field;
        let mut samples_per_frame = None;
        let mut skew = None;
        let mut byte_offset = None;

        // Extract byte_offset (if present, marked with '+')
        if let Some(plus_pos) = format_str.find('+') {
            let offset_str = &format_str[plus_pos + 1..];
            byte_offset = Some(
                offset_str
                    .parse()
                    .map_err(|e| Error::InvalidHeader(format!("Invalid byte offset: {e}")))?,
            );
            format_str = &format_str[..plus_pos];
        }

        // Extract skew (if present, marked with ':')
        if let Some(colon_pos) = format_str.find(':') {
            let skew_str = &format_str[colon_pos + 1..];
            skew = Some(
                skew_str
                    .parse()
                    .map_err(|e| Error::InvalidHeader(format!("Invalid skew: {e}")))?,
            );
            format_str = &format_str[..colon_pos];
        }

        // Extract samples_per_frame (if present, marked with 'x')
        if let Some(x_pos) = format_str.find('x') {
            let spf_str = &format_str[x_pos + 1..];
            samples_per_frame =
                Some(spf_str.parse().map_err(|e| {
                    Error::InvalidHeader(format!("Invalid samples per frame: {e}"))
                })?);
            if let Some(spf) = samples_per_frame
                && spf == 0
            {
                return Err(Error::InvalidHeader(
                    "Samples per frame must be greater than zero".to_string(),
                ));
            }
            format_str = &format_str[..x_pos];
        }

        // Parse the base format code
        let format_code: u16 = format_str
            .parse()
            .map_err(|e| Error::InvalidHeader(format!("Invalid format code: {e}")))?;

        // Convert to SignalFormat enum
        let format = SignalFormat::try_from(format_code).map_err(|_| {
            Error::InvalidHeader(format!("Unsupported signal format: {format_code}"))
        })?;

        Ok((format, samples_per_frame, skew, byte_offset))
    }

    /// Parse optional fields following the format field.
    fn parse_optional_fields(fields: &[&str]) -> Result<OptionalFields> {
        // Return early if no optional fields
        if fields.is_empty() {
            return Ok(OptionalFields {
                adc_gain: None,
                baseline: None,
                units: None,
                adc_resolution: None,
                adc_zero: None,
                initial_value: None,
                checksum: None,
                block_size: None,
                description: None,
            });
        }

        let mut adc_gain = None;
        let mut baseline = None;
        let mut units = None;
        let mut adc_resolution = None;
        let mut adc_zero = None;
        let mut initial_value = None;
        let mut checksum = None;
        let mut block_size = None;
        let mut description = None;

        let mut state = ParseState::Start;

        for (field_idx, field) in fields.iter().enumerate() {
            let field_type = Self::detect_field_type(field, state)?;

            match field_type {
                FieldType::Gain => {
                    (adc_gain, baseline, units) = Self::parse_gain_field(field)?;
                    state = ParseState::AfterGain;
                }
                FieldType::Resolution => {
                    adc_resolution = Some(Self::parse_resolution(field)?);
                    state = ParseState::AfterResolution;
                }
                FieldType::AdcZero => {
                    adc_zero = Some(Self::parse_adc_zero(field)?);
                    state = ParseState::AfterZero;
                }
                FieldType::InitialValue => {
                    initial_value = Some(Self::parse_initial_value(field)?);
                    state = ParseState::AfterInitial;
                }
                FieldType::Checksum => {
                    checksum = Some(Self::parse_checksum(field)?);
                    state = ParseState::AfterChecksum;
                }
                FieldType::BlockSize => {
                    block_size = Some(Self::parse_block_size(field)?);
                    state = ParseState::AfterBlockSize;
                }
                FieldType::Description => {
                    description = Some(Self::join_description(&fields[field_idx..]));
                    break;
                }
            }
        }

        Ok(OptionalFields {
            adc_gain,
            baseline,
            units,
            adc_resolution,
            adc_zero,
            initial_value,
            checksum,
            block_size,
            description,
        })
    }

    /// Detect the type of an optional field based on its format and current parse state.
    ///
    /// # Errors
    ///
    /// Returns an error if a field appears out of order or is duplicated.
    fn detect_field_type(field: &str, state: ParseState) -> Result<FieldType> {
        match state {
            ParseState::Start => {
                // Try to detect gain field (has '/' or '(' or is valid positive float)
                if field.contains('/') || field.contains('(') {
                    // Check if it's a valid gain field
                    if Self::parse_gain_field(field).is_ok() {
                        Ok(FieldType::Gain)
                    } else {
                        Ok(FieldType::Description)
                    }
                } else if let Ok(val) = field.parse::<f64>() {
                    // Positive float is gain, zero or negative is block_size
                    if val > 0.0 {
                        Ok(FieldType::Gain)
                    } else if field.parse::<i32>().is_ok() {
                        Ok(FieldType::BlockSize)
                    } else {
                        Ok(FieldType::Description)
                    }
                } else if field.parse::<i32>().is_ok() {
                    // Plain integer is block size
                    Ok(FieldType::BlockSize)
                } else {
                    // Non-numeric is description
                    Ok(FieldType::Description)
                }
            }
            ParseState::AfterGain => {
                if field.parse::<u8>().is_ok() {
                    Ok(FieldType::Resolution)
                } else {
                    Err(Error::InvalidHeader(format!(
                        "Expected ADC resolution after gain, found '{field}'"
                    )))
                }
            }
            ParseState::AfterResolution => {
                if field.parse::<i32>().is_ok() {
                    Ok(FieldType::AdcZero)
                } else {
                    Err(Error::InvalidHeader(format!(
                        "Expected ADC zero after resolution, found '{field}'"
                    )))
                }
            }
            ParseState::AfterZero => {
                if field.parse::<i32>().is_ok() {
                    Ok(FieldType::InitialValue)
                } else {
                    Err(Error::InvalidHeader(format!(
                        "Expected initial value after ADC zero, found '{field}'"
                    )))
                }
            }
            ParseState::AfterInitial => {
                if field.parse::<i16>().is_ok() {
                    Ok(FieldType::Checksum)
                } else {
                    Err(Error::InvalidHeader(format!(
                        "Expected checksum after initial value, found '{field}'"
                    )))
                }
            }
            ParseState::AfterChecksum => {
                if field.parse::<i32>().is_ok() {
                    Ok(FieldType::BlockSize)
                } else {
                    Err(Error::InvalidHeader(format!(
                        "Expected block size after checksum, found '{field}'"
                    )))
                }
            }
            ParseState::AfterBlockSize => Ok(FieldType::Description),
        }
    }

    /// Parse ADC resolution field.
    fn parse_resolution(field: &str) -> Result<u8> {
        field
            .parse()
            .map_err(|e| Error::InvalidHeader(format!("Invalid ADC resolution: {e}")))
    }

    /// Parse ADC zero field.
    fn parse_adc_zero(field: &str) -> Result<i32> {
        field
            .parse()
            .map_err(|e| Error::InvalidHeader(format!("Invalid ADC zero: {e}")))
    }

    /// Parse initial value field.
    fn parse_initial_value(field: &str) -> Result<i32> {
        field
            .parse()
            .map_err(|e| Error::InvalidHeader(format!("Invalid initial value: {e}")))
    }

    /// Parse checksum field.
    fn parse_checksum(field: &str) -> Result<i16> {
        field
            .parse()
            .map_err(|e| Error::InvalidHeader(format!("Invalid checksum: {e}")))
    }

    /// Parse block size field.
    fn parse_block_size(field: &str) -> Result<i32> {
        field
            .parse()
            .map_err(|e| Error::InvalidHeader(format!("Invalid block size: {e}")))
    }

    /// Parse ADC gain field: `gain[(baseline)][/units]`
    fn parse_gain_field(field: &str) -> Result<(Option<f64>, Option<i32>, Option<String>)> {
        let mut gain_part = field;
        let mut units = None;

        // Extract units (if present, marked with '/')
        if let Some(slash_pos) = field.find('/') {
            let units_str = &field[slash_pos + 1..];
            if units_str.is_empty() {
                return Err(Error::InvalidHeader("Units field is empty".to_string()));
            }
            units = Some(units_str.to_string());
            gain_part = &field[..slash_pos];
        }

        let mut baseline = None;

        // Extract baseline (if present, surrounded by parentheses)
        if let Some(paren_start) = gain_part.find('(') {
            let paren_end = gain_part.find(')').ok_or_else(|| {
                Error::InvalidHeader("Missing closing parenthesis in baseline".to_string())
            })?;

            baseline = Some(
                gain_part[paren_start + 1..paren_end]
                    .parse()
                    .map_err(|e| Error::InvalidHeader(format!("Invalid baseline value: {e}")))?,
            );

            gain_part = &gain_part[..paren_start];
        }

        // Parse the gain value
        if gain_part.is_empty() {
            return Err(Error::InvalidHeader("ADC gain is empty".to_string()));
        }

        let gain = Some(
            gain_part
                .parse()
                .map_err(|e| Error::InvalidHeader(format!("Invalid ADC gain: {e}")))?,
        );

        // Validate gain is positive
        if let Some(g) = gain
            && g <= 0.0
        {
            return Err(Error::InvalidHeader(format!(
                "ADC gain must be greater than zero, got {g}"
            )));
        }

        Ok((gain, baseline, units))
    }

    /// Join remaining fields into a description string.
    fn join_description(fields: &[&str]) -> String {
        fields.join(" ")
    }

    // [Accessors]

    /// Get the file name of the signal.
    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Get the format of the signal.
    #[must_use]
    pub const fn format(&self) -> SignalFormat {
        self.format
    }

    /// Get the samples per frame.
    ///
    /// Fallback to the default value when omitted.
    #[must_use]
    pub fn samples_per_frame(&self) -> u32 {
        self.samples_per_frame
            .unwrap_or(Self::DEFAULT_SAMPLES_PER_FRAME)
    }

    /// Get the skew of the signal.
    ///
    /// Fallback to the default value when omitted.
    #[must_use]
    pub fn skew(&self) -> u32 {
        self.skew.unwrap_or(Self::DEFAULT_SKEW)
    }

    /// Get the byte offset of the signal file.
    ///
    /// Fallback to the default value when omitted.
    #[must_use]
    pub fn byte_offset(&self) -> u64 {
        self.byte_offset.unwrap_or(Self::DEFAULT_BYTE_OFFSET)
    }

    /// Get the ADC gain of the signal.
    ///
    /// Fallback to the default ADC gain when omitted.
    #[must_use]
    pub fn adc_gain(&self) -> f64 {
        self.adc_gain.unwrap_or(Self::DEFAULT_ADC_GAIN)
    }

    /// Get the baseline value of the signal.
    ///
    /// Fallback to the ADC zero value when omitted.
    #[must_use]
    pub fn baseline(&self) -> i32 {
        self.baseline.unwrap_or_else(|| self.adc_zero())
    }

    /// Get the physical units of the signal.
    ///
    /// Fallback to the default units when omitted.
    #[must_use]
    pub fn units(&self) -> &str {
        self.units.as_deref().unwrap_or(Self::DEFAULT_UNITS)
    }

    /// Get the ADC resolution of the signal.
    ///
    /// Returns the specified resolution if present, otherwise returns default
    /// based on signal format (12 bits for amplitude formats, 10 for difference).
    #[must_use]
    pub const fn adc_resolution(&self) -> u8 {
        if let Some(res) = self.adc_resolution {
            res
        } else if matches!(self.format, SignalFormat::Format8) {
            // Difference format
            Self::DEFAULT_ADC_RESOLUTION_DIFFERENCE
        } else {
            // Amplitude format
            Self::DEFAULT_ADC_RESOLUTION_AMPLITUDE
        }
    }

    /// Get the ADC zero value of the signal.
    ///
    /// Fallback to the default ADC zero when omitted.
    #[must_use]
    pub fn adc_zero(&self) -> i32 {
        self.adc_zero.unwrap_or(Self::DEFAULT_ADC_ZERO)
    }

    /// Get the initial value of the signal.
    ///
    /// Fallback to the ADC zero value when omitted.
    #[must_use]
    pub fn initial_value(&self) -> i32 {
        self.initial_value.unwrap_or_else(|| self.adc_zero())
    }

    /// Get the checksum of the signal.
    #[must_use]
    pub const fn checksum(&self) -> Option<i16> {
        self.checksum
    }

    /// Get the block size of the signal file.
    ///
    /// Fallback to the default block size when omitted.
    #[must_use]
    pub fn block_size(&self) -> i32 {
        self.block_size.unwrap_or(Self::DEFAULT_BLOCK_SIZE)
    }

    /// Get the description of the signal.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}
