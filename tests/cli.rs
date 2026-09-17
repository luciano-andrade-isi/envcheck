use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use assert_cmd::Command;
use tempfile::TempDir;

struct ExampleFixture {
    _directory: TempDir,
    target: PathBuf,
    example: PathBuf,
}

impl ExampleFixture {
    fn new(target: &str, example: &str) -> Self {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target_path = directory.path().join(".env");
        let example_path = directory.path().join(".env.example");
        fs::write(&target_path, target).expect("write target dotenv");
        fs::write(&example_path, example).expect("write example dotenv");
        Self {
            _directory: directory,
            target: target_path,
            example: example_path,
        }
    }
}

fn run_example(target: &Path, example: &Path) -> Output {
    let mut command = Command::cargo_bin("envcheck").expect("compiled envcheck binary");
    command.arg(target).arg("--example").arg(example);
    command.output().expect("execute envcheck")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_target_values_redacted(output: &Output, target_values: &[&str]) {
    let stdout = stdout(output);
    let stderr = stderr(output);
    for value in target_values {
        assert!(
            !stdout.contains(value),
            "stdout exposed target value: {value}"
        );
        assert!(
            !stderr.contains(value),
            "stderr exposed target value: {value}"
        );
    }
}

#[test]
fn explicit_example_success_exits_zero_and_reports_ok_without_values() {
    let fixture = ExampleFixture::new(
        "APP_NAME=synthetic-app-secret\nREDIS_URL=redis://synthetic-secret-host\n",
        "APP_NAME=example-name\nREDIS_URL=example-redis\n",
    );

    let output = run_example(&fixture.target, &fixture.example);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("ok: environment validation succeeded"));
    assert!(stderr(&output).is_empty());
    assert_target_values_redacted(
        &output,
        &["synthetic-app-secret", "redis://synthetic-secret-host"],
    );
}

#[test]
fn missing_redis_url_exits_one_without_exposing_target_values() {
    let fixture = ExampleFixture::new(
        "APP_NAME=synthetic-app-secret\nDATABASE_URL=postgres://synthetic-secret-db\n",
        "APP_NAME=ignored\nDATABASE_URL=ignored\nREDIS_URL=ignored\n",
    );

    let output = run_example(&fixture.target, &fixture.example);

    assert_eq!(output.status.code(), Some(1));
    let stdout = stdout(&output);
    assert!(stdout.contains("error"));
    assert!(stdout.contains("REDIS_URL"));
    assert_target_values_redacted(
        &output,
        &["synthetic-app-secret", "postgres://synthetic-secret-db"],
    );
}

#[test]
fn duplicate_target_declaration_exits_one_without_exposing_values() {
    let fixture = ExampleFixture::new("PORT=3000\nPORT=4000\n", "PORT=ignored\n");

    let output = run_example(&fixture.target, &fixture.example);

    assert_eq!(output.status.code(), Some(1));
    let stdout = stdout(&output);
    assert!(stdout.contains("error"));
    assert!(stdout.contains("PORT"));
    assert_target_values_redacted(&output, &["3000", "4000"]);
}

#[test]
fn target_only_variable_is_warning_only_and_exits_zero() {
    let fixture = ExampleFixture::new(
        "APP_NAME=synthetic-app-secret\nLEGACY_FLAG=synthetic-legacy-secret\n",
        "APP_NAME=ignored\n",
    );

    let output = run_example(&fixture.target, &fixture.example);

    assert_eq!(output.status.code(), Some(0));
    let stdout = stdout(&output);
    assert!(stdout.contains("warning"));
    assert!(stdout.contains("LEGACY_FLAG"));
    assert!(stdout.contains("ok: environment validation succeeded"));
    assert_target_values_redacted(
        &output,
        &["synthetic-app-secret", "synthetic-legacy-secret"],
    );
}

#[test]
fn malformed_target_dotenv_exits_three_with_safe_file_context() {
    let fixture = ExampleFixture::new(
        "API_KEY=synthetic-target-secret\nnot an assignment\n",
        "API_KEY=ignored\n",
    );

    let output = run_example(&fixture.target, &fixture.example);

    assert_eq!(output.status.code(), Some(3));
    let stderr = stderr(&output);
    assert!(stderr.contains(&fixture.target.display().to_string()));
    assert!(stderr.contains("invalid dotenv"));
    assert!(stderr.contains(":2"));
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}

#[test]
fn malformed_example_dotenv_exits_three_with_safe_file_context() {
    let fixture = ExampleFixture::new(
        "API_KEY=synthetic-target-secret\n",
        "API_KEY=ignored\nnot an assignment\n",
    );

    let output = run_example(&fixture.target, &fixture.example);

    assert_eq!(output.status.code(), Some(3));
    let stderr = stderr(&output);
    assert!(stderr.contains(&fixture.example.display().to_string()));
    assert!(stderr.contains("invalid dotenv"));
    assert!(stderr.contains(":2"));
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}
