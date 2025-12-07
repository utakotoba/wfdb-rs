use crate::{Error, Result};
use std::io::Read;

/// AHA format annotation parser.
///
/// > This is currently a stub implementation. AHA format parsing will be implemented
pub struct AhaParser;

impl super::format::AnnotationParser for AhaParser {
    fn parse<R: Read>(_reader: R) -> Result<Vec<super::Annotation>> {
        Err(Error::UnsupportedAnnotationFormat(
            "AHA format parsing is not yet implemented".to_string(),
        ))
    }
}
