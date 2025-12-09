use crate::{Error, Result};

/// Segment specification from a WFDB header segment line.
///
/// # Examples
///
/// Here are a few examples of validated segment specification lines:
///
/// - `100s 21600` _standard segment with record name and sample count_
/// - `null 1800` _null segment (invalid/missing data)_
/// - `~ 1800` _null segment (alternative notation)_
/// - `ecg_segment_01 360000` _named segment with 360000 samples_
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentInfo {
    /// Name of the record that comprises this segment.
    ///
    /// For null segments, this is "~".
    pub record_name: String,
    /// Number of samples per signal in this segment.
    pub num_samples: u64,
}

impl SegmentInfo {
    // [Segment info decoding functions]

    /// Build segment information from a segment specification line in a WFDB header.
    ///
    /// # Errors
    ///
    /// Will return an error if the format of the segment specification line is invalid.
    pub fn from_segment_line(line: &str) -> Result<Self> {
        let line = line.trim();
        let mut parts = line.split_whitespace();

        // First field: record name (required)
        let record_name = parts
            .next()
            .ok_or_else(|| Error::InvalidHeader("Missing record name".to_string()))?
            // After this, the record name is guaranteed to be non-empty
            .to_string();

        // Validate record name contains only letters, digits, underscores, or tilde
        if !Self::is_valid_record_name(&record_name) {
            return Err(Error::InvalidHeader(format!(
                "Invalid record name '{record_name}': must contain only letters, digits, underscores, or '~'"
            )));
        }

        // Second field: number of samples per signal (required)
        let num_samples_str = parts
            .next()
            .ok_or_else(|| Error::InvalidHeader("Missing number of samples".to_string()))?;

        let num_samples = num_samples_str
            .parse()
            .map_err(|e| Error::InvalidHeader(format!("Invalid number of samples: {e}")))?;

        // Check for extra fields
        if parts.next().is_some() {
            return Err(Error::InvalidHeader(
                "Extra fields found in segment specification line".to_string(),
            ));
        }

        Ok(Self {
            record_name,
            num_samples,
        })
    }

    /// Validate record name format.
    ///
    /// Valid record names contain only letters, digits, underscores, or '~' for null segments.
    fn is_valid_record_name(name: &str) -> bool {
        // Special case: "~" is valid for null segments
        if name == "~" {
            return true;
        }

        // Check all characters are alphanumeric or underscore
        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    // [Accessors]

    /// Get the record name of the segment.
    #[must_use]
    pub fn record_name(&self) -> &str {
        &self.record_name
    }

    /// Get the number of samples per signal in this segment.
    #[must_use]
    pub const fn num_samples(&self) -> u64 {
        self.num_samples
    }

    /// Check if this is a null segment.
    ///
    /// Null segments are identified by record name "~" and represent
    /// periods of invalid or missing data.
    #[must_use]
    pub fn is_null_segment(&self) -> bool {
        self.record_name == "~"
    }
}
