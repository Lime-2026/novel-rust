use std::fs;
use std::path::Path;

pub(crate) fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_file())
        .unwrap_or(false)
}