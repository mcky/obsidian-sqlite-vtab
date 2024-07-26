use std::collections::HashMap;

pub fn sql_schema_from_properties(properties: HashMap<&str, &str>) -> String {
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
