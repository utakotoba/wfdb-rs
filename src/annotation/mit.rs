use super::types::Annotation;
use crate::{Error, Result};
use std::io::Read;

/// Pseudo-annotation codes used in MIT format.
const SKIP: u8 = 59; // Skip forward/backward in time
const NUM: u8 = 60; // Set annotator number
const SUB: u8 = 61; // Set subtype
const CHN: u8 = 62; // Set channel number
const AUX: u8 = 63; // Auxiliary information

/// MIT format annotation parser.
///
/// This parser implements parsing of the standard MIT (WFDB) annotation format.
pub struct MitParser;

impl super::format::AnnotationParser for MitParser {
    fn parse<R: Read>(mut reader: R) -> Result<Vec<Annotation>> {
        let mut annotations = Vec::new();
        let mut state = ParserState::default();

        loop {
            let mut bytes = [0u8; 2];
            match reader.read_exact(&mut bytes) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(Error::Io(e)),
            }

            let [b0, b1] = bytes;

            // Extract annotation type (bits 2-7 of byte1) and time difference
            let annotation_type = (b1 >> 2) & 0x3F;
            let time_diff = u16::from(b1 & 0x03) << 8 | u16::from(b0);

            if annotation_type == 0 && time_diff == 0 {
                break;
            }

            match annotation_type {
                SKIP => {
                    state.handle_skip(&mut reader)?;
                }
                NUM => {
                    state.handle_num(b0);
                }
                SUB => {
                    state.handle_sub(b0);
                }
                CHN => {
                    state.handle_chn(b0);
                }
                AUX => {
                    ParserState::handle_aux(&mut reader, b0, &mut annotations)?;
                }
                _ => {
                    state.handle_annotation(annotation_type, time_diff, &mut annotations)?;
                }
            }
        }

        Ok(annotations)
    }
}

/// Internal parser state for tracking current annotation context.
#[derive(Debug, Default)]
struct ParserState {
    time: crate::Time,
    num: i8,
    chan: u8,
    subtyp: i8,
}

impl ParserState {
    fn handle_skip<R: Read>(&mut self, reader: &mut R) -> Result<()> {
        let mut skip_bytes = [0u8; 4];
        reader.read_exact(&mut skip_bytes)?;
        let skip_time = crate::Time::from(u32::from_le_bytes(skip_bytes));
        self.time = skip_time;
        Ok(())
    }

    const fn handle_num(&mut self, byte0: u8) {
        self.num = i8::from_ne_bytes([byte0]);
    }

    const fn handle_sub(&mut self, byte0: u8) {
        self.subtyp = i8::from_ne_bytes([byte0]);
    }

    const fn handle_chn(&mut self, byte0: u8) {
        self.chan = byte0;
    }

    fn handle_aux<R: Read>(
        reader: &mut R,
        aux_len: u8,
        annotations: &mut [Annotation],
    ) -> Result<()> {
        let aux_len = aux_len as usize;
        if aux_len == 0 {
            return Ok(());
        }

        let mut aux_string = vec![0u8; aux_len];
        reader.read_exact(&mut aux_string)?;

        // Handle word alignment: if length is odd, skip one byte
        if aux_len & 1 != 0 {
            let mut pad = [0u8; 1];
            reader.read_exact(&mut pad)?;
        }

        let aux_str = String::from_utf8_lossy(&aux_string).to_string();

        if let Some(last_ann) = annotations.last_mut() {
            last_ann.aux = Some(aux_str);
        }

        Ok(())
    }

    fn handle_annotation(
        &mut self,
        annotation_type: u8,
        time_diff: u16,
        annotations: &mut Vec<Annotation>,
    ) -> Result<()> {
        // Update time by adding the time difference
        self.time = self.time.wrapping_add(crate::Time::from(time_diff));

        let code: super::types::AnnotationCode = annotation_type.try_into()?;

        let annotation = Annotation {
            time: self.time,
            code,
            subtyp: self.subtyp,
            chan: self.chan,
            num: self.num,
            aux: None,
        };

        annotations.push(annotation);

        self.subtyp = 0;

        Ok(())
    }
}
