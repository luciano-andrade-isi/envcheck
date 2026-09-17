// Placeholder module for numeric rules. Behavior begins in later phases.

#[cfg(test)]
mod tests {
    use crate::validation::error::RuleCode;

    use super::{validate_float_bounds, validate_integer_bounds};

    #[test]
    fn integer_min_and_max_are_inclusive_with_adjacent_values_rejected() {
        assert!(validate_integer_bounds(10, Some(10), Some(20)).is_empty());
        assert!(validate_integer_bounds(20, Some(10), Some(20)).is_empty());
        assert_eq!(
            validate_integer_bounds(9, Some(10), Some(20)),
            vec![RuleCode::BelowMin]
        );
        assert_eq!(
            validate_integer_bounds(21, Some(10), Some(20)),
            vec![RuleCode::AboveMax]
        );
    }

    #[test]
    fn integer_bound_comparison_preserves_i64_precision_beyond_f64_exact_range() {
        let exact = 9_007_199_254_740_993_i64;
        assert!(validate_integer_bounds(exact, Some(exact), Some(exact)).is_empty());
        assert_eq!(
            validate_integer_bounds(exact - 1, Some(exact), None),
            vec![RuleCode::BelowMin]
        );
        assert_eq!(
            validate_integer_bounds(exact + 1, None, Some(exact)),
            vec![RuleCode::AboveMax]
        );
    }

    #[test]
    fn float_min_and_max_are_inclusive_with_next_representable_values_rejected() {
        let min = 1.0_f64;
        let max = 2.0_f64;
        let just_below_min = f64::from_bits(min.to_bits() - 1);
        let just_above_max = f64::from_bits(max.to_bits() + 1);

        assert!(validate_float_bounds(min, Some(min), Some(max)).is_empty());
        assert!(validate_float_bounds(max, Some(min), Some(max)).is_empty());
        assert_eq!(
            validate_float_bounds(just_below_min, Some(min), Some(max)),
            vec![RuleCode::BelowMin]
        );
        assert_eq!(
            validate_float_bounds(just_above_max, Some(min), Some(max)),
            vec![RuleCode::AboveMax]
        );
    }
}
