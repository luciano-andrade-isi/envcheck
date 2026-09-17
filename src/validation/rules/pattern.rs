// Placeholder module for pattern rules. Behavior begins in later phases.

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
