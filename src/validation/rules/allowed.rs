// Placeholder module for allowed-value rules. Behavior begins in later phases.

#[cfg(test)]
mod tests {
    use crate::schema::model::{SchemaScalar, VariableType};

    use super::is_allowed;

    #[test]
    fn string_allowed_comparison_is_case_sensitive() {
        let allowed = [SchemaScalar::String("prod".to_owned())];
        assert!(is_allowed(
            &SchemaScalar::String("prod".to_owned()),
            &allowed,
            VariableType::String
        ));
        assert!(!is_allowed(
            &SchemaScalar::String("PROD".to_owned()),
            &allowed,
            VariableType::String
        ));
    }

    #[test]
    fn integer_allowed_comparison_preserves_i64_without_f64_coercion() {
        let allowed = [SchemaScalar::Integer(9_007_199_254_740_993)];
        assert!(is_allowed(
            &SchemaScalar::Integer(9_007_199_254_740_993),
            &allowed,
            VariableType::Integer
        ));
        assert!(!is_allowed(
            &SchemaScalar::Integer(9_007_199_254_740_992),
            &allowed,
            VariableType::Integer
        ));
    }

    #[test]
    fn float_allowed_comparison_accepts_finite_toml_integer_or_float_entries() {
        let allowed = [SchemaScalar::Integer(1), SchemaScalar::Float(2.5)];
        assert!(is_allowed(
            &SchemaScalar::Float(1.0),
            &allowed,
            VariableType::Float
        ));
        assert!(is_allowed(
            &SchemaScalar::Float(2.5),
            &allowed,
            VariableType::Float
        ));
        assert!(!is_allowed(
            &SchemaScalar::Float(3.0),
            &allowed,
            VariableType::Float
        ));
    }

    #[test]
    fn boolean_allowed_comparison_uses_boolean_meaning() {
        let allowed = [SchemaScalar::Boolean(true)];
        assert!(is_allowed(
            &SchemaScalar::Boolean(true),
            &allowed,
            VariableType::Boolean
        ));
        assert!(!is_allowed(
            &SchemaScalar::Boolean(false),
            &allowed,
            VariableType::Boolean
        ));
    }

    #[test]
    fn incompatible_scalar_kinds_are_never_implicitly_coerced() {
        assert!(!is_allowed(
            &SchemaScalar::Integer(1),
            &[SchemaScalar::String("1".to_owned())],
            VariableType::Integer
        ));
        assert!(!is_allowed(
            &SchemaScalar::Boolean(true),
            &[SchemaScalar::String("true".to_owned())],
            VariableType::Boolean
        ));
        assert!(!is_allowed(
            &SchemaScalar::String("1".to_owned()),
            &[SchemaScalar::Integer(1)],
            VariableType::String
        ));
    }
}
