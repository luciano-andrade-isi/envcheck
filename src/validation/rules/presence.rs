use crate::env::EnvDocument;
use crate::validation::ValidationResult;
use crate::validation::error::{Diagnostic, RuleCode, Severity};

pub(crate) fn validate_example_presence(
    target: &EnvDocument,
    example: &EnvDocument,
) -> ValidationResult {
    let mut diagnostics = Vec::new();

    add_duplicate_diagnostics(target, "target", &mut diagnostics);
    add_duplicate_diagnostics(example, "example", &mut diagnostics);

    for (key, indexes) in &example.key_index {
        if target.key_index.contains_key(key) {
            continue;
        }

        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: RuleCode::MissingRequired,
            variable: Some(key.clone()),
            rule: "required variable is missing".to_owned(),
            expected: Some("variable to be present".to_owned()),
            line: indexes
                .first()
                .and_then(|index| example.entries.get(*index))
                .map(|entry| entry.line),
        });
    }

    for (key, indexes) in &target.key_index {
        if example.key_index.contains_key(key) {
            continue;
        }

        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: RuleCode::AdditionalVariable,
            variable: Some(key.clone()),
            rule: "variable is not declared in example".to_owned(),
            expected: None,
            line: indexes
                .first()
                .and_then(|index| target.entries.get(*index))
                .map(|entry| entry.line),
        });
    }

    ValidationResult { diagnostics }
}

fn add_duplicate_diagnostics(
    document: &EnvDocument,
    document_kind: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (key, indexes) in &document.key_index {
        if indexes.len() < 2 {
            continue;
        }

        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: RuleCode::DuplicateKey,
            variable: Some(key.clone()),
            rule: format!("duplicate variable declaration in {document_kind}"),
            expected: Some("a single declaration".to_owned()),
            line: indexes
                .get(1)
                .and_then(|index| document.entries.get(*index))
                .map(|entry| entry.line),
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::env::{EnvDocument, EnvEntry};
    use crate::validation::ValidationResult;
    use crate::validation::error::{RuleCode, Severity};

    use super::validate_example_presence;

    fn document(path: &str, entries: &[(&str, &str, usize)]) -> EnvDocument {
        EnvDocument::new(
            PathBuf::from(path),
            entries
                .iter()
                .map(|(key, value, line)| EnvEntry {
                    key: (*key).to_owned(),
                    value: (*value).to_owned(),
                    line: *line,
                })
                .collect(),
        )
    }

    fn has_diagnostic(
        result: &ValidationResult,
        severity: Severity,
        code: RuleCode,
        variable: &str,
    ) -> bool {
        result.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == severity
                && diagnostic.code == code
                && diagnostic.variable.as_deref() == Some(variable)
        })
    }

    #[test]
    fn missing_example_key_is_an_error() {
        let target = document(".env", &[("APP_NAME", "actual", 1)]);
        let example = document(
            ".env.example",
            &[("APP_NAME", "example", 1), ("REDIS_URL", "ignored", 2)],
        );

        let result = validate_example_presence(&target, &example);

        assert!(result.has_errors());
        assert!(has_diagnostic(
            &result,
            Severity::Error,
            RuleCode::MissingRequired,
            "REDIS_URL"
        ));
    }

    #[test]
    fn reordered_keys_are_valid() {
        let target = document(
            ".env",
            &[
                ("DATABASE_URL", "actual-db", 1),
                ("APP_NAME", "actual-app", 2),
            ],
        );
        let example = document(
            ".env.example",
            &[
                ("APP_NAME", "example-app", 1),
                ("DATABASE_URL", "example-db", 2),
            ],
        );

        let result = validate_example_presence(&target, &example);

        assert!(!result.has_errors());
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn example_values_are_ignored() {
        let target = document(".env", &[("APP_NAME", "actual-value", 1)]);
        let example = document(".env.example", &[("APP_NAME", "different-value", 1)]);

        let result = validate_example_presence(&target, &example);

        assert!(!result.has_errors());
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn duplicate_target_key_is_an_error() {
        let target = document(".env", &[("PORT", "3000", 1), ("PORT", "4000", 2)]);
        let example = document(".env.example", &[("PORT", "ignored", 1)]);

        let result = validate_example_presence(&target, &example);

        assert!(result.has_errors());
        assert!(has_diagnostic(
            &result,
            Severity::Error,
            RuleCode::DuplicateKey,
            "PORT"
        ));
    }

    #[test]
    fn duplicate_example_key_is_an_error() {
        let target = document(".env", &[("PORT", "3000", 1)]);
        let example = document(
            ".env.example",
            &[("PORT", "first", 1), ("PORT", "second", 2)],
        );

        let result = validate_example_presence(&target, &example);

        assert!(result.has_errors());
        assert!(has_diagnostic(
            &result,
            Severity::Error,
            RuleCode::DuplicateKey,
            "PORT"
        ));
    }

    #[test]
    fn empty_example_is_valid_and_target_keys_are_warnings() {
        let target = document(".env", &[("ONLY_TARGET", "sensitive-value", 1)]);
        let example = document(".env.example", &[]);

        let result = validate_example_presence(&target, &example);

        assert!(!result.has_errors());
        assert!(has_diagnostic(
            &result,
            Severity::Warning,
            RuleCode::AdditionalVariable,
            "ONLY_TARGET"
        ));
    }

    #[test]
    fn matching_is_case_sensitive() {
        let target = document(".env", &[("port", "3000", 1)]);
        let example = document(".env.example", &[("PORT", "ignored", 1)]);

        let result = validate_example_presence(&target, &example);

        assert!(result.has_errors());
        assert!(has_diagnostic(
            &result,
            Severity::Error,
            RuleCode::MissingRequired,
            "PORT"
        ));
        assert!(has_diagnostic(
            &result,
            Severity::Warning,
            RuleCode::AdditionalVariable,
            "port"
        ));
    }

    #[test]
    fn target_only_key_is_an_additional_variable_warning() {
        let target = document(
            ".env",
            &[("APP_NAME", "actual", 1), ("LEGACY_FLAG", "enabled", 2)],
        );
        let example = document(".env.example", &[("APP_NAME", "ignored", 1)]);

        let result = validate_example_presence(&target, &example);

        assert!(!result.has_errors());
        assert!(has_diagnostic(
            &result,
            Severity::Warning,
            RuleCode::AdditionalVariable,
            "LEGACY_FLAG"
        ));
    }
}
