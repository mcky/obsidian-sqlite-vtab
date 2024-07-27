use std::{ffi::OsStr, fs, path::PathBuf};
use walkdir::WalkDir;

use crate::{ObsidianNote, Properties, Records};

pub fn read_note(file_path: PathBuf) -> anyhow::Result<ObsidianNote> {
    let file_contents = fs::read_to_string(&file_path)?;

    let note = ObsidianNote {
        file_path,
        file_contents,
        properties: None,
    };

    Ok(note)
}

pub fn read_notes_in_dir(path: &str) -> anyhow::Result<Records> {
    let mut records: Records = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        // @TODO: Make this configurable
        .filter(|entry| entry.path().extension().and_then(OsStr::to_str) == Some("md"))
    {
        if entry.file_type().is_file() {
            let note = read_note(entry.into_path())?;

            records.push(note);
        }
    }

    Ok(records)
}

pub fn flatten_note(note: &ObsidianNote) -> Properties {
    let mut flattened = Properties::new();

    if let Some(properties) = &note.properties {
        flattened.extend(properties.clone());
    }

    flattened.insert(
        "file_path".to_string(),
        note.file_path.display().to_string(),
    );

    flattened.insert(
        "file_contents".to_string(),
        String::from(&note.file_contents),
    );

    flattened
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Context;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;

    fn copy_fixtures_to_tmp_dir(source_dir: &str) -> TempDir {
        let temp_dir = TempDir::new().expect("failed to create new TempDir");

        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            if entry.file_type().is_file() {
                let to = temp_dir.as_ref().join(entry.file_name());

                fs::copy(&entry.path(), &to)
                    .with_context(|| {
                        format!(
                            "attempting to copy {} into {}",
                            entry.path().display(),
                            to.display()
                        )
                    })
                    .expect("should be able to copy into tmp dir");
                ()
            }
        }

        temp_dir
    }

    #[test]
    fn read_notes_returns_note_records_from_dir() {
        let dir = TempDir::new().expect("failed to create new TempDir");

        let first_note = ObsidianNote {
            file_path: dir.join("first-note.md"),
            file_contents: "The contents of the file named first-note.md".to_string(),
            properties: None,
        };

        let second_note = ObsidianNote {
            file_path: dir.join("second-note.md"),
            file_contents: "The contents of the file named second-note.md".to_string(),
            properties: None,
        };

        dir.child("first-note.md")
            .write_str(&first_note.file_contents)
            .expect("failed to create file in TempDir");

        dir.child("second-note.md")
            .write_str(&second_note.file_contents)
            .expect("failed to create file in TempDir");

        let path = dir.path().to_str().unwrap();
        let notes = read_notes_in_dir(path).expect("couldn't read notes");

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
