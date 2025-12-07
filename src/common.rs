use crate::{Error, Result};
use std::path::{Path, PathBuf};

/// Resolves a record path to its components.
///
/// Returns `(header_path, base_dir, record_name)`.
///
/// # Arguments
///
/// * `path` - Path to the record (header file or record name).
pub(crate) fn resolve_record_path<P: AsRef<Path>>(path: P) -> Result<(PathBuf, PathBuf, String)> {
    let path = path.as_ref();
    
    if path.is_file() || path.extension().map_or(false, |ext| ext == "hea") {
        // It's likely a file path (e.g., "data/100.hea" or "data/100") pointing to a file
        // If it has .hea extension, easy.
        if path.extension().map_or(false, |ext| ext == "hea") {
            let dir = path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| Error::InvalidPath(format!("Invalid record path: {:?}", path)))?
                .to_string();
            Ok((path.to_path_buf(), dir, name))
        } else {
            // It's a file but not .hea? Maybe .dat? 
            // Usually we expect the argument to be the record name (no extension) or header file.
            // If user passes "100.dat", we shouldn't treat it as the record path typically.
            // But let's assume if it exists and no extension, or we treat it as record base.
            
            // If the user passed a path that exists and is a file, use its parent and stem.
            let dir = path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| Error::InvalidPath(format!("Invalid record path: {:?}", path)))?
                .to_string();
            
            // Header path is dir/name.hea
            let header_path = dir.join(format!("{}.hea", name));
            Ok((header_path, dir, name))
        }
    } else {
        // It's typically a record name string like "100" or "data/100"
        // We assume it refers to {path}.hea
        
        let dir = path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::InvalidPath(format!("Invalid record path: {:?}", path)))?
            .to_string();
            
        let header_path = if path.extension().map_or(false, |ext| ext == "hea") {
            path.to_path_buf()
        } else {
            // If path ends in .hea (handled above in is_file check? No, is_file only if it exists)
            // If path doesn't exist yet (creating?), we might still want to parse it.
            // But simpler: just append .hea if missing
            path.with_extension("hea")
        };
        
        Ok((header_path, dir, name))
    }
}

