use crate::{Error, Result, Time};
use std::convert::{From, TryFrom};

/// Maximum valid annotation code value.
///
/// Annotation codes must be in the range 1 to `ACMAX` (inclusive).
/// Code 0 (`NOTQRS`) is not a valid annotation code but is used internally.
pub const ACMAX: u8 = 49;

/// A single annotation from a WFDB annotation file.
///
/// This structure represents an annotation at a specific time point in a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    /// Annotation time, in sample intervals from the beginning of the record.
    pub time: Time,
    /// Annotation type code (must be in range 1 to `ACMAX`).
    pub code: AnnotationCode,
    /// Annotation subtype (signed 8-bit integer).
    pub subtyp: i8,
    /// Channel number (unsigned 8-bit integer).
    pub chan: u8,
    /// Annotator number (signed 8-bit integer).
    pub num: i8,
    /// Auxiliary information string, if present.
    pub aux: Option<String>,
}

impl Annotation {
    /// Creates a new annotation with the given time and code.
    ///
    /// All other fields are set to their default values (0 for numeric fields, None for aux).
    pub fn new(time: Time, code: AnnotationCode) -> Self {
        Self {
            time,
            code,
            subtyp: 0,
            chan: 0,
            num: 0,
            aux: None,
        }
    }

    /// Returns `true` if this annotation has auxiliary information.
    pub fn has_aux(&self) -> bool {
        self.aux.is_some()
    }

    /// Returns `true` if this annotation is associated with a specific channel.
    pub fn has_channel(&self) -> bool {
        self.chan != 0
    }
}

/// Annotation type codes.
///
/// These codes correspond to the predefined annotation types in `ecgcodes.h`.
/// Codes in the range 42-49 are user-defined and are represented as `Other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AnnotationCode {
    /// Not-QRS (not a valid annotation code, used internally).
    NotQrs = 0,
    /// Normal beat.
    Normal = 1,
    /// Left bundle branch block beat.
    Lbbb = 2,
    /// Right bundle branch block beat.
    Rbbb = 3,
    /// Aberrated atrial premature beat.
    Aberr = 4,
    /// Premature ventricular contraction.
    Pvc = 5,
    /// Fusion of ventricular and normal beat.
    Fusion = 6,
    /// Nodal (junctional) premature beat.
    Npc = 7,
    /// Atrial premature contraction.
    Apc = 8,
    /// Premature or ectopic supraventricular beat.
    Svpb = 9,
    /// Ventricular escape beat.
    Vesc = 10,
    /// Nodal (junctional) escape beat.
    Nesc = 11,
    /// Paced beat.
    Pace = 12,
    /// Unclassifiable beat.
    Unknown = 13,
    /// Signal quality change.
    Noise = 14,
    /// Isolated QRS-like artifact.
    Arfct = 16,
    /// ST change.
    Stch = 18,
    /// T-wave change.
    Tch = 19,
    /// Systole.
    Systole = 20,
    /// Diastole.
    Diastole = 21,
    /// Comment annotation.
    Note = 22,
    /// Measurement annotation.
    Measure = 23,
    /// P-wave peak.
    Pwave = 24,
    /// Left or right bundle branch block.
    Bbb = 25,
    /// Non-conducted pacer spike.
    Pacesp = 26,
    /// T-wave peak.
    Twave = 27,
    /// Rhythm change.
    Rhythm = 28,
    /// U-wave peak.
    Uwave = 29,
    /// Learning.
    Learn = 30,
    /// Ventricular flutter wave.
    Flwav = 31,
    /// Start of ventricular flutter/fibrillation.
    Vfon = 32,
    /// End of ventricular flutter/fibrillation.
    Vfoff = 33,
    /// Atrial escape beat.
    Aesc = 34,
    /// Supraventricular escape beat.
    Svesc = 35,
    /// Link to external data (aux contains URL).
    Link = 36,
    /// Non-conducted P-wave (blocked APB).
    Napc = 37,
    /// Fusion of paced and normal beat.
    Pfus = 38,
    /// Waveform onset / PQ junction (beginning of QRS).
    Wfon = 39,
    /// Waveform end / J point (end of QRS).
    Wfoff = 40,
    /// R-on-T premature ventricular contraction.
    Ront = 41,
    /// User-defined annotation code (42-49).
    Other(u8),
}

impl TryFrom<u8> for AnnotationCode {
    type Error = Error;

    fn try_from(code: u8) -> Result<Self> {
        match code {
            0 => Ok(AnnotationCode::NotQrs),
            1 => Ok(AnnotationCode::Normal),
            2 => Ok(AnnotationCode::Lbbb),
            3 => Ok(AnnotationCode::Rbbb),
            4 => Ok(AnnotationCode::Aberr),
            5 => Ok(AnnotationCode::Pvc),
            6 => Ok(AnnotationCode::Fusion),
            7 => Ok(AnnotationCode::Npc),
            8 => Ok(AnnotationCode::Apc),
            9 => Ok(AnnotationCode::Svpb),
            10 => Ok(AnnotationCode::Vesc),
            11 => Ok(AnnotationCode::Nesc),
            12 => Ok(AnnotationCode::Pace),
            13 => Ok(AnnotationCode::Unknown),
            14 => Ok(AnnotationCode::Noise),
            16 => Ok(AnnotationCode::Arfct),
            18 => Ok(AnnotationCode::Stch),
            19 => Ok(AnnotationCode::Tch),
            20 => Ok(AnnotationCode::Systole),
            21 => Ok(AnnotationCode::Diastole),
            22 => Ok(AnnotationCode::Note),
            23 => Ok(AnnotationCode::Measure),
            24 => Ok(AnnotationCode::Pwave),
            25 => Ok(AnnotationCode::Bbb),
            26 => Ok(AnnotationCode::Pacesp),
            27 => Ok(AnnotationCode::Twave),
            28 => Ok(AnnotationCode::Rhythm),
            29 => Ok(AnnotationCode::Uwave),
            30 => Ok(AnnotationCode::Learn),
            31 => Ok(AnnotationCode::Flwav),
            32 => Ok(AnnotationCode::Vfon),
            33 => Ok(AnnotationCode::Vfoff),
            34 => Ok(AnnotationCode::Aesc),
            35 => Ok(AnnotationCode::Svesc),
            36 => Ok(AnnotationCode::Link),
            37 => Ok(AnnotationCode::Napc),
            38 => Ok(AnnotationCode::Pfus),
            39 => Ok(AnnotationCode::Wfon),
            40 => Ok(AnnotationCode::Wfoff),
            41 => Ok(AnnotationCode::Ront),
            42..=49 => Ok(AnnotationCode::Other(code)),
            _ => Err(Error::InvalidAnnotationCode(code)),
        }
    }
}

impl From<AnnotationCode> for u8 {
    fn from(code: AnnotationCode) -> Self {
        match code {
            AnnotationCode::NotQrs => 0,
            AnnotationCode::Normal => 1,
            AnnotationCode::Lbbb => 2,
            AnnotationCode::Rbbb => 3,
            AnnotationCode::Aberr => 4,
            AnnotationCode::Pvc => 5,
            AnnotationCode::Fusion => 6,
            AnnotationCode::Npc => 7,
            AnnotationCode::Apc => 8,
            AnnotationCode::Svpb => 9,
            AnnotationCode::Vesc => 10,
            AnnotationCode::Nesc => 11,
            AnnotationCode::Pace => 12,
            AnnotationCode::Unknown => 13,
            AnnotationCode::Noise => 14,
            AnnotationCode::Arfct => 16,
            AnnotationCode::Stch => 18,
            AnnotationCode::Tch => 19,
            AnnotationCode::Systole => 20,
            AnnotationCode::Diastole => 21,
            AnnotationCode::Note => 22,
            AnnotationCode::Measure => 23,
            AnnotationCode::Pwave => 24,
            AnnotationCode::Bbb => 25,
            AnnotationCode::Pacesp => 26,
            AnnotationCode::Twave => 27,
            AnnotationCode::Rhythm => 28,
            AnnotationCode::Uwave => 29,
            AnnotationCode::Learn => 30,
            AnnotationCode::Flwav => 31,
            AnnotationCode::Vfon => 32,
            AnnotationCode::Vfoff => 33,
            AnnotationCode::Aesc => 34,
            AnnotationCode::Svesc => 35,
            AnnotationCode::Link => 36,
            AnnotationCode::Napc => 37,
            AnnotationCode::Pfus => 38,
            AnnotationCode::Wfon => 39,
            AnnotationCode::Wfoff => 40,
            AnnotationCode::Ront => 41,
            AnnotationCode::Other(code) => code,
        }
    }
}

impl AnnotationCode {
    /// Returns `true` if this is a valid annotation code for storage.
    ///
    /// `NotQrs` (0) is not a valid annotation code for storage.
    pub fn is_valid(&self) -> bool {
        !matches!(self, AnnotationCode::NotQrs)
    }

    /// Returns `true` if this is a user-defined annotation code.
    pub fn is_user_defined(&self) -> bool {
        matches!(self, AnnotationCode::Other(_))
    }
}
