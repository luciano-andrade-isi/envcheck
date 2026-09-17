pub(crate) mod error;
pub(crate) mod rules;
pub(crate) mod validator;

#[cfg(test)]
mod tests {
    use super::ValidationResult;
    use super::error::{Diagnostic, RuleCode, Severity};

    fn diagnostic(severity: Severity, variable: &str, code: RuleCode, line: usize) -> Diagnostic {
        Diagnostic {
            severity,
            code,
            variable: Some(variable.to_owned()),
            rule: "synthetic rule".to_owned(),
            expected: None,
            line: Some(line),
        }
    }

    #[test]
    fn sorts_diagnostics_canonically() {
        let mut result = ValidationResult {
            diagnostics: vec![
                diagnostic(Severity::Warning, "A", RuleCode::AdditionalVariable, 1),
                diagnostic(Severity::Error, "B", RuleCode::MissingRequired, 1),
                diagnostic(Severity::Error, "A", RuleCode::TypeMismatch, 1),
                diagnostic(Severity::Error, "A", RuleCode::MissingRequired, 4),
                diagnostic(Severity::Error, "A", RuleCode::MissingRequired, 2),
            ],
        };

        result.sort_diagnostics();

        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert_eq!(result.diagnostics[0].variable.as_deref(), Some("A"));
        assert_eq!(result.diagnostics[0].code, RuleCode::MissingRequired);
        assert_eq!(result.diagnostics[0].line, Some(2));

        assert_eq!(result.diagnostics[1].code, RuleCode::MissingRequired);
        assert_eq!(result.diagnostics[1].line, Some(4));

        assert_eq!(result.diagnostics[2].code, RuleCode::TypeMismatch);
        assert_eq!(result.diagnostics[3].variable.as_deref(), Some("B"));
        assert_eq!(result.diagnostics[4].severity, Severity::Warning);
    }

    #[test]
    fn warnings_only_do_not_count_as_errors() {
        let result = ValidationResult {
            diagnostics: vec![diagnostic(
                Severity::Warning,
                "LEGACY_FLAG",
                RuleCode::AdditionalVariable,
                1,
            )],
        };

        assert!(!result.has_errors());
    }
}
