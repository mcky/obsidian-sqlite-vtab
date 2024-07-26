use std::{collections::HashMap, ffi::OsStr};
use walkdir::WalkDir;

use crate::Records;

pub fn read_notes(path: &str) -> anyhow::Result<Records> {
    let mut records: Records = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        // @TODO: Make this configurable
        .filter(|entry| entry.path().extension().and_then(OsStr::to_str) == Some("md"))
    {
        if entry.file_type().is_file() {
            let file_contents = std::fs::read_to_string(entry.path())?;

            let mut record: HashMap<String, String> = HashMap::new();

            record.insert("file_path".to_string(), entry.path().display().to_string());
            record.insert("file_contents".to_string(), file_contents);

            records.push(record);
        }
    }

    Ok(records)
}
