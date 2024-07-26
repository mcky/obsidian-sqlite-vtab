use std::{ffi::OsStr, fs};
use walkdir::WalkDir;

use crate::{Record, Records};

pub fn read_notes(path: &str) -> anyhow::Result<Records> {
    let mut records: Records = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        // @TODO: Make this configurable
        .filter(|entry| entry.path().extension().and_then(OsStr::to_str) == Some("md"))
    {
        if entry.file_type().is_file() {
            let file_contents = fs::read_to_string(entry.path())?;

            let mut record: Record = Record::new();

            record.insert("file_path".to_string(), entry.path().display().to_string());
            record.insert("file_contents".to_string(), file_contents);

            records.push(record);
        }
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use crate::Record;

    use super::*;

    use anyhow::Context;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use maplit::hashmap;

    #[test]
    fn read_notes_returns_note_records_from_dir() {
        let dir = TempDir::new().expect("failed to create new TempDir");

        let first_note: Record = hashmap! {
            "file_path".to_string() => dir.join("first-note.md").to_str().to_owned().unwrap().to_string(),
            "file_contents".to_string() => "The contents of the file named first-note.md".to_string(),
        };

        let second_note: Record = hashmap! {
            "file_path".to_string() => dir.join("second-note.md").to_str().to_owned().unwrap().to_string(),
            "file_contents".to_string() => "The contents of the file named second-note.md".to_string(),
        };

        dir.child("first-note.md")
            .write_str(&first_note["file_contents"])
            .expect("failed to create file in TempDir");

        dir.child("second-note.md")
            .write_str(&second_note["file_contents"])
            .expect("failed to create file in TempDir");

        let path = dir.path().to_str().unwrap();
        let notes = read_notes(path).expect("couldn't read notes");

        assert!(
            notes.contains(&first_note),
            "expected {notes:?} to contain {first_note:?}"
        );

        assert!(
            notes.contains(&second_note),
            "expected {notes:?} to contain {second_note:?}"
        );
    }
}
