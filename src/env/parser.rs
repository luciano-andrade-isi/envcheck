#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::parse_dotenv;

    fn parse(input: &str) -> crate::env::EnvDocument {
        parse_dotenv(Path::new(".env"), input).expect("dotenv input should parse")
    }

    #[test]
    fn ignores_blank_and_comment_only_lines() {
        let document = parse("\n# comment\nAPP_NAME=value\n   \n# second comment\nPORT=3000\n");

        assert_eq!(document.entries.len(), 2);
        assert_eq!(document.entries[0].key, "APP_NAME");
        assert_eq!(document.entries[0].value, "value");
        assert_eq!(document.entries[0].line, 3);
        assert_eq!(document.entries[1].key, "PORT");
        assert_eq!(document.entries[1].value, "3000");
        assert_eq!(document.entries[1].line, 6);
    }

    #[test]
    fn removes_common_quote_delimiters_without_losing_content() {
        let document = parse("SINGLE='value with # and ='\nDOUBLE=\"value with # and =\"\n");

        assert_eq!(document.entries[0].value, "value with # and =");
        assert_eq!(document.entries[1].value, "value with # and =");
    }

    #[test]
    fn accepts_export_prefix_and_crlf_line_endings() {
        let document = parse("export APP_NAME=value\r\nexport PORT=3000\r\n");

        assert_eq!(document.entries.len(), 2);
        assert_eq!(document.entries[0].key, "APP_NAME");
        assert_eq!(document.entries[0].line, 1);
        assert_eq!(document.entries[1].key, "PORT");
        assert_eq!(document.entries[1].line, 2);
    }

    #[test]
    fn preserves_empty_and_literal_interpolation_values() {
        let document = parse("EMPTY=\nREFERENCE=${NAME}\n");

        assert_eq!(document.entries[0].value, "");
        assert_eq!(document.entries[1].value, "${NAME}");
    }

    #[test]
    fn preserves_duplicate_assignments_in_source_order() {
        let document = parse("PORT=3000\nPORT=4000\n");

        assert_eq!(document.entries.len(), 2);
        assert_eq!(document.entries[0].key, "PORT");
        assert_eq!(document.entries[0].value, "3000");
        assert_eq!(document.entries[0].line, 1);
        assert_eq!(document.entries[1].key, "PORT");
        assert_eq!(document.entries[1].value, "4000");
        assert_eq!(document.entries[1].line, 2);
    }

    #[test]
    fn rejects_non_comment_lines_that_are_not_assignments() {
        let error = parse_dotenv(Path::new(".env"), "APP_NAME=value\nnot an assignment\n")
            .expect_err("malformed non-comment line must fail");

        assert_eq!(error.path, PathBuf::from(".env"));
        assert_eq!(error.line, Some(2));
    }

    #[test]
    fn enforces_identifier_grammar() {
        for invalid in ["1PORT=value\n", "APP-NAME=value\n", "APP.NAME=value\n"] {
            assert!(
                parse_dotenv(Path::new(".env"), invalid).is_err(),
                "{invalid}"
            );
        }

        for valid in ["_PORT=value\n", "APP_NAME=value\n", "A1=value\n"] {
            assert!(parse_dotenv(Path::new(".env"), valid).is_ok(), "{valid}");
        }
    }

    #[test]
    fn treats_variable_names_as_case_sensitive() {
        let document = parse("PORT=3000\nport=4000\n");

        assert_eq!(document.entries.len(), 2);
        assert_eq!(document.entries[0].key, "PORT");
        assert_eq!(document.entries[1].key, "port");
    }

    #[test]
    fn applies_inline_hash_semantics() {
        let document = parse(
            "COMMENTED=value # comment\nUNQUOTED=value#literal\nDOUBLE=\"value # literal\"\nSINGLE='value # literal'\n",
        );

        assert_eq!(document.entries[0].value, "value");
        assert_eq!(document.entries[1].value, "value#literal");
        assert_eq!(document.entries[2].value, "value # literal");
        assert_eq!(document.entries[3].value, "value # literal");
    }

    #[test]
    fn malformed_identifier_error_keeps_known_line_without_exposing_value() {
        let secret = "synthetic-secret-value";
        let input = format!("GOOD=ok\n# comment\n1SECRET={secret}\n");
        let error = parse_dotenv(Path::new(".env"), &input)
            .expect_err("invalid identifier must be rejected by Envcheck");

        assert_eq!(error.path, PathBuf::from(".env"));
        assert_eq!(error.line, Some(3));
        assert!(!error.reason.contains(secret));
        assert!(!format!("{error}").contains(secret));
        assert!(!format!("{error:?}").contains(secret));
    }
}
