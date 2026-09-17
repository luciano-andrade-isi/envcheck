// Placeholder module for string-length rules. Behavior begins in later phases.

use crate::validation::error::RuleCode;

// T034 will orchestrate length rules; counting uses Unicode scalar values by contract.
#[allow(dead_code)]
pub(crate) fn validate_string_length(
    value: &str,
    min_length: Option<u64>,
    max_length: Option<u64>,
) -> Vec<RuleCode> {
    let length =
        u64::try_from(value.chars().count()).expect("supported target usize values fit within u64");
    let mut violations = Vec::new();

    if min_length.is_some_and(|minimum| length < minimum) {
        violations.push(RuleCode::TooShort);
    }
    if max_length.is_some_and(|maximum| length > maximum) {
        violations.push(RuleCode::TooLong);
    }

    violations
}

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
