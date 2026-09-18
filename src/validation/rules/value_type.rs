// Placeholder module for scalar type rules. Behavior begins in later phases.

use crate::schema::model::{SchemaScalar, VariableType};
use crate::validation::error::RuleCode;

// T034 will orchestrate scalar conversion; this phase validates the rule independently.
#[allow(dead_code)]
pub(crate) fn parse_scalar(
    value: &str,
    variable_type: VariableType,
) -> Result<SchemaScalar, RuleCode> {
    match variable_type {
        VariableType::String => Ok(SchemaScalar::String(value.to_owned())),
        VariableType::Integer => value
            .parse::<i64>()
            .map(SchemaScalar::Integer)
            .map_err(|_| RuleCode::TypeMismatch),
        VariableType::Float => value
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map(SchemaScalar::Float)
            .ok_or(RuleCode::TypeMismatch),
        VariableType::Boolean if value.eq_ignore_ascii_case("true") => {
            Ok(SchemaScalar::Boolean(true))
        }
        VariableType::Boolean if value.eq_ignore_ascii_case("false") => {
            Ok(SchemaScalar::Boolean(false))
        }
        VariableType::Boolean => Err(RuleCode::TypeMismatch),
    }
}

#[cfg(test)]
mod tests {
    use crate::schema::model::{SchemaScalar, VariableType};
    use crate::validation::error::RuleCode;

    use super::parse_scalar;

    fn assert_type_mismatch(value: &str, variable_type: VariableType) {
        assert_eq!(
            parse_scalar(value, variable_type),
            Err(RuleCode::TypeMismatch)
        );
    }

    #[test]
    fn string_values_preserve_parsed_text() {
        assert_eq!(
            parse_scalar("hello world", VariableType::String),
            Ok(SchemaScalar::String("hello world".to_owned()))
        );
    }

    #[test]
    fn integer_accepts_full_i64_boundaries() {
        assert_eq!(
            parse_scalar(&i64::MIN.to_string(), VariableType::Integer),
            Ok(SchemaScalar::Integer(i64::MIN))
        );
        assert_eq!(
            parse_scalar(&i64::MAX.to_string(), VariableType::Integer),
            Ok(SchemaScalar::Integer(i64::MAX))
        );
    }

    #[test]
    fn integer_rejects_underflow_overflow_decimal_and_scientific_notation() {
        for value in [
            "-9223372036854775809",
            "9223372036854775808",
            "1.0",
            "1e3",
            "-2E4",
        ] {
            assert_type_mismatch(value, VariableType::Integer);
        }
    }

    #[test]
    fn float_accepts_finite_decimal_and_scientific_notation() {
        assert_eq!(
            parse_scalar("1.5", VariableType::Float),
            Ok(SchemaScalar::Float(1.5))
        );
        assert_eq!(
            parse_scalar("1.5e2", VariableType::Float),
            Ok(SchemaScalar::Float(150.0))
        );
        assert_eq!(
            parse_scalar("-2E-1", VariableType::Float),
            Ok(SchemaScalar::Float(-0.2))
        );
    }

    #[test]
    fn float_rejects_nan_positive_infinity_negative_infinity_and_overflow() {
        for value in ["NaN", "inf", "+inf", "-inf", "1e400", "-1e400"] {
            assert_type_mismatch(value, VariableType::Float);
        }
    }

    #[test]
    fn boolean_accepts_true_false_case_insensitively_only() {
        for value in ["true", "TRUE", "True", "tRuE"] {
            assert_eq!(
                parse_scalar(value, VariableType::Boolean),
                Ok(SchemaScalar::Boolean(true))
            );
        }
        for value in ["false", "FALSE", "False", "fAlSe"] {
            assert_eq!(
                parse_scalar(value, VariableType::Boolean),
                Ok(SchemaScalar::Boolean(false))
            );
        }
        for value in ["1", "0", "yes", "no", "on", "off"] {
            assert_type_mismatch(value, VariableType::Boolean);
        }
    }
}
