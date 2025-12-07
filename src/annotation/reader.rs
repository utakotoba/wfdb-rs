use crate::{Result, Time};
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::Path;

use super::aha::AhaParser;
use super::format::{AnnotationFormat, AnnotationParser, detect_format};
use super::mit::MitParser;
use super::types::Annotation;
use crate::common::resolve_record_path;

/// A reader for WFDB annotation files.
///
/// This reader loads all annotations from a file into memory and provides
/// an iterator interface for accessing them. Annotations are stored sorted
/// by time.
///
/// # Example
///
/// ```no_run
/// use wfdb::annotation::AnnotationReader;
///
/// let mut reader = AnnotationReader::open("path/to/record", "atr")?;
/// for annotation in reader {
///     let annotation = annotation?;
///     println!("Time: {}, Code: {:?}", annotation.time, annotation.code);
/// }
/// # Ok::<(), wfdb::Error>(())
/// ```
#[derive(Debug)]
pub struct AnnotationReader {
    annotations: Vec<Annotation>,
    current_index: usize,
    position: Time,
}

impl AnnotationReader {
    /// Opens an annotation file for the specified record and annotator.
    ///
    /// The annotation file is expected to be located in the same directory as the
    /// record header file, with the name `{record_name}.{annotator}`.
    ///
    /// # Arguments
    ///
    /// * `record_path` - Path to the record header file (e.g., "100.hea") or record name.
    /// * `annotator` - The annotator name (e.g., "atr" for reference annotations).
    ///
    /// # Returns
    ///
    /// A new `AnnotationReader` instance, or an error if the file cannot be opened
    /// or parsed.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened, read, or if the format is invalid.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wfdb::annotation::AnnotationReader;
    ///
    /// // Read reference annotations
    /// let reader = AnnotationReader::open("100", "atr")?;
    /// # Ok::<(), wfdb::Error>(())
    /// ```
    pub fn open<P: AsRef<Path>>(record_path: P, annotator: &str) -> Result<Self> {
        let (_, record_dir, record_name) = resolve_record_path(record_path)?;

        // Construct annotation file path
        let annotation_file = record_dir.join(format!("{}.{}", record_name, annotator));

        // Open and read the annotation file
        let file = File::open(&annotation_file)?;
        let mut reader = BufReader::new(file);

        // Detect format
        let format = detect_format(&mut reader)?;

        // Parse annotations based on format
        reader.seek(SeekFrom::Start(0))?;
        let annotations = match format {
            AnnotationFormat::Mit => MitParser::parse(reader)?,
            AnnotationFormat::Aha => AhaParser::parse(reader)?,
        };

        Ok(Self {
            annotations,
            current_index: 0,
            position: 0,
        })
    }

    /// Seeks to a specific time position in the annotation stream.
    ///
    /// After seeking, the next annotation returned will be the first annotation
    /// at or after the specified time.
    ///
    /// # Arguments
    ///
    /// * `time` - The time (sample number) to seek to.
    pub fn seek_to_time(&mut self, time: Time) {
        self.current_index = self
            .annotations
            .binary_search_by_key(&time, |ann| ann.time)
            .unwrap_or_else(|idx| idx);

        self.position = if self.current_index < self.annotations.len() {
            self.annotations[self.current_index].time
        } else {
            time
        };
    }

    /// Returns the current time position.
    ///
    /// This is the time of the last annotation returned, or the seek position
    /// if no annotations have been read yet.
    #[must_use] 
    pub const fn position(&self) -> Time {
        self.position
    }

    /// Returns the total number of annotations.
    #[must_use] 
    pub const fn len(&self) -> usize {
        self.annotations.len()
    }

    /// Returns `true` if there are no annotations.
    #[must_use] 
    pub const fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }

    /// Returns a reference to all annotations.
    ///
    /// This provides access to all annotations without consuming the reader.
    #[must_use] 
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Returns an iterator over annotations in a time range.
    ///
    /// # Arguments
    ///
    /// * `start_time` - Start time (inclusive).
    /// * `end_time` - End time (inclusive).
    ///
    /// # Returns
    ///
    /// An iterator over annotations in the specified time range.
    pub fn range(&self, start_time: Time, end_time: Time) -> impl Iterator<Item = &Annotation> {
        self.annotations
            .iter()
            .skip_while(move |ann| ann.time < start_time)
            .take_while(move |ann| ann.time <= end_time)
    }

    /// Reads the next annotation.
    ///
    /// Returns `Some(annotation)` if an annotation is available,
    /// `None` if the end of the file has been reached.
    fn read_next(&mut self) -> Option<Annotation> {
        if self.current_index >= self.annotations.len() {
            return None;
        }

        let annotation = self.annotations[self.current_index].clone();
        self.current_index += 1;
        self.position = annotation.time;

        Some(annotation)
    }
}

impl Iterator for AnnotationReader {
    type Item = Result<Annotation>;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_next().map(Ok)
    }
}
