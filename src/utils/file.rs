use std::fs::{read_dir,metadata};
use std::path::Path;

pub(crate) fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    metadata(path)
        .map(|meta| meta.is_file())
        .unwrap_or(false)
}

pub(crate) fn get_folders(path_str: &str) -> Vec<String> {
    let path = Path::new(path_str);
    let mut folder_names = Vec::new();
    let Ok(dir_entries) = read_dir(path) else {
        return folder_names;
    };
    for entry_result in dir_entries {
        let Ok(entry) = entry_result else { continue; };
        let Ok(metadata) = entry.metadata() else { continue; };
        if !metadata.is_dir() { continue; }
        if let Some(file_name) = entry.file_name().to_str() {
            folder_names.push(file_name.to_string());
        }
    }
    folder_names
}