use crate::header::parse_header;
use crate::shared::Header;
use crate::{Error, Result, SignalReader};
use std::path::{Path, PathBuf};

/// A WFDB record, containing header information and a reader for signal data.
pub struct Record {
    pub header: Header,
    pub reader: SignalReader,
}

impl Record {
    /// Opens a single WFDB record from a path.
    ///
    /// # Errors
    ///
    /// Will return an error if the file cannot be opened, read, or if the format is unsupported.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let (header_path, base_dir, _) = resolve_record_path(path)?;
        let header = parse_header(&header_path)?;
        let reader = SignalReader::new(header.clone(), &base_dir)?;
        Ok(Self { header, reader })
    }
}

/// Iterator over WFDB records found in a directory.
pub struct RecordIterator {
    entries: std::fs::ReadDir,
}

impl Iterator for RecordIterator {
    type Item = Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.entries.next() {
                Some(Ok(entry)) => {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "hea") {
                        return Some(Record::open(&path));
                    }
                }
                Some(Err(e)) => return Some(Err(Error::Io(e))),
                None => return None,
            }
        }
    }
}

/// Opens a unified reader for a single record or a directory of records.
///
/// If `path` is a directory, returns an iterator over all records found in that directory (by finding `.hea` files).
/// If `path` is a file or record name, returns an iterator yielding just that single record.
///
/// # Example
///
/// ```no_run
/// // Iterate over all records in "data/"
/// for record in wfdb::open("data/")? {
///     let mut record = record?;
///     println!("Record: {}", record.header.metadata.name);
///     if let Some(samples) = record.reader.read_frame()? {
///         println!("First frame: {:?}", samples);
///     }
/// }
///
/// // Read a single record
/// for record in wfdb::open("data/100")? {
///     let mut record = record?;
///     println!("Single Record: {}", record.header.metadata.name);
/// }
/// # Ok::<(), wfdb::Error>(())
/// ```
///
/// # Errors
///
/// Will return an error if the file(s) cannot be opened, read.
pub fn open<P: AsRef<Path>>(path: P) -> Result<Box<dyn Iterator<Item = Result<Record>>>> {
    let path = path.as_ref();
    if path.is_dir() {
        let entries = std::fs::read_dir(path)?;
        Ok(Box::new(RecordIterator { entries }))
    } else {
        let record = Record::open(path)?;
        Ok(Box::new(std::iter::once(Ok(record))))
    }
}

/// Resolves a record path to its components.
///
/// Returns `(header_path, base_dir, record_name)`.
///
/// # Arguments
///
/// * `path` - Path to the record (header file or record name).
pub fn resolve_record_path<P: AsRef<Path>>(path: P) -> Result<(PathBuf, PathBuf, String)> {
    let path = path.as_ref();

    if path.is_file() || path.extension().is_some_and(|ext| ext == "hea") {
        if path.extension().is_some_and(|ext| ext == "hea") {
            let dir = path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| {
                    Error::InvalidPath(format!("Invalid record path: {}", path.display()))
                })?
                .to_string();
            Ok((path.to_path_buf(), dir, name))
        } else {
            let dir = path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| {
                    Error::InvalidPath(format!("Invalid record path: {}", path.display()))
                })?
                .to_string();

            // Header path is dir/name.hea
            let header_path = dir.join(format!("{name}.hea"));
            Ok((header_path, dir, name))
        }
    } else {
        let dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::InvalidPath(format!("Invalid record path: {}", path.display())))?
            .to_string();

        let header_path = if path.extension().is_some_and(|ext| ext == "hea") {
            path.to_path_buf()
        } else {
            path.with_extension("hea")
        };

        Ok((header_path, dir, name))
    }
}
