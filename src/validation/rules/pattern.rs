// Placeholder module for pattern rules. Behavior begins in later phases.

use regex::Regex;

// T034 will orchestrate pattern diagnostics; matching itself adds no implicit anchors.
#[allow(dead_code)]
pub(crate) fn matches_pattern(value: &str, pattern: &Regex) -> bool {
    pattern.is_match(value)
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::matches_pattern;

    #[test]
    fn pattern_uses_regex_is_match_substring_semantics_without_implicit_anchors() {
        let regex = Regex::new("cat").expect("valid regex");
        assert!(matches_pattern("concatenate", &regex));
    }

    #[test]
    fn schema_authored_anchors_are_respected() {
        let regex = Regex::new("^cat$").expect("valid regex");
        assert!(matches_pattern("cat", &regex));
        assert!(!matches_pattern("concatenate", &regex));
    }
}

#[cfg(test)]
mod us3_tests {
    use std::path::Path;

    use crate::schema::parser::parse_schema;

    #[test]
    fn us3_invalid_regex_is_rejected_as_schema_definition_error() {
        let error = parse_schema(
            Path::new("config/.env.schema"),
            r#"
version = 1
[variables.VALUE]
type = "string"
pattern = "["
"#,
        )
        .expect_err("invalid regex must invalidate the schema before target validation");

        assert_eq!(error.path, Path::new("config/.env.schema"));
        let reason = error.reason.to_ascii_lowercase();
        assert!(reason.contains("regex") || reason.contains("pattern"));
    }
}
