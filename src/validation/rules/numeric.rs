use std::cmp::Ordering;

use crate::schema::model::SchemaScalar;
use crate::validation::error::RuleCode;

pub(crate) fn validate_integer_bounds(
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
) -> Vec<RuleCode> {
    let mut violations = Vec::new();
    if min.is_some_and(|minimum| value < minimum) {
        violations.push(RuleCode::BelowMin);
    }
    if max.is_some_and(|maximum| value > maximum) {
        violations.push(RuleCode::AboveMax);
    }
    violations
}

pub(crate) fn compare_float_to_integer(value: f64, integer: i64) -> Ordering {
    assert!(
        value.is_finite(),
        "float/integer comparison requires a finite float"
    );

    const I64_MIN_AS_F64: f64 = -9_223_372_036_854_775_808.0;
    const I64_UPPER_EXCLUSIVE_AS_F64: f64 = 9_223_372_036_854_775_808.0;

    if value < I64_MIN_AS_F64 {
        return Ordering::Less;
    }
    if value >= I64_UPPER_EXCLUSIVE_AS_F64 {
        return Ordering::Greater;
    }

    let truncated = value as i64;
    match truncated.cmp(&integer) {
        Ordering::Equal => {
            let fraction = value.fract();
            if fraction < 0.0 {
                Ordering::Less
            } else if fraction > 0.0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }
        ordering => ordering,
    }
}

fn compare_float_to_bound(value: f64, bound: &SchemaScalar) -> Ordering {
    match bound {
        SchemaScalar::Integer(integer) => compare_float_to_integer(value, *integer),
        SchemaScalar::Float(float) => value
            .partial_cmp(float)
            .expect("validated float bounds and target values are finite"),
        SchemaScalar::String(_) | SchemaScalar::Boolean(_) => {
            unreachable!("validated float bounds must be numeric")
        }
    }
}

pub(crate) fn validate_float_bounds(
    value: f64,
    min: Option<&SchemaScalar>,
    max: Option<&SchemaScalar>,
) -> Vec<RuleCode> {
    let mut violations = Vec::new();
    if min.is_some_and(|minimum| compare_float_to_bound(value, minimum) == Ordering::Less) {
        violations.push(RuleCode::BelowMin);
    }
    if max.is_some_and(|maximum| compare_float_to_bound(value, maximum) == Ordering::Greater) {
        violations.push(RuleCode::AboveMax);
    }
    violations
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::schema::model::SchemaScalar;
    use crate::validation::error::RuleCode;

    use super::{compare_float_to_integer, validate_float_bounds, validate_integer_bounds};

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
        let min = SchemaScalar::Float(1.0);
        let max = SchemaScalar::Float(2.0);
        let just_below_min = f64::from_bits(1.0_f64.to_bits() - 1);
        let just_above_max = f64::from_bits(2.0_f64.to_bits() + 1);

        assert!(validate_float_bounds(1.0, Some(&min), Some(&max)).is_empty());
        assert!(validate_float_bounds(2.0, Some(&min), Some(&max)).is_empty());
        assert_eq!(
            validate_float_bounds(just_below_min, Some(&min), Some(&max)),
            vec![RuleCode::BelowMin]
        );
        assert_eq!(
            validate_float_bounds(just_above_max, Some(&min), Some(&max)),
            vec![RuleCode::AboveMax]
        );
    }

    #[test]
    fn float_integer_bounds_preserve_exact_numeric_meaning_beyond_two_to_the_53() {
        let odd_integer = SchemaScalar::Integer(9_007_199_254_740_993);

        assert_eq!(
            validate_float_bounds(9_007_199_254_740_992.0, Some(&odd_integer), None),
            vec![RuleCode::BelowMin]
        );
        assert_eq!(
            validate_float_bounds(9_007_199_254_740_994.0, None, Some(&odd_integer)),
            vec![RuleCode::AboveMax]
        );

        assert_eq!(
            compare_float_to_integer(9_007_199_254_740_992.0, 9_007_199_254_740_993),
            Ordering::Less
        );
        assert_eq!(
            compare_float_to_integer(9_007_199_254_740_994.0, 9_007_199_254_740_993),
            Ordering::Greater
        );
    }

    #[test]
    fn float_integer_comparison_handles_i64_extremes_without_rounding_them() {
        assert_eq!(
            compare_float_to_integer(-9_223_372_036_854_775_808.0, i64::MIN),
            Ordering::Equal
        );
        assert_eq!(
            compare_float_to_integer(9_223_372_036_854_775_808.0, i64::MAX),
            Ordering::Greater
        );
    }
}
