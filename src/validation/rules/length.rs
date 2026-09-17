// Placeholder module for string-length rules. Behavior begins in later phases.

#[cfg(test)]
mod tests {
    use crate::validation::error::RuleCode;

    use super::validate_string_length;

    #[test]
    fn unicode_length_counts_scalar_values_not_utf8_bytes() {
        let value = "é🦀";
        assert_eq!(value.len(), 6);
        assert_eq!(value.chars().count(), 2);
        assert!(validate_string_length(value, Some(2), Some(2)).is_empty());
    }

    #[test]
    fn min_length_is_inclusive_and_one_scalar_below_is_rejected() {
        assert!(validate_string_length("é🦀", Some(2), None).is_empty());
        assert_eq!(
            validate_string_length("é", Some(2), None),
            vec![RuleCode::TooShort]
        );
    }

    #[test]
    fn max_length_is_inclusive_and_one_scalar_above_is_rejected() {
        assert!(validate_string_length("é🦀", None, Some(2)).is_empty());
        assert_eq!(
            validate_string_length("é🦀x", None, Some(2)),
            vec![RuleCode::TooLong]
        );
    }
}
