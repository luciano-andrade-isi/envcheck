use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

mod cli;
// Foundational parser/domain APIs are wired into story flows by later tasks.
#[allow(dead_code)]
mod env;
// Foundational rendering primitives are wired into story flows by later tasks.
#[allow(dead_code)]
mod output;
mod schema;
// Foundational validation APIs are wired into story flows by later tasks.
#[allow(dead_code)]
mod validation;

pub(crate) const EXIT_SUCCESS: u8 = 0;
// Stable public exit constants are defined now and consumed by later orchestration tasks.
#[allow(dead_code)]
pub(crate) const EXIT_VALIDATION_ERROR: u8 = 1;
#[allow(dead_code)]
pub(crate) const EXIT_CLI_USAGE_ERROR: u8 = 2;
#[allow(dead_code)]
pub(crate) const EXIT_INPUT_ERROR: u8 = 3;
#[allow(dead_code)]
pub(crate) const EXIT_SCHEMA_ERROR: u8 = 4;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FailureCategory {
    Input,
    Discovery,
    Dotenv,
    Schema,
}

impl fmt::Display for FailureCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Input => "input",
            Self::Discovery => "discovery",
            Self::Dotenv => "dotenv",
            Self::Schema => "schema",
        };
        formatter.write_str(name)
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SafeFailureReason {
    UnreadableFile,
    NoValidationDefinition,
    InvalidDotenvSyntax,
    InvalidSchemaSyntax,
    InvalidSchemaDefinition,
}

impl fmt::Display for SafeFailureReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            Self::UnreadableFile => "required file could not be read",
            Self::NoValidationDefinition => "no validation definition is available",
            Self::InvalidDotenvSyntax => "invalid dotenv syntax",
            Self::InvalidSchemaSyntax => "invalid schema syntax",
            Self::InvalidSchemaDefinition => "invalid schema definition",
        };
        formatter.write_str(reason)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreventingFailure {
    pub(crate) category: FailureCategory,
    pub(crate) path: PathBuf,
    pub(crate) reason: SafeFailureReason,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>,
}

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    let _ = (&cli.env_file, &cli.example, &cli.schema);
    ExitCode::from(EXIT_SUCCESS)
}
