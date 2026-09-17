use std::fmt;
use std::path::{Path, PathBuf};

use dotenvx_primitives::{ScanOptions, scan};

use super::{EnvDocument, EnvEntry};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DotenvParseError {
    pub(crate) path: PathBuf,
    pub(crate) reason: &'static str,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>,
}

impl DotenvParseError {
    fn at_line(path: &Path, line: usize, reason: &'static str) -> Self {
        Self {
            path: path.to_path_buf(),
            reason,
            line: Some(line),
            column: None,
        }
    }
}

impl fmt::Display for DotenvParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path.display(), self.reason)?;
        if let Some(line) = self.line {
            write!(formatter, " at line {line}")?;
            if let Some(column) = self.column {
                write!(formatter, ", column {column}")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for DotenvParseError {}

pub(crate) fn parse_dotenv(path: &Path, input: &str) -> Result<EnvDocument, DotenvParseError> {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let mut entries = Vec::new();

    for (line_index, line) in normalized.lines().enumerate() {
        let line_number = line_index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let (key, raw_value) = assignment_parts(line).ok_or_else(|| {
            DotenvParseError::at_line(path, line_number, "invalid dotenv declaration")
        })?;

        if !is_valid_identifier(key) {
            return Err(DotenvParseError::at_line(
                path,
                line_number,
                "invalid variable name",
            ));
        }

        if quoted_value_is_unterminated(raw_value) {
            return Err(DotenvParseError::at_line(
                path,
                line_number,
                "unterminated quoted value",
            ));
        }

        let scanned = scan(line, &ScanOptions::default());
        let scanned_value = scanned
            .parsed
            .get(key)
            .and_then(|values| (values.len() == 1).then(|| values[0].clone()))
            .ok_or_else(|| {
                DotenvParseError::at_line(path, line_number, "invalid dotenv declaration")
            })?;

        let value = if starts_with_supported_quote(raw_value) {
            scanned_value
        } else {
            clean_unquoted_value(raw_value)
        };

        entries.push(EnvEntry {
            key: key.to_owned(),
            value,
            line: line_number,
        });
    }

    Ok(EnvDocument::new(path.to_path_buf(), entries))
}

fn assignment_parts(line: &str) -> Option<(&str, &str)> {
    let mut input = line.trim_start();
    if let Some(rest) = input.strip_prefix("export ") {
        input = rest.trim_start();
    }

    let equals = input.find('=')?;
    let key = input[..equals].trim();
    if key.is_empty() {
        return None;
    }

    Some((key, &input[equals + 1..]))
}

fn is_valid_identifier(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn starts_with_supported_quote(raw_value: &str) -> bool {
    matches!(raw_value.trim_start().chars().next(), Some('\'' | '"'))
}

fn quoted_value_is_unterminated(raw_value: &str) -> bool {
    let input = raw_value.trim_start();
    let Some(quote @ ('\'' | '"')) = input.chars().next() else {
        return false;
    };

    let mut escaped = false;
    for character in input[quote.len_utf8()..].chars() {
        if character == quote && !escaped {
            return false;
        }
        escaped = character == '\\' && !escaped;
        if character != '\\' {
            escaped = false;
        }
    }

    true
}

fn clean_unquoted_value(raw_value: &str) -> String {
    let input = raw_value.trim();
    for (index, character) in input.char_indices() {
        if character != '#' {
            continue;
        }

        let preceded_by_whitespace = input[..index]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);
        if preceded_by_whitespace {
            return input[..index].trim_end().to_owned();
        }
    }

    input.to_owned()
}

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
