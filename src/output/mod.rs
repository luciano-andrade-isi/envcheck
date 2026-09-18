use std::io::{self, Write};

use crate::PreventingFailure;
use crate::validation::ValidationResult;
use crate::validation::error::Diagnostic;

pub(crate) fn render_diagnostic(diagnostic: &Diagnostic) -> String {
    let mut rendered = format!("{}: ", diagnostic.severity.as_str());
    if let Some(variable) = &diagnostic.variable {
        rendered.push_str(variable);
        rendered.push_str(": ");
    }
    rendered.push_str(&diagnostic.rule);
    if let Some(expected) = &diagnostic.expected {
        rendered.push_str(" (expected ");
        rendered.push_str(expected);
        rendered.push(')');
    }
    rendered
}

pub(crate) fn render_validation_result(result: &ValidationResult) -> String {
    let mut ordered = result.clone();
    ordered.sort_diagnostics();

    let mut rendered = String::new();
    for diagnostic in &ordered.diagnostics {
        rendered.push_str(&render_diagnostic(diagnostic));
        rendered.push('\n');
    }

    if ordered.has_errors() {
        rendered.push_str("summary: environment validation failed\n");
    } else {
        rendered.push_str("ok: environment validation succeeded\n");
    }

    rendered
}

pub(crate) fn render_preventing_failure(failure: &PreventingFailure) -> String {
    let mut location = failure.path.display().to_string();
    if let Some(line) = failure.line {
        location.push(':');
        location.push_str(&line.to_string());
        if let Some(column) = failure.column {
            location.push(':');
            location.push_str(&column.to_string());
        }
    }

    format!(
        "error: {}: {}: {}\n",
        failure.category, location, failure.reason
    )
}

pub(crate) fn write_validation_stdout(result: &ValidationResult) -> io::Result<()> {
    let rendered = render_validation_result(result);
    io::stdout().lock().write_all(rendered.as_bytes())
}

pub(crate) fn write_preventing_stderr(failure: &PreventingFailure) -> io::Result<()> {
    let rendered = render_preventing_failure(failure);
    io::stderr().lock().write_all(rendered.as_bytes())
}
