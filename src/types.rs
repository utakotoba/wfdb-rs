use crate::{Error, Result};

/// The format of a waveform signal data.
///
/// > Refer to [WFDB Format Specification](https://wfdb.io/spec/signal-files.html) for more details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalFormat {
    /// Null signal format, nothing to read or write.
    Format0,

    /// First differences stored as signed 8-bit integers.
    Format8,

    /// 16-bit two's complement integers (little-endian).
    Format16,

    /// 24-bit two's complement integers (little-endian).
    Format24,

    /// 32-bit two's complement integers (little-endian).
    Format32,

    /// 16-bit two's complement integers (big-endian).
    Format61,

    /// 8-bit offset binary (unsigned 8-bit, subtract 128 to recover).
    Format80,

    /// 16-bit offset binary (unsigned 16-bit, subtract 32,768 to recover).
    Format160,

    /// Packed 12-bit two's complement samples (compact format, common in `PhysioBank`).
    Format212,

    /// Packed 10-bit two's complement samples (legacy format).
    Format310,

    /// Alternative packed 10-bit samples (different packing from 310).
    Format311,

    /// Signals compressed with FLAC (8 bits per sample). (Format 508)
    Flac8,

    /// Signals compressed with FLAC (16 bits per sample). (Format 516)
    Flac16,

    /// Signals compressed with FLAC (24 bits per sample). (Format 524)
    Flac24,
}

impl SignalFormat {
    /// Converts a format code to a `SignalFormat` enum.
    ///
    /// # Errors
    ///
    /// Returns an error if the format code is not supported.
    pub const fn from_code(format_code: u16) -> Result<Self> {
        match format_code {
            0 => Ok(Self::Format0),
            8 => Ok(Self::Format8),
            16 => Ok(Self::Format16),
            24 => Ok(Self::Format24),
            32 => Ok(Self::Format32),
            61 => Ok(Self::Format61),
            80 => Ok(Self::Format80),
            160 => Ok(Self::Format160),
            212 => Ok(Self::Format212),
            310 => Ok(Self::Format310),
            311 => Ok(Self::Format311),
            508 => Ok(Self::Flac8),
            516 => Ok(Self::Flac16),
            524 => Ok(Self::Flac24),
            _ => Err(Error::UnsupportedSignalFormat(format_code)),
        }
    }

    /// Converts a `SignalFormat` enum to corresponding format code.
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            Self::Format0 => 0,
            Self::Format8 => 8,
            Self::Format16 => 16,
            Self::Format24 => 24,
            Self::Format32 => 32,
            Self::Format61 => 61,
            Self::Format80 => 80,
            Self::Format160 => 160,
            Self::Format212 => 212,
            Self::Format310 => 310,
            Self::Format311 => 311,
            Self::Flac8 => 508,
            Self::Flac16 => 516,
            Self::Flac24 => 524,
        }
    }
}
