// T034/T035 will consume this parser; US3 adds strict semantic schema rejection later.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use super::model::SchemaDefinition;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SchemaParseError {
    pub(crate) path: PathBuf,
    pub(crate) reason: String,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>,
}

pub(crate) fn parse_schema(path: &Path, input: &str) -> Result<SchemaDefinition, SchemaParseError> {
    let schema = toml::from_str::<SchemaDefinition>(input)
        .map_err(|error| toml_error(path, input, &error))?;

    for variable in schema.variables.keys() {
        if !is_valid_variable_name(variable) {
            return Err(SchemaParseError {
                path: path.to_path_buf(),
                reason: format!("invalid variable name `{variable}`"),
                line: None,
                column: None,
            });
        }
    }

    Ok(schema)
}

fn toml_error(path: &Path, input: &str, error: &toml::de::Error) -> SchemaParseError {
    let location = error.span().map(|span| line_and_column(input, span.start));

    SchemaParseError {
        path: path.to_path_buf(),
        reason: error.message().to_owned(),
        line: location.map(|(line, _)| line),
        column: location.map(|(_, column)| column),
    }
}

fn line_and_column(input: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for (offset, character) in input.char_indices() {
        if offset >= byte_offset {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

fn is_valid_variable_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };

    (first == b'_' || first.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::parse_schema;

    #[test]
    fn parses_compatible_version_one_schema() {
        let schema = parse_schema(
            Path::new(".env.schema"),
            r#"
version = 1
[variables.APP_PORT]
type = "integer"
required = true
min = 1
max = 65535
"#,
        )
        .expect("compatible version-one schema should parse");

        assert_eq!(schema.version, 1);
        assert!(schema.variables.contains_key("APP_PORT"));
    }

    #[test]
    fn rejects_invalid_variable_identifier_at_parser_boundary() {
        let error = parse_schema(
            Path::new("config/.env.schema"),
            r#"
version = 1
[variables."BAD-NAME"]
type = "string"
"#,
        )
        .expect_err("invalid variable name should be rejected");

        assert_eq!(error.path, Path::new("config/.env.schema"));
        assert!(error.reason.contains("invalid variable name"));
        assert_eq!(error.line, None);
        assert_eq!(error.column, None);
    }
}
