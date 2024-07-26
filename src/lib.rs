use sqlite_loadable::table::{IndexInfo, VTab, VTabArguments, VTabCursor};
use sqlite_loadable::vtab_argparse::{parse_argument, Argument, ConfigOptionValue};
use sqlite_loadable::{api, define_virtual_table};
use sqlite_loadable::{define_scalar_function, prelude::*};
use sqlite_loadable::{BestIndexError, Error, Result};
use std::collections::HashMap;
use std::path::Path;
use std::{mem, os::raw::c_int};

mod notes;
mod obsidian_notes;
mod properties;

pub type Record = HashMap<String, String>;
pub type Records = Vec<Record>;
pub type Headers = Vec<String>;

#[sqlite_entrypoint]
pub fn sqlite3_obsidiansqlitevtab_init(db: *mut sqlite3) -> Result<()> {
    define_virtual_table::<obsidian_notes::ObsidianTable>(db, "obsidian_notes", None)?;
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
