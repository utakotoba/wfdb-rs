use chrono::{NaiveDate, NaiveTime};

use crate::{Error, Result};

/// Return type for parsed optional fields from a WFDB header record line.
///
/// __INTERNAL USE ONLY__
struct OptionalFields {
    sampling_frequency: f64,
    counter_frequency: Option<f64>,
    base_counter: Option<f64>,
    num_samples: Option<u64>,
    base_time: Option<NaiveTime>,
    base_date: Option<NaiveDate>,
}

/// Type of optional field detected by format.
///
/// __INTERNAL USE ONLY__
#[derive(Debug, Clone, Copy, PartialEq)]
enum FieldType {
    Frequency,
    NumSamples,
    Time,
    Date,
}

/// Parsing state for optional fields.
///
/// __INTERNAL USE ONLY__
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum ParseState {
    Start,
    AfterFrequency,
    AfterNumSamples,
    AfterTime,
    AfterDate,
}

/// Metadata from the WFDB header record line.
///
/// # Examples
///
/// Here are a few examples of a validated record line:
///
/// - `100 2 360 650000 12:00:00 01/01/2000` _refer to the example on
///   [WFDB website](https://wfdb.io/spec/header-files.html#record-line)_
/// - `my_record_0 12 500/100(50) 675000 14:49:37 07/06/2025`
/// - `24_record/2 4 102400` (Many fields are optional.)
/// - `rec 2 12:30:45 01/01/2000` (Optional fields can be omitted in the middle.)
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    /// Identifier for the record (letters, digits, underscores only).
    pub name: String,
    /// If present, appended as /n.  Indicates a multi-segment record.
    pub num_segments: Option<usize>,
    /// Number of signals described in the header.
    pub num_signals: usize,
    /// Samples per second (Hz) per signal.
    pub sampling_frequency: f64,
    /// Frequency (Hz) for counter (secondary clock).
    pub counter_frequency: Option<f64>,
    /// Offset value for counter.
    pub base_counter: Option<f64>,
    /// Total samples per signal.
    pub num_samples: Option<u64>,
    /// Start time of the recording (HH:MM:SS).
    pub base_time: Option<NaiveTime>,
    /// Start date of the recording (DD/MM/YYYY).
    pub base_date: Option<NaiveDate>,
}

impl Metadata {
    const DEFAULT_SAMPLING_FREQUENCY: f64 = 250.0;

    /// Build a metadata from the record line (first line) of WFDB header.
    ///
    /// # Errors
    ///
    /// Will return an error if the format of the record line is invalid.
    pub fn from_record_line(line: &str) -> Result<Self> {
        let line = line.trim();
        if line.is_empty() {
            return Err(Error::InvalidHeader("Empty record line".to_string()));
        }

        let mut parts = line.split_whitespace();

        // Resolve first part: record name (required) and number of segments (optional)
        let (name, num_segments) = Self::parse_record_name(
            parts
                .next()
                .ok_or_else(|| Error::InvalidHeader("Missing record name".to_string()))?,
        )?;

        // Resolve second part: number of signals (required)
        let num_signals = parts
            .next()
            .ok_or_else(|| Error::InvalidHeader("Missing number of signals".to_string()))?
            .parse()
            .map_err(|e| Error::InvalidHeader(format!("Invalid number of signals: {e}")))?;

        // Collect remaining optional fields
        let remaining: Vec<&str> = parts.collect();

        // Parse optional fields by detecting their format
        let optional_fields = Self::parse_optional_fields(&remaining)?;

        Ok(Self {
            name,
            num_segments,
            num_signals,
            sampling_frequency: optional_fields.sampling_frequency,
            counter_frequency: optional_fields.counter_frequency,
            base_counter: optional_fields.base_counter,
            num_samples: optional_fields.num_samples,
            base_time: optional_fields.base_time,
            base_date: optional_fields.base_date,
        })
    }

    /// Parse optional fields by detecting their format.
    fn parse_optional_fields(fields: &[&str]) -> Result<OptionalFields> {
        let mut sampling_frequency = Self::DEFAULT_SAMPLING_FREQUENCY;
        let mut counter_frequency = None;
        let mut base_counter = None;
        let mut num_samples = None;
        let mut base_time = None;
        let mut base_date = None;

        let mut state = ParseState::Start;

        for field in fields {
            let field_type = Self::detect_field_type(field, state)?;

            match field_type {
                FieldType::Frequency => {
                    (sampling_frequency, counter_frequency, base_counter) =
                        Self::parse_frequency_field(Some(field))?;
                    state = ParseState::AfterFrequency;
                }
                FieldType::NumSamples => {
                    let n = field.parse().map_err(|e| {
                        Error::InvalidHeader(format!("Invalid number of samples: {e}"))
                    })?;
                    num_samples = Some(n);
                    state = ParseState::AfterNumSamples;
                }
                FieldType::Time => {
                    base_time = Self::parse_base_time(Some(field))?;
                    state = ParseState::AfterTime;
                }
                FieldType::Date => {
                    base_date = Self::parse_base_date(Some(field))?;
                    state = ParseState::AfterDate;
                }
            }
        }

        Ok(OptionalFields {
            sampling_frequency,
            counter_frequency,
            base_counter,
            num_samples,
            base_time,
            base_date,
        })
    }

    /// Detect the type of an optional field based on its format and current parse state.
    ///
    /// # Errors
    ///
    /// Returns an error if a field appears out of order or is duplicated.
    fn detect_field_type(field: &str, state: ParseState) -> Result<FieldType> {
        // Time: contains colon (HH:MM:SS)
        if field.contains(':') {
            if state >= ParseState::AfterTime {
                return Err(Error::InvalidHeader(
                    "Duplicate or out-of-order time field".to_string(),
                ));
            }
            return Ok(FieldType::Time);
        }

        // Date: DD/MM/YYYY pattern - contains two `/` separators
        if field.matches('/').count() == 2 {
            if state >= ParseState::AfterDate {
                return Err(Error::InvalidHeader(
                    "Duplicate or out-of-order date field".to_string(),
                ));
            }
            return Ok(FieldType::Date);
        }

        // Frequency with counter: contains single `/` or `(`
        if field.contains('/') || field.contains('(') {
            if state >= ParseState::AfterFrequency {
                return Err(Error::InvalidHeader(
                    "Duplicate or out-of-order frequency field".to_string(),
                ));
            }
            return Ok(FieldType::Frequency);
        }

        // Plain numeric field: frequency or num_samples based on state
        match state {
            ParseState::Start => Ok(FieldType::Frequency),
            ParseState::AfterFrequency => Ok(FieldType::NumSamples),
            _ => Err(Error::InvalidHeader(format!(
                "Unexpected numeric field '{field}' after time/date"
            ))),
        }
    }

    /// Parse record name and num of segments (optional).
    fn parse_record_name(field: &str) -> Result<(String, Option<usize>)> {
        let (name, num_segments) = match field.split_once('/') {
            Some((name, num_segments)) => {
                let num_segments = num_segments.parse().map_err(|e| {
                    Error::InvalidHeader(format!("Invalid number of segments: {e}"))
                })?;
                (name, Some(num_segments))
            }
            None => (field, None),
        };

        // The record name only contains letters, digits, and underscores
        if name.is_empty() {
            return Err(Error::InvalidHeader("Record name is empty".to_string()));
        }
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(Error::InvalidHeader(format!(
                "Record name '{name}' contains invalid characters, expected letters, digits, and underscores"
            )));
        }

        Ok((name.to_string(), num_segments))
    }

    /// Parse frequency definitions (optional)
    fn parse_frequency_field(field: Option<&str>) -> Result<(f64, Option<f64>, Option<f64>)> {
        let Some(field) = field else {
            // Return only default sampling frequency when omitted.
            return Ok((Self::DEFAULT_SAMPLING_FREQUENCY, None, None));
        };

        let (sampling_part, counter_part) = match field.split_once('/') {
            Some((s, c)) => (s, Some(c)),
            None => (field, None),
        };

        let sampling_frequency = sampling_part
            .parse()
            .map_err(|e| Error::InvalidHeader(format!("Invalid sampling frequency: {e}")))?;

        let (counter_frequency, base_counter) = match counter_part {
            Some(counter_str) => Self::parse_counter_frequency(counter_str)?,
            None => (None, None),
        };

        Ok((sampling_frequency, counter_frequency, base_counter))
    }

    /// Parse `counter_freq` or `counter_freq(base_counter)`
    fn parse_counter_frequency(field: &str) -> Result<(Option<f64>, Option<f64>)> {
        // Check for parentheses: counter_freq(base_counter)
        if let Some(paren_start) = field.find('(') {
            let paren_end = field.find(')').ok_or_else(|| {
                Error::InvalidHeader("Missing closing parenthesis in counter frequency".to_string())
            })?;

            let counter_freq = field[..paren_start]
                .parse()
                .map_err(|e| Error::InvalidHeader(format!("Invalid counter frequency: {e}")))?;

            let base_counter = field[paren_start + 1..paren_end]
                .parse()
                .map_err(|e| Error::InvalidHeader(format!("Invalid base counter value: {e}")))?;

            Ok((Some(counter_freq), Some(base_counter)))
        } else {
            let counter_freq = field
                .parse()
                .map_err(|e| Error::InvalidHeader(format!("Invalid counter frequency: {e}")))?;
            Ok((Some(counter_freq), None))
        }
    }

    /// Parse time in HH:MM:SS format
    fn parse_base_time(field: Option<&str>) -> Result<Option<NaiveTime>> {
        match field {
            Some(s) => {
                let time = NaiveTime::parse_from_str(s, "%H:%M:%S").map_err(|_| {
                    Error::InvalidHeader(format!("Invalid base time '{s}', expected HH:MM:SS"))
                })?;
                Ok(Some(time))
            }
            None => Ok(None),
        }
    }

    /// Parse date in DD/MM/YYYY format
    fn parse_base_date(field: Option<&str>) -> Result<Option<NaiveDate>> {
        match field {
            Some(s) => {
                let date = NaiveDate::parse_from_str(s, "%d/%m/%Y").map_err(|_| {
                    Error::InvalidHeader(format!("Invalid base date '{s}', expected DD/MM/YYYY"))
                })?;
                Ok(Some(date))
            }
            None => Ok(None),
        }
    }
}
