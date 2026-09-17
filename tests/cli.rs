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

struct SchemaFixture {
    _directory: TempDir,
    target: PathBuf,
    schema: PathBuf,
}

impl SchemaFixture {
    fn new(target: &str, schema: &str) -> Self {
        let directory = tempfile::tempdir().expect("temporary directory");
        let target_path = directory.path().join(".env");
        let schema_path = directory.path().join(".env.schema");
        fs::write(&target_path, target).expect("write target dotenv");
        fs::write(&schema_path, schema).expect("write env schema");
        Self {
            _directory: directory,
            target: target_path,
            schema: schema_path,
        }
    }
}

fn run_schema(target: &Path, schema: &Path) -> Output {
    let mut command = Command::cargo_bin("envcheck").expect("compiled envcheck binary");
    command.arg(target).arg("--schema").arg(schema);
    command.output().expect("execute envcheck")
}

#[test]
fn schema_complete_valid_contract_accepts_all_types_and_dotenv_forms() {
    let fixture = SchemaFixture::new(
        "APP_NAME=\"My Application\"\nAPP_PORT=65535\nREQUEST_TIMEOUT=1.5e1\nDEBUG=TrUe\nHOST=api.internal\nEMPTY_OK=\nLITERAL=${NAME}\nexport EXPORTED=enabled\n",
        r#"
version = 1

[variables.APP_NAME]
type = "string"
required = true
min_length = 2
max_length = 20
allowed = ["My Application"]

[variables.APP_PORT]
type = "integer"
required = true
min = 1
max = 65535
allowed = [65535]

[variables.REQUEST_TIMEOUT]
type = "float"
required = true
min = 0.1
max = 60.0
allowed = [15, 15.0]

[variables.DEBUG]
type = "boolean"
required = true
allowed = [true]

[variables.HOST]
type = "string"
required = true
pattern = "api\\.internal"

[variables.EMPTY_OK]
type = "integer"
required = true
allow_empty = true
min = 10

[variables.LITERAL]
type = "string"
required = true
allowed = ["${NAME}"]

[variables.EXPORTED]
type = "string"
required = true
allowed = ["enabled"]

[variables.OPTIONAL]
type = "boolean"
"#,
    );

    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("ok: environment validation succeeded"));
    assert!(stderr(&output).is_empty());
    assert_target_values_redacted(
        &output,
        &["My Application", "api.internal", "${NAME}", "enabled"],
    );
}

#[test]
fn schema_required_missing_is_a_validation_error() {
    let fixture = SchemaFixture::new(
        "",
        r#"
version = 1
[variables.REQUIRED_VALUE]
type = "string"
required = true
"#,
    );

    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("REQUIRED_VALUE"));
    assert!(stdout(&output).contains("missing"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn schema_existing_empty_is_distinct_and_rejected_when_not_allowed() {
    let fixture = SchemaFixture::new(
        "REQUIRED_VALUE=\n",
        r#"
version = 1
[variables.REQUIRED_VALUE]
type = "string"
required = true
allow_empty = false
"#,
    );

    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("REQUIRED_VALUE"));
    assert!(stdout(&output).contains("empty"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn schema_optional_missing_variable_is_valid() {
    let fixture = SchemaFixture::new(
        "",
        r#"
version = 1
[variables.OPTIONAL_VALUE]
type = "integer"
"#,
    );

    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("ok: environment validation succeeded"));
}

#[test]
fn schema_integer_and_float_bounds_are_inclusive_and_reject_outside_values() {
    let integer_schema = r#"
version = 1
[variables.VALUE]
type = "integer"
required = true
min = 10
max = 20
"#;
    for value in ["10", "20"] {
        let fixture = SchemaFixture::new(&format!("VALUE={value}\n"), integer_schema);
        assert_eq!(
            run_schema(&fixture.target, &fixture.schema).status.code(),
            Some(0)
        );
    }
    for value in ["9", "21"] {
        let fixture = SchemaFixture::new(&format!("VALUE={value}\n"), integer_schema);
        assert_eq!(
            run_schema(&fixture.target, &fixture.schema).status.code(),
            Some(1)
        );
    }

    let float_schema = r#"
version = 1
[variables.VALUE]
type = "float"
required = true
min = 0.5
max = 1.5
"#;
    for value in ["0.5", "1.5"] {
        let fixture = SchemaFixture::new(&format!("VALUE={value}\n"), float_schema);
        assert_eq!(
            run_schema(&fixture.target, &fixture.schema).status.code(),
            Some(0)
        );
    }
    for value in ["0.49", "1.51"] {
        let fixture = SchemaFixture::new(&format!("VALUE={value}\n"), float_schema);
        assert_eq!(
            run_schema(&fixture.target, &fixture.schema).status.code(),
            Some(1)
        );
    }
}

#[test]
fn schema_unicode_scalar_lengths_enforce_exact_and_adjacent_boundaries() {
    let schema = r#"
version = 1
[variables.TEXT]
type = "string"
required = true
min_length = 2
max_length = 2
"#;

    let exact = SchemaFixture::new("TEXT=é🦀\n", schema);
    assert_eq!(
        run_schema(&exact.target, &exact.schema).status.code(),
        Some(0)
    );

    let below = SchemaFixture::new("TEXT=é\n", schema);
    assert_eq!(
        run_schema(&below.target, &below.schema).status.code(),
        Some(1)
    );

    let above = SchemaFixture::new("TEXT=é🦀x\n", schema);
    assert_eq!(
        run_schema(&above.target, &above.schema).status.code(),
        Some(1)
    );
}

#[test]
fn schema_typed_allowed_values_preserve_scalar_semantics() {
    let schema = r#"
version = 1
[variables.MODE]
type = "string"
required = true
allowed = ["prod"]

[variables.BIG]
type = "integer"
required = true
allowed = [9007199254740993]

[variables.RATIO]
type = "float"
required = true
allowed = [1, 2.5]

[variables.ENABLED]
type = "boolean"
required = true
allowed = [true]
"#;

    let valid = SchemaFixture::new(
        "MODE=prod\nBIG=9007199254740993\nRATIO=1.0\nENABLED=TRUE\n",
        schema,
    );
    assert_eq!(
        run_schema(&valid.target, &valid.schema).status.code(),
        Some(0)
    );

    let wrong_case = SchemaFixture::new(
        "MODE=PROD\nBIG=9007199254740993\nRATIO=1.0\nENABLED=TRUE\n",
        schema,
    );
    assert_eq!(
        run_schema(&wrong_case.target, &wrong_case.schema)
            .status
            .code(),
        Some(1)
    );
}

#[test]
fn schema_pattern_uses_regex_is_match_without_implicit_anchors() {
    let unanchored = SchemaFixture::new(
        "VALUE=concatenate\n",
        r#"
version = 1
[variables.VALUE]
type = "string"
required = true
pattern = "cat"
"#,
    );
    assert_eq!(
        run_schema(&unanchored.target, &unanchored.schema)
            .status
            .code(),
        Some(0)
    );

    let anchored = SchemaFixture::new(
        "VALUE=concatenate\n",
        r#"
version = 1
[variables.VALUE]
type = "string"
required = true
pattern = "^cat$"
"#,
    );
    assert_eq!(
        run_schema(&anchored.target, &anchored.schema).status.code(),
        Some(1)
    );
}

#[test]
fn schema_additional_target_variable_warns_without_failing_or_disclosing_value() {
    let fixture = SchemaFixture::new(
        "KNOWN=synthetic-known\nEXTRA=synthetic-extra-secret\n",
        r#"
version = 1
[variables.KNOWN]
type = "string"
required = true
"#,
    );

    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("warning"));
    assert!(stdout(&output).contains("EXTRA"));
    assert_target_values_redacted(&output, &["synthetic-known", "synthetic-extra-secret"]);
}

#[test]
fn schema_integer_outside_i64_range_is_rejected_without_value_disclosure() {
    let fixture = SchemaFixture::new(
        "VALUE=9223372036854775808\n",
        r#"
version = 1
[variables.VALUE]
type = "integer"
required = true
"#,
    );

    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("VALUE"));
    assert_target_values_redacted(&output, &["9223372036854775808"]);
}

#[test]
fn schema_non_finite_and_overflowing_float_representations_are_rejected() {
    let schema = r#"
version = 1
[variables.VALUE]
type = "float"
required = true
"#;

    for value in ["NaN", "+inf", "-inf", "1e400"] {
        let fixture = SchemaFixture::new(&format!("VALUE={value}\n"), schema);
        let output = run_schema(&fixture.target, &fixture.schema);
        assert_eq!(output.status.code(), Some(1), "value {value} must fail");
        assert!(stdout(&output).contains("VALUE"));
        assert_target_values_redacted(&output, &[value]);
    }
}

fn assert_rendered_schema_location(rendered: &str, path: &Path) {
    let path_text = path.display().to_string();
    let (_, suffix) = rendered
        .split_once(&path_text)
        .expect("stderr must contain the schema path");
    let suffix = suffix
        .strip_prefix(':')
        .expect("schema path must be followed by parser location");
    let mut parts = suffix.split(':');
    let line = parts
        .next()
        .expect("line component")
        .parse::<usize>()
        .expect("line must be numeric");
    let column = parts
        .next()
        .expect("column component")
        .parse::<usize>()
        .expect("column must be numeric");
    assert!(line > 0);
    assert!(column > 0);
}

fn assert_invalid_schema_prevents_validation(schema: &str, expect_location: bool) {
    let fixture = SchemaFixture::new("VALUE=synthetic-target-secret\n", schema);
    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(4));
    assert!(stdout(&output).is_empty());
    let rendered = stderr(&output);
    assert!(rendered.contains(&fixture.schema.display().to_string()));
    assert!(rendered.contains("schema"));
    assert!(rendered.contains("invalid schema"));
    if expect_location {
        assert_rendered_schema_location(&rendered, &fixture.schema);
    }
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}

#[test]
fn invalid_schema_malformed_toml_exits_four_with_location_and_redaction() {
    assert_invalid_schema_prevents_validation(
        "version = 1\n[variables.VALUE\ntype = \"string\"\n",
        true,
    );
}

#[test]
fn invalid_schema_unknown_top_level_property_exits_four() {
    assert_invalid_schema_prevents_validation(
        r#"
version = 1
unexpected = true
[variables.VALUE]
type = "string"
"#,
        false,
    );
}

#[test]
fn invalid_schema_unknown_variable_rule_property_exits_four() {
    assert_invalid_schema_prevents_validation(
        r#"
version = 1
[variables.VALUE]
type = "string"
unexpected = true
"#,
        false,
    );
}

#[test]
fn invalid_schema_unsupported_version_exits_four() {
    assert_invalid_schema_prevents_validation(
        r#"
version = 2
[variables.VALUE]
type = "string"
"#,
        false,
    );
}

#[test]
fn invalid_schema_incompatible_constraint_exits_four() {
    assert_invalid_schema_prevents_validation(
        r#"
version = 1
[variables.VALUE]
type = "integer"
min_length = 1
"#,
        false,
    );
}

#[test]
fn invalid_schema_contradictory_bounds_exit_four() {
    assert_invalid_schema_prevents_validation(
        r#"
version = 1
[variables.VALUE]
type = "integer"
min = 20
max = 10
"#,
        false,
    );
}

#[test]
fn invalid_schema_incompatible_allowed_entry_exits_four_without_coercion() {
    assert_invalid_schema_prevents_validation(
        r#"
version = 1
[variables.VALUE]
type = "integer"
allowed = ["synthetic-target-secret"]
"#,
        false,
    );
}

#[test]
fn invalid_schema_regex_exits_four_instead_of_becoming_target_validation() {
    assert_invalid_schema_prevents_validation(
        r#"
version = 1
[variables.VALUE]
type = "string"
pattern = "["
"#,
        false,
    );
}
