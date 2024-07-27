use sqlite_loadable::Result;
use sqlite_loadable::{api, define_virtual_table};
use sqlite_loadable::{define_scalar_function, prelude::*};
use std::collections::HashMap;
use std::path::PathBuf;

mod notes;
mod obsidian_notes;
mod properties;

pub type Properties = HashMap<String, String>;
pub type Records = Vec<ObsidianNote>;
pub type Headers = Vec<String>;

#[derive(Debug, PartialEq)]
pub struct ObsidianNote {
    file_path: PathBuf,
    file_contents: String,
    properties: Option<Properties>,
}

#[sqlite_entrypoint]
pub fn sqlite3_obsidiansqlitevtab_init(db: *mut sqlite3) -> Result<()> {
    define_virtual_table::<obsidian_notes::ObsidianNotesTable>(db, "obsidian_notes", None)?;
    define_scalar_function(
        db,
        "obsidian_version",
        0,
        obsidian_version,
        FunctionFlags::DETERMINISTIC,
    )?;

    Ok(())
}

fn obsidian_version(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    let blurb = format!("v{}", env!("CARGO_PKG_VERSION"));
    api::result_text(context, blurb)?;

    Ok(())
}
