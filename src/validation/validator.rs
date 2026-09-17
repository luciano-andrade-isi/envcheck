use crate::env::EnvDocument;

use super::rules::presence::validate_example_presence;
use super::ValidationResult;

pub(crate) fn validate_example(target: &EnvDocument, example: &EnvDocument) -> ValidationResult {
    let mut result = validate_example_presence(target, example);
    result.sort_diagnostics();
    result
}
