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

#[cfg(test)]
mod us3_tests {
    use std::path::Path;

    use super::{SchemaParseError, parse_schema};

    fn schema_error(input: &str) -> SchemaParseError {
        parse_schema(Path::new("config/.env.schema"), input).expect_err("schema must be rejected")
    }

    #[test]
    fn us3_malformed_toml_preserves_schema_path_safe_reason_and_location() {
        let error = schema_error("version = 1\n[variables.VALUE\ntype = \"string\"\n");

        assert_eq!(error.path, Path::new("config/.env.schema"));
        assert!(!error.reason.trim().is_empty());
        assert!(
            error.line.is_some(),
            "TOML parser location should retain line"
        );
        assert!(
            error.column.is_some(),
            "TOML parser location should retain column"
        );
        assert!(!format!("{error:?}").contains("synthetic-target-secret"));
    }

    #[test]
    fn us3_unknown_top_level_property_is_rejected() {
        schema_error(
            r#"
version = 1
unexpected = true
[variables.VALUE]
type = "string"
"#,
        );
    }

    #[test]
    fn us3_unknown_variable_rule_property_is_rejected() {
        schema_error(
            r#"
version = 1
[variables.VALUE]
type = "string"
unexpected = true
"#,
        );
    }

    #[test]
    fn us3_missing_required_schema_structure_is_rejected() {
        schema_error("version = 1\n");
        schema_error(
            r#"
[variables.VALUE]
type = "string"
"#,
        );
    }

    #[test]
    fn us3_unsupported_variable_type_is_rejected() {
        schema_error(
            r#"
version = 1
[variables.VALUE]
type = "array"
"#,
        );
    }

    #[test]
    fn us3_unsupported_schema_version_is_rejected() {
        schema_error(
            r#"
version = 2
[variables.VALUE]
type = "string"
"#,
        );
    }

    #[test]
    fn us3_incompatible_constraint_for_declared_type_is_rejected() {
        schema_error(
            r#"
version = 1
[variables.VALUE]
type = "integer"
min_length = 1
"#,
        );
    }

    #[test]
    fn us3_wrong_constraint_scalar_kind_is_rejected() {
        schema_error(
            r#"
version = 1
[variables.VALUE]
type = "integer"
min = 1.5
"#,
        );
    }

    #[test]
    fn us3_contradictory_numeric_bounds_are_rejected() {
        schema_error(
            r#"
version = 1
[variables.VALUE]
type = "integer"
min = 20
max = 10
"#,
        );
    }

    #[test]
    fn us3_contradictory_string_length_bounds_are_rejected() {
        schema_error(
            r#"
version = 1
[variables.VALUE]
type = "string"
min_length = 5
max_length = 4
"#,
        );
    }

    #[test]
    fn us3_incompatible_typed_allowed_entries_are_rejected_without_coercion() {
        for schema in [
            r#"
version = 1
[variables.VALUE]
type = "string"
allowed = [1]
"#,
            r#"
version = 1
[variables.VALUE]
type = "integer"
allowed = [1.0]
"#,
            r#"
version = 1
[variables.VALUE]
type = "float"
allowed = ["1.0"]
"#,
            r#"
version = 1
[variables.VALUE]
type = "boolean"
allowed = [1]
"#,
        ] {
            schema_error(schema);
        }
    }

    #[test]
    fn us3_invalid_variable_identifier_is_a_schema_error() {
        let error = schema_error(
            r#"
version = 1
[variables."BAD-NAME"]
type = "string"
"#,
        );
        assert!(error.reason.contains("invalid variable name"));
    }

    #[test]
    fn us3_non_finite_float_constraints_are_rejected() {
        for bound in ["inf", "-inf", "nan"] {
            let schema =
                format!("version = 1\n[variables.VALUE]\ntype = \"float\"\nmin = {bound}\n");
            schema_error(&schema);
        }
    }
}
