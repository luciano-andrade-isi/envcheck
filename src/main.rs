use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

mod cli;
mod env;
mod output;
mod schema;
mod validation;

pub(crate) const EXIT_SUCCESS: u8 = 0;
pub(crate) const EXIT_VALIDATION_ERROR: u8 = 1;
pub(crate) const EXIT_INPUT_ERROR: u8 = 3;
pub(crate) const EXIT_SCHEMA_ERROR: u8 = 4;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreventingFailure {
    pub(crate) category: FailureCategory,
    pub(crate) path: PathBuf,
    pub(crate) reason: SafeFailureReason,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DefinitionPath {
    Example(PathBuf),
    Schema(PathBuf),
}

fn main() -> ExitCode {
    let cli = cli::Cli::parse();

    match resolve_definition(&cli) {
        Ok(DefinitionPath::Example(path)) => run_example(&cli.env_file, &path),
        Ok(DefinitionPath::Schema(path)) => run_schema(&cli.env_file, &path),
        Err(failure) => preventing_exit(failure),
    }
}

fn resolve_definition(cli: &cli::Cli) -> Result<DefinitionPath, PreventingFailure> {
    if let Some(schema_path) = &cli.schema {
        return Ok(DefinitionPath::Schema(schema_path.clone()));
    }
    if let Some(example_path) = &cli.example {
        return Ok(DefinitionPath::Example(example_path.clone()));
    }

    let target_directory = cli
        .env_file
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let sibling_schema = target_directory.join(".env.schema");
    if definition_exists(&sibling_schema)? {
        return Ok(DefinitionPath::Schema(sibling_schema));
    }

    let sibling_example = target_directory.join(".env.example");
    if definition_exists(&sibling_example)? {
        return Ok(DefinitionPath::Example(sibling_example));
    }

    Err(PreventingFailure {
        category: FailureCategory::Discovery,
        path: cli.env_file.clone(),
        reason: SafeFailureReason::NoValidationDefinition,
        line: None,
        column: None,
    })
}

fn definition_exists(path: &Path) -> Result<bool, PreventingFailure> {
    path.try_exists().map_err(|_| unreadable_failure(path))
}

fn run_example(target_path: &Path, example_path: &Path) -> ExitCode {
    let target_text = match read_input(target_path) {
        Ok(text) => text,
        Err(failure) => return preventing_exit(failure),
    };
    let example_text = match read_input(example_path) {
        Ok(text) => text,
        Err(failure) => return preventing_exit(failure),
    };

    let target = match env::parser::parse_dotenv(target_path, &target_text) {
        Ok(document) => document,
        Err(error) => return preventing_exit(dotenv_failure(error)),
    };
    let example = match env::parser::parse_dotenv(example_path, &example_text) {
        Ok(document) => document,
        Err(error) => return preventing_exit(dotenv_failure(error)),
    };

    completed_validation_exit(validation::validator::validate_example(&target, &example))
}

fn run_schema(target_path: &Path, schema_path: &Path) -> ExitCode {
    let target_text = match read_input(target_path) {
        Ok(text) => text,
        Err(failure) => return preventing_exit(failure),
    };
    let schema_text = match read_input(schema_path) {
        Ok(text) => text,
        Err(failure) => return preventing_exit(failure),
    };

    let target = match env::parser::parse_dotenv(target_path, &target_text) {
        Ok(document) => document,
        Err(error) => return preventing_exit(dotenv_failure(error)),
    };
    let schema = match schema::parser::parse_schema(schema_path, &schema_text) {
        Ok(schema) => schema,
        Err(error) => return preventing_exit(schema_failure(error)),
    };

    completed_validation_exit(validation::validator::validate_schema(&target, &schema))
}

fn completed_validation_exit(result: validation::ValidationResult) -> ExitCode {
    let exit = if result.has_errors() {
        EXIT_VALIDATION_ERROR
    } else {
        EXIT_SUCCESS
    };
    let _ = output::write_validation_stdout(&result);
    ExitCode::from(exit)
}

fn read_input(path: &Path) -> Result<String, PreventingFailure> {
    fs::read_to_string(path).map_err(|_| unreadable_failure(path))
}

fn unreadable_failure(path: &Path) -> PreventingFailure {
    PreventingFailure {
        category: FailureCategory::Input,
        path: path.to_path_buf(),
        reason: SafeFailureReason::UnreadableFile,
        line: None,
        column: None,
    }
}

fn dotenv_failure(error: env::parser::DotenvParseError) -> PreventingFailure {
    PreventingFailure {
        category: FailureCategory::Dotenv,
        path: error.path,
        reason: SafeFailureReason::InvalidDotenvSyntax,
        line: error.line,
        column: error.column,
    }
}

fn schema_failure(error: schema::parser::SchemaParseError) -> PreventingFailure {
    let reason = match error.kind {
        schema::parser::SchemaErrorKind::Syntax => SafeFailureReason::InvalidSchemaSyntax,
        schema::parser::SchemaErrorKind::Definition => SafeFailureReason::InvalidSchemaDefinition,
    };
    PreventingFailure {
        category: FailureCategory::Schema,
        path: error.path,
        reason,
        line: error.line,
        column: error.column,
    }
}

fn preventing_exit(failure: PreventingFailure) -> ExitCode {
    let exit = match failure.category {
        FailureCategory::Schema => EXIT_SCHEMA_ERROR,
        FailureCategory::Input | FailureCategory::Discovery | FailureCategory::Dotenv => {
            EXIT_INPUT_ERROR
        }
    };
    let _ = output::write_preventing_stderr(&failure);
    ExitCode::from(exit)
}
