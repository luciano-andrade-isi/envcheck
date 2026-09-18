use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "envcheck", version, about = "Validate environment files")]
pub(crate) struct Cli {
    #[arg(value_name = "ENV_FILE")]
    pub(crate) env_file: PathBuf,

    #[arg(long, value_name = "FILE", conflicts_with = "schema")]
    pub(crate) example: Option<PathBuf>,

    #[arg(long, value_name = "FILE", conflicts_with = "example")]
    pub(crate) schema: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::{Parser, error::ErrorKind};

    use super::Cli;

    #[test]
    fn env_file_is_required() {
        let error = Cli::try_parse_from(["envcheck"])
            .expect_err("ENV_FILE must be a mandatory positional argument");

        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn example_and_schema_flags_are_mutually_exclusive() {
        let error = Cli::try_parse_from([
            "envcheck",
            ".env",
            "--example",
            ".env.example",
            "--schema",
            ".env.schema",
        ])
        .expect_err("--example and --schema must conflict");

        assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn target_without_explicit_definition_is_a_valid_argument_shape() {
        let cli = Cli::try_parse_from(["envcheck", ".env"])
            .expect("definition discovery is allowed after CLI parsing");

        assert_eq!(cli.env_file, PathBuf::from(".env"));
        assert_eq!(cli.example, None);
        assert_eq!(cli.schema, None);
    }
}
