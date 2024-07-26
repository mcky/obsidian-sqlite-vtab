use sqlite_loadable::table::{IndexInfo, VTab, VTabArguments, VTabCursor};
use sqlite_loadable::vtab_argparse::{parse_argument, Argument, ConfigOptionValue};
use sqlite_loadable::{api, define_virtual_table};
use sqlite_loadable::{define_scalar_function, prelude::*};
use sqlite_loadable::{BestIndexError, Error, Result};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use std::{mem, os::raw::c_int};
use walkdir::WalkDir;

pub type Record = HashMap<String, String>;
pub type Records = Vec<Record>;
pub type Headers = Vec<String>;

#[sqlite_entrypoint]
pub fn sqlite3_obsidiansqlitevtab_init(db: *mut sqlite3) -> Result<()> {
    define_virtual_table::<ObsidianTable>(db, "obsidian", None)?;
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

#[repr(C)]
pub struct ObsidianTable {
    /// must be first
    base: sqlite3_vtab,
    db: *mut sqlite3,
    path: String,
}

impl<'vtab> VTab<'vtab> for ObsidianTable {
    type Aux = u8;
    type Cursor = ObsidianCursor;

    fn create(
        db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        args: VTabArguments,
    ) -> Result<(String, Self)> {
        Self::connect(db, aux, args)
    }

    fn connect(
        db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        args: VTabArguments,
    ) -> Result<(String, Self)> {
        let arguments = parse_arguments(db, args.arguments)?;
        let mut property_types = HashMap::new();

        let schema = sql_schema_from_properties(property_types);

        let vtab = Self {
            base: unsafe { mem::zeroed() },
            db,
            path: arguments.dirname,
        };

        Ok((schema, vtab))
    }

    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        // TODO: No matter how the set is queried, always jsut read from top to bottom,
        info.set_estimated_cost(10000.0);
        info.set_estimated_rows(10000);
        // info.set_idxnum(1);

        Ok(())
    }

    fn open(&mut self) -> Result<Self::Cursor> {
        let records = read_data(&self.path).map_err(|err| Error::new_message(&err.to_string()))?;

        Self::Cursor::new(records)
    }
}

#[repr(C)]
pub struct ObsidianCursor {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    /// The record headers (i.e. the column names)
    headers: Headers,
    records: Records,
    /// Current cursor position used as rowid
    rowid: i64,
    eof: bool,
}

impl ObsidianCursor {
    fn new(records: Records) -> Result<Self> {
        let mut cursor = Self {
            base: unsafe { mem::zeroed() },
            headers: vec![
                "file_path".to_string(),
                "file_contents".to_string(),
            ],
            records,
            rowid: 0,
            eof: false,
        };

        cursor.next().map(|_| cursor)
    }
}

impl VTabCursor for ObsidianCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        _values: &[*mut sqlite3_value],
    ) -> Result<()> {
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        if self.rowid as usize == self.records.len() {
            self.eof = true;

            return Ok(());
        }

        self.rowid += 1;

        Ok(())
    }

    fn eof(&self) -> bool {
        self.eof
    }

    fn column(&self, ctx: *mut sqlite3_context, col_idx: c_int) -> Result<()> {
        if col_idx < 0 || col_idx as usize >= self.headers.len() {
            return Err(Error::new_message(&format!(
                "column index out of bounds: {col_idx}"
            )));
        }

        let row_idx = (self.rowid - 1) as usize;

        if let Some(record) = &self.records.get(row_idx) {
            let col_name = &self.headers[col_idx as usize];
            if let Some(sx) = record.get(col_name) {
                api::result_text(ctx, sx)?;
            } else {
                api::result_null(ctx);
            }
        } else {
            api::result_null(ctx);
        }

        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid)
    }
}

#[derive(Debug, PartialEq)]
struct ObsidianArguments {
    dirname: String,
}

fn parse_arguments(db: *mut sqlite3, arguments: Vec<String>) -> Result<ObsidianArguments> {
    let mut dirname: Option<String> = None;

    for arg in arguments {
        match parse_argument(arg.as_str()) {
            Ok(arg) => match arg {
                Argument::Config(config) => match config.key.as_str() {
                    "dirname" => {
                        let value = parse_path(db, config.value)?;

                        dirname = Some(value);
                    }
                    _ => (),
                },
                Argument::Column(_column) => (),
            },
            Err(err) => return Err(Error::new_message(err.as_str())),
        };
    }

    let dirname = dirname.ok_or_else(|| Error::new_message("no dirname given. Specify a path to a directory containing an obsidian vault to read from. E.g. 'dirname=\"my_vault\"'"))?;

    Ok(ObsidianArguments { dirname })
}

pub fn parse_path(_db: *mut sqlite3, value: ConfigOptionValue) -> Result<String> {
    let value = match value {
        ConfigOptionValue::Quoted(value) => Ok(value),
        // ConfigOptionValue::SqliteParameter(value) => {
        //     match sqlite_parameter_value(db, value.as_str()) {
        //         Ok(result) => match result {
        //             Some(path) => Ok(path),
        //             None => Err(Error::new_message(
        //                 format!("{value} is not defined in temp.sqlite_parameters table").as_str(),
        //             )),
        //         },
        //         Err(_) => Err(Error::new_message(
        //             "temp.sqlite_parameters is not defined, can't use sqlite parameters as value",
        //         )),
        //     }
        // }
        _ => Err(Error::new_message(
            "'dirname' value must be a string. Wrap in single or double quotes.",
        )),
    }?;

    if !Path::new(&value).exists() {
        return Err(Error::new_message(
            &format!("dir '{value}' does not exist",),
        ));
    }

    Ok(value)
}

fn read_data(path: &str) -> anyhow::Result<Records> {
    let mut records: Records = Vec::new();

    for entry in WalkDir::new(path) {
        let entry = entry?;
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

fn sql_schema_from_properties(properties: HashMap<&str, &str>) -> String {
    let mut props = properties.clone();
    props.insert("file_path", "TEXT");
    props.insert("file_contents", "TEXT");

    sql_schema_from_map(props)
}

fn sql_schema_from_map(properties: HashMap<&str, &str>) -> String {
    let mut columns = properties
        .into_iter()
        .map(|(k, v)| format!("\"{k}\" {v}"))
        .collect::<Vec<String>>();

    columns.sort();

    format!(
        "CREATE TABLE x(\n    {columns}\n);",
        columns = columns.join(",\n    ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::collections::HashMap;

    #[test]
    fn sql_schema_from_properties_always_returns_base() {
        let out = sql_schema_from_properties(HashMap::new());

        assert_eq!(
            out,
            indoc! { r#"
            CREATE TABLE x(
                "file_contents" TEXT,
                "file_path" TEXT
            );"#
            }
        )
    }

    #[test]
    fn sql_schema_from_properties_returns_properties() {
        let mut properties = HashMap::new();
        properties.insert("str_key", "TEXT");
        properties.insert("int_key", "INTEGER");

        let out = sql_schema_from_properties(properties);

        assert_eq!(
            out,
            indoc! { r#"
            CREATE TABLE x(
                "file_contents" TEXT,
                "file_path" TEXT,
                "int_key" INTEGER,
                "str_key" TEXT
            );"#
            }
        )
    }
}
