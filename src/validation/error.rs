#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub(crate) const fn sort_rank(self) -> u8 {
        match self {
            Self::Error => 0,
            Self::Warning => 1,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuleCode {
    DuplicateKey,
    MissingRequired,
    EmptyNotAllowed,
    TypeMismatch,
    BelowMin,
    AboveMax,
    TooShort,
    TooLong,
    NotAllowed,
    PatternMismatch,
    AdditionalVariable,
}

impl RuleCode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::DuplicateKey => "duplicate_key",
            Self::MissingRequired => "missing_required",
            Self::EmptyNotAllowed => "empty_not_allowed",
            Self::TypeMismatch => "type_mismatch",
            Self::BelowMin => "below_min",
            Self::AboveMax => "above_max",
            Self::TooShort => "too_short",
            Self::TooLong => "too_long",
            Self::NotAllowed => "not_allowed",
            Self::PatternMismatch => "pattern_mismatch",
            Self::AdditionalVariable => "additional_variable",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Diagnostic {
    pub(crate) severity: Severity,
    pub(crate) code: RuleCode,
    pub(crate) variable: Option<String>,
    pub(crate) rule: String,
    pub(crate) expected: Option<String>,
    pub(crate) line: Option<usize>,
}
