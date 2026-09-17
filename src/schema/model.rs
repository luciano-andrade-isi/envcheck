// T034/T035 intentionally wire these schema types later; this phase defines the model first.
#![allow(dead_code)]

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub(crate) struct SchemaDefinition {
    pub(crate) version: u32,
    pub(crate) variables: BTreeMap<String, VariableRule>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum VariableType {
    String,
    Integer,
    Float,
    Boolean,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub(crate) struct VariableRule {
    #[serde(rename = "type")]
    pub(crate) r#type: VariableType,
    #[serde(default)]
    pub(crate) required: bool,
    #[serde(default)]
    pub(crate) allow_empty: bool,
    pub(crate) min: Option<SchemaScalar>,
    pub(crate) max: Option<SchemaScalar>,
    pub(crate) min_length: Option<u64>,
    pub(crate) max_length: Option<u64>,
    pub(crate) allowed: Option<Vec<SchemaScalar>>,
    pub(crate) pattern: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub(crate) enum SchemaScalar {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

#[cfg(test)]
mod tests {
    use super::{SchemaDefinition, SchemaScalar, VariableRule, VariableType};

    fn parse_schema(input: &str) -> SchemaDefinition {
        toml::from_str(input).expect("schema should deserialize")
    }

    #[test]
    fn deserializes_schema_definition_variable_rules_types_and_scalar_kinds() {
        let schema = parse_schema(
            r#"
version = 1

[variables.APP_NAME]
type = "string"
required = true
allow_empty = true
min_length = 2
max_length = 20
allowed = ["dev", "prod"]
pattern = "^[a-z]+$"

[variables.APP_PORT]
type = "integer"
min = -10
max = 10
allowed = [-10, 0, 10]

[variables.RATIO]
type = "float"
min = 1
max = 2.5
allowed = [1, 1.5, 2]

[variables.DEBUG]
type = "boolean"
allowed = [true, false]
"#,
        );

        assert_eq!(schema.version, 1);
        assert_eq!(schema.variables.len(), 4);

        let string_rule: &VariableRule = &schema.variables["APP_NAME"];
        assert_eq!(string_rule.r#type, VariableType::String);
        assert!(string_rule.required);
        assert!(string_rule.allow_empty);
        assert_eq!(string_rule.min_length, Some(2));
        assert_eq!(string_rule.max_length, Some(20));
        assert_eq!(string_rule.pattern.as_deref(), Some("^[a-z]+$"));
        assert_eq!(
            string_rule.allowed.as_deref(),
            Some(
                [
                    SchemaScalar::String("dev".to_owned()),
                    SchemaScalar::String("prod".to_owned()),
                ]
                .as_slice()
            )
        );

        let integer_rule = &schema.variables["APP_PORT"];
        assert_eq!(integer_rule.r#type, VariableType::Integer);
        assert_eq!(integer_rule.min, Some(SchemaScalar::Integer(-10)));
        assert_eq!(integer_rule.max, Some(SchemaScalar::Integer(10)));
        assert_eq!(
            integer_rule.allowed.as_deref(),
            Some(
                [
                    SchemaScalar::Integer(-10),
                    SchemaScalar::Integer(0),
                    SchemaScalar::Integer(10),
                ]
                .as_slice()
            )
        );

        let float_rule = &schema.variables["RATIO"];
        assert_eq!(float_rule.r#type, VariableType::Float);
        assert_eq!(float_rule.min, Some(SchemaScalar::Integer(1)));
        assert_eq!(float_rule.max, Some(SchemaScalar::Float(2.5)));
        assert_eq!(
            float_rule.allowed.as_deref(),
            Some(
                [
                    SchemaScalar::Integer(1),
                    SchemaScalar::Float(1.5),
                    SchemaScalar::Integer(2),
                ]
                .as_slice()
            )
        );

        let boolean_rule = &schema.variables["DEBUG"];
        assert_eq!(boolean_rule.r#type, VariableType::Boolean);
        assert_eq!(
            boolean_rule.allowed.as_deref(),
            Some([SchemaScalar::Boolean(true), SchemaScalar::Boolean(false)].as_slice())
        );
    }

    #[test]
    fn required_and_allow_empty_default_to_false() {
        let schema = parse_schema(
            r#"
version = 1

[variables.OPTIONAL]
type = "string"
"#,
        );

        let rule = &schema.variables["OPTIONAL"];
        assert!(!rule.required);
        assert!(!rule.allow_empty);
        assert_eq!(rule.min, None);
        assert_eq!(rule.max, None);
        assert_eq!(rule.min_length, None);
        assert_eq!(rule.max_length, None);
        assert_eq!(rule.allowed, None);
        assert_eq!(rule.pattern, None);
    }
}
