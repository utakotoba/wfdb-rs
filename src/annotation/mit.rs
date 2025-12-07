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
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(Error::Io(e)),
            }

            let [byte0, byte1] = bytes;

            // Extract annotation type (bits 2-7 of byte1) and time difference
            let annotation_type = (byte1 >> 2) & 0x3F;
            let time_diff = ((byte1 & 0x03) as u16) << 8 | (byte0 as u16);

            if annotation_type == 0 && time_diff == 0 {
                break;
            }

            match annotation_type {
                SKIP => {
                    state.handle_skip(&mut reader)?;
                }
                NUM => {
                    state.handle_num(byte0);
                }
                SUB => {
                    state.handle_sub(byte0);
                }
                CHN => {
                    state.handle_chn(byte0);
                }
                AUX => {
                    state.handle_aux(&mut reader, byte0, &mut annotations)?;
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
    current_time: crate::Time,
    current_num: i8,
    current_chan: u8,
    current_subtyp: i8,
}

impl ParserState {
    fn handle_skip<R: Read>(&mut self, reader: &mut R) -> Result<()> {
        let mut skip_bytes = [0u8; 4];
        reader.read_exact(&mut skip_bytes)?;
        let skip_time = u32::from_le_bytes(skip_bytes) as crate::Time;
        self.current_time = skip_time;
        Ok(())
    }

    fn handle_num(&mut self, byte0: u8) {
        self.current_num = byte0 as i8;
    }

    fn handle_sub(&mut self, byte0: u8) {
        self.current_subtyp = byte0 as i8;
    }

    fn handle_chn(&mut self, byte0: u8) {
        self.current_chan = byte0;
    }

    fn handle_aux<R: Read>(
        &self,
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
        self.current_time = self.current_time.wrapping_add(time_diff as crate::Time);

        let code: super::types::AnnotationCode = annotation_type.try_into()?;

        let annotation = Annotation {
            time: self.current_time,
            code,
            subtyp: self.current_subtyp,
            chan: self.current_chan,
            num: self.current_num,
            aux: None,
        };

        annotations.push(annotation);

        self.current_subtyp = 0;

        Ok(())
    }
}
