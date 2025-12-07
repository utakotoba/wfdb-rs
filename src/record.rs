use crate::common::resolve_record_path;
use crate::header::parse_header;
use crate::{Error, Header, Result, SignalReader};
use std::path::Path;

/// A WFDB record, containing header information and a reader for signal data.
pub struct Record {
    pub header: Header,
    pub reader: SignalReader,
}

impl Record {
    /// Opens a single WFDB record from a path.
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
                    if path.extension().map_or(false, |ext| ext == "hea") {
                        // Found a header file, try to open the record
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
pub fn open<P: AsRef<Path>>(path: P) -> Result<Box<dyn Iterator<Item = Result<Record>>>> {
    let path = path.as_ref();
    if path.is_dir() {
        let entries = std::fs::read_dir(path)?;
        Ok(Box::new(RecordIterator { entries }))
    } else {
        // Assume it's a record path (file or base name)
        // If it fails to open, we return the error in the iterator (or immediately?)
        // Ideally immediately if the path itself is invalid, but `Record::open` checks existence.
        // But `open` interface says "returns iterator".
        // Let's try to open it.
        let record = Record::open(path)?;
        Ok(Box::new(std::iter::once(Ok(record))))
    }
}
