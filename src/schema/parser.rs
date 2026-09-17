use std::path::{Path, PathBuf};

use regex::Regex;

use super::model::{SchemaDefinition, SchemaScalar, VariableRule, VariableType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SchemaErrorKind {
    Syntax,
    Definition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SchemaParseError {
    pub(crate) kind: SchemaErrorKind,
    pub(crate) path: PathBuf,
    pub(crate) reason: String,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>,
}

pub(crate) fn parse_schema(path: &Path, input: &str) -> Result<SchemaDefinition, SchemaParseError> {
    toml::from_str::<toml::Value>(input).map_err(|error| syntax_error(path, input, &error))?;

    let schema = toml::from_str::<SchemaDefinition>(input)
        .map_err(|error| definition_deserialization_error(path, input, &error))?;

    validate_schema_definition(path, &schema)?;
    Ok(schema)
}

fn validate_schema_definition(
    path: &Path,
    schema: &SchemaDefinition,
) -> Result<(), SchemaParseError> {
    if schema.version != 1 {
        return Err(definition_error(path, "unsupported schema version"));
    }

    for (variable, rule) in &schema.variables {
        if !is_valid_variable_name(variable) {
            return Err(definition_error(
                path,
                format!("invalid variable name `{variable}`"),
            ));
        }
        validate_rule(path, variable, rule)?;
    }

    Ok(())
}

fn validate_rule(path: &Path, variable: &str, rule: &VariableRule) -> Result<(), SchemaParseError> {
    match rule.r#type {
        VariableType::String => validate_string_rule(path, variable, rule)?,
        VariableType::Integer => validate_integer_rule(path, variable, rule)?,
        VariableType::Float => validate_float_rule(path, variable, rule)?,
        VariableType::Boolean => validate_boolean_rule(path, variable, rule)?,
    }

    if let Some(allowed) = rule.allowed.as_deref() {
        for item in allowed {
            let compatible = match rule.r#type {
                VariableType::String => matches!(item, SchemaScalar::String(_)),
                VariableType::Integer => matches!(item, SchemaScalar::Integer(_)),
                VariableType::Float => match item {
                    SchemaScalar::Integer(_) => true,
                    SchemaScalar::Float(value) => value.is_finite(),
                    _ => false,
                },
                VariableType::Boolean => matches!(item, SchemaScalar::Boolean(_)),
            };
            if !compatible {
                return Err(definition_error(
                    path,
                    format!("incompatible allowed entry for `{variable}`"),
                ));
            }
        }
    }

    Ok(())
}

fn validate_string_rule(
    path: &Path,
    variable: &str,
    rule: &VariableRule,
) -> Result<(), SchemaParseError> {
    if rule.min.is_some() || rule.max.is_some() {
        return Err(incompatible_constraint(path, variable, "min/max"));
    }
    if let (Some(min), Some(max)) = (rule.min_length, rule.max_length)
        && min > max
    {
        return Err(definition_error(
            path,
            format!("min_length exceeds max_length for `{variable}`"),
        ));
    }
    if let Some(pattern) = rule.pattern.as_deref()
        && Regex::new(pattern).is_err()
    {
        return Err(definition_error(
            path,
            format!("invalid regex pattern for `{variable}`"),
        ));
    }
    Ok(())
}

fn validate_integer_rule(
    path: &Path,
    variable: &str,
    rule: &VariableRule,
) -> Result<(), SchemaParseError> {
    if rule.min_length.is_some() || rule.max_length.is_some() || rule.pattern.is_some() {
        return Err(incompatible_constraint(path, variable, "string constraint"));
    }

    let min = match rule.min.as_ref() {
        Some(SchemaScalar::Integer(value)) => Some(*value),
        Some(_) => {
            return Err(definition_error(
                path,
                format!("integer min has incompatible scalar type for `{variable}`"),
            ));
        }
        None => None,
    };
    let max = match rule.max.as_ref() {
        Some(SchemaScalar::Integer(value)) => Some(*value),
        Some(_) => {
            return Err(definition_error(
                path,
                format!("integer max has incompatible scalar type for `{variable}`"),
            ));
        }
        None => None,
    };
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        return Err(definition_error(
            path,
            format!("min exceeds max for `{variable}`"),
        ));
    }
    Ok(())
}

fn validate_float_rule(
    path: &Path,
    variable: &str,
    rule: &VariableRule,
) -> Result<(), SchemaParseError> {
    if rule.min_length.is_some() || rule.max_length.is_some() || rule.pattern.is_some() {
        return Err(incompatible_constraint(path, variable, "string constraint"));
    }

    let min = float_constraint(path, variable, "min", rule.min.as_ref())?;
    let max = float_constraint(path, variable, "max", rule.max.as_ref())?;
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        return Err(definition_error(
            path,
            format!("min exceeds max for `{variable}`"),
        ));
    }
    Ok(())
}

fn validate_boolean_rule(
    path: &Path,
    variable: &str,
    rule: &VariableRule,
) -> Result<(), SchemaParseError> {
    if rule.min.is_some()
        || rule.max.is_some()
        || rule.min_length.is_some()
        || rule.max_length.is_some()
        || rule.pattern.is_some()
    {
        return Err(incompatible_constraint(path, variable, "value constraint"));
    }
    Ok(())
}

fn float_constraint(
    path: &Path,
    variable: &str,
    name: &str,
    value: Option<&SchemaScalar>,
) -> Result<Option<f64>, SchemaParseError> {
    match value {
        None => Ok(None),
        Some(SchemaScalar::Integer(value)) => Ok(Some(*value as f64)),
        Some(SchemaScalar::Float(value)) if value.is_finite() => Ok(Some(*value)),
        Some(SchemaScalar::Float(_)) => Err(definition_error(
            path,
            format!("non-finite float {name} for `{variable}`"),
        )),
        Some(_) => Err(definition_error(
            path,
            format!("float {name} has incompatible scalar type for `{variable}`"),
        )),
    }
}

fn incompatible_constraint(path: &Path, variable: &str, name: &str) -> SchemaParseError {
    definition_error(
        path,
        format!("incompatible {name} for declared type of `{variable}`"),
    )
}

fn syntax_error(path: &Path, input: &str, error: &toml::de::Error) -> SchemaParseError {
    let location = error.span().map(|span| line_and_column(input, span.start));
    SchemaParseError {
        kind: SchemaErrorKind::Syntax,
        path: path.to_path_buf(),
        reason: "malformed TOML schema".to_owned(),
        line: location.map(|(line, _)| line),
        column: location.map(|(_, column)| column),
    }
}

fn definition_deserialization_error(
    path: &Path,
    input: &str,
    error: &toml::de::Error,
) -> SchemaParseError {
    let location = error.span().map(|span| line_and_column(input, span.start));
    SchemaParseError {
        kind: SchemaErrorKind::Definition,
        path: path.to_path_buf(),
        reason: "invalid schema structure, property, or scalar type".to_owned(),
        line: location.map(|(line, _)| line),
        column: location.map(|(_, column)| column),
    }
}

fn definition_error(path: &Path, reason: impl Into<String>) -> SchemaParseError {
    SchemaParseError {
        kind: SchemaErrorKind::Definition,
        path: path.to_path_buf(),
        reason: reason.into(),
        line: None,
        column: None,
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
