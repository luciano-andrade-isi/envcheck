use regex::Regex;

use crate::env::{EnvDocument, EnvEntry};
use crate::schema::model::{SchemaDefinition, SchemaScalar, VariableRule, VariableType};

use super::ValidationResult;
use super::error::{Diagnostic, RuleCode, Severity};
use super::rules::allowed::is_allowed;
use super::rules::length::validate_string_length;
use super::rules::numeric::{validate_float_bounds, validate_integer_bounds};
use super::rules::pattern::matches_pattern;
use super::rules::presence::{validate_example_presence, validate_schema_presence};
use super::rules::value_type::parse_scalar;

pub(crate) fn validate_example(target: &EnvDocument, example: &EnvDocument) -> ValidationResult {
    let mut result = validate_example_presence(target, example);
    result.sort_diagnostics();
    result
}

pub(crate) fn validate_schema(target: &EnvDocument, schema: &SchemaDefinition) -> ValidationResult {
    let mut diagnostics = Vec::new();

    add_target_duplicate_diagnostics(target, &mut diagnostics);

    for (variable, rule) in &schema.variables {
        let entry = match target.key_index.get(variable) {
            Some(indexes) if indexes.len() > 1 => continue,
            Some(indexes) => indexes.first().and_then(|index| target.entries.get(*index)),
            None => None,
        };

        let (presence_result, skip_value_rules) =
            validate_schema_presence(variable, entry, rule.required, rule.allow_empty);
        diagnostics.extend(presence_result.diagnostics);
        if skip_value_rules {
            continue;
        }

        let Some(entry) = entry else {
            continue;
        };
        let parsed = match parse_scalar(&entry.value, rule.r#type) {
            Ok(value) => value,
            Err(RuleCode::TypeMismatch) => {
                diagnostics.push(validation_error(
                    variable,
                    entry,
                    RuleCode::TypeMismatch,
                    "value does not match the declared type",
                    Some(variable_type_name(rule.r#type).to_owned()),
                ));
                continue;
            }
            Err(_) => unreachable!("scalar conversion only reports type mismatch"),
        };

        apply_type_constraints(variable, entry, rule, &parsed, &mut diagnostics);

        if let Some(allowed) = rule.allowed.as_deref()
            && !is_allowed(&parsed, allowed, rule.r#type)
        {
            diagnostics.push(validation_error(
                variable,
                entry,
                RuleCode::NotAllowed,
                "value is not in the configured allowed set",
                Some("one of the configured allowed values".to_owned()),
            ));
        }
    }

    for (variable, indexes) in &target.key_index {
        if schema.variables.contains_key(variable) {
            continue;
        }
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: RuleCode::AdditionalVariable,
            variable: Some(variable.clone()),
            rule: "variable is not declared in schema".to_owned(),
            expected: None,
            line: indexes
                .first()
                .and_then(|index| target.entries.get(*index))
                .map(|entry| entry.line),
        });
    }

    let mut result = ValidationResult { diagnostics };
    result.sort_diagnostics();
    result
}

fn add_target_duplicate_diagnostics(target: &EnvDocument, diagnostics: &mut Vec<Diagnostic>) {
    for (variable, indexes) in &target.key_index {
        if indexes.len() < 2 {
            continue;
        }
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: RuleCode::DuplicateKey,
            variable: Some(variable.clone()),
            rule: "duplicate variable declaration in target".to_owned(),
            expected: Some("a single declaration".to_owned()),
            line: indexes
                .get(1)
                .and_then(|index| target.entries.get(*index))
                .map(|entry| entry.line),
        });
    }
}

fn apply_type_constraints(
    variable: &str,
    entry: &EnvEntry,
    rule: &VariableRule,
    parsed: &SchemaScalar,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match (rule.r#type, parsed) {
        (VariableType::Integer, SchemaScalar::Integer(value)) => {
            let min = rule.min.as_ref().and_then(integer_bound);
            let max = rule.max.as_ref().and_then(integer_bound);
            for code in validate_integer_bounds(*value, min, max) {
                let expected = match code {
                    RuleCode::BelowMin => min.map(|bound| format!("integer >= {bound}")),
                    RuleCode::AboveMax => max.map(|bound| format!("integer <= {bound}")),
                    _ => None,
                };
                diagnostics.push(validation_error(
                    variable,
                    entry,
                    code,
                    numeric_rule_message(code),
                    expected,
                ));
            }
        }
        (VariableType::Float, SchemaScalar::Float(value)) => {
            let min = rule.min.as_ref();
            let max = rule.max.as_ref();
            for code in validate_float_bounds(*value, min, max) {
                let expected =
                    match code {
                        RuleCode::BelowMin => min
                            .map(|bound| format!("float >= {}", numeric_constraint_display(bound))),
                        RuleCode::AboveMax => max
                            .map(|bound| format!("float <= {}", numeric_constraint_display(bound))),
                        _ => None,
                    };
                diagnostics.push(validation_error(
                    variable,
                    entry,
                    code,
                    numeric_rule_message(code),
                    expected,
                ));
            }
        }
        (VariableType::String, SchemaScalar::String(value)) => {
            for code in validate_string_length(value, rule.min_length, rule.max_length) {
                let expected = match code {
                    RuleCode::TooShort => rule
                        .min_length
                        .map(|bound| format!("at least {bound} Unicode scalar values")),
                    RuleCode::TooLong => rule
                        .max_length
                        .map(|bound| format!("at most {bound} Unicode scalar values")),
                    _ => None,
                };
                let message = match code {
                    RuleCode::TooShort => "string is shorter than the configured minimum",
                    RuleCode::TooLong => "string is longer than the configured maximum",
                    _ => "string length constraint is violated",
                };
                diagnostics.push(validation_error(variable, entry, code, message, expected));
            }

            if let Some(pattern) = rule.pattern.as_deref()
                && let Ok(regex) = Regex::new(pattern)
                && !matches_pattern(value, &regex)
            {
                diagnostics.push(validation_error(
                    variable,
                    entry,
                    RuleCode::PatternMismatch,
                    "string does not match the configured pattern",
                    Some("a string matching the configured pattern".to_owned()),
                ));
            }
        }
        (VariableType::Boolean, SchemaScalar::Boolean(_)) => {}
        _ => unreachable!("parsed scalar kind must match the declared variable type"),
    }
}

fn integer_bound(value: &SchemaScalar) -> Option<i64> {
    match value {
        SchemaScalar::Integer(value) => Some(*value),
        _ => None,
    }
}

fn numeric_constraint_display(value: &SchemaScalar) -> String {
    match value {
        SchemaScalar::Integer(value) => value.to_string(),
        SchemaScalar::Float(value) if value.is_finite() => value.to_string(),
        _ => unreachable!("validated numeric constraint must be finite"),
    }
}

fn numeric_rule_message(code: RuleCode) -> &'static str {
    match code {
        RuleCode::BelowMin => "value is below the configured minimum",
        RuleCode::AboveMax => "value is above the configured maximum",
        _ => "numeric constraint is violated",
    }
}

fn validation_error(
    variable: &str,
    entry: &EnvEntry,
    code: RuleCode,
    rule: &str,
    expected: Option<String>,
) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        code,
        variable: Some(variable.to_owned()),
        rule: rule.to_owned(),
        expected,
        line: Some(entry.line),
    }
}

fn variable_type_name(variable_type: VariableType) -> &'static str {
    match variable_type {
        VariableType::String => "string",
        VariableType::Integer => "integer",
        VariableType::Float => "finite float",
        VariableType::Boolean => "boolean true/false",
    }
}
