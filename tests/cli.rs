// All environment values in this integration suite are synthetic test data.
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

fn run_target_without_definition_flag(target: &Path) -> Output {
    let mut command = Command::cargo_bin("envcheck").expect("compiled envcheck binary");
    command.arg(target);
    command.output().expect("execute envcheck")
}

#[test]
fn us4_discovery_prefers_sibling_schema_over_sibling_example() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join(".env");
    let schema = directory.path().join(".env.schema");
    let example = directory.path().join(".env.example");
    fs::write(&target, "VALUE=synthetic-not-an-integer\n").expect("write target");
    fs::write(
        &schema,
        "version = 1\n[variables.VALUE]\ntype = \"integer\"\nrequired = true\n",
    )
    .expect("write schema");
    fs::write(&example, "VALUE=ignored\n").expect("write example");

    let output = run_target_without_definition_flag(&target);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("VALUE"));
    assert!(stdout(&output).contains("declared type"));
    assert!(stderr(&output).is_empty());
    assert_target_values_redacted(&output, &["synthetic-not-an-integer"]);
}

#[test]
fn us4_discovery_falls_back_to_sibling_example_when_schema_is_absent() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join(".env");
    let example = directory.path().join(".env.example");
    fs::write(&target, "APP_NAME=synthetic-app-secret\n").expect("write target");
    fs::write(&example, "APP_NAME=ignored\nREDIS_URL=ignored\n").expect("write example");

    let output = run_target_without_definition_flag(&target);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("REDIS_URL"));
    assert!(stderr(&output).is_empty());
    assert_target_values_redacted(&output, &["synthetic-app-secret"]);
}

#[test]
fn us4_explicit_example_overrides_sibling_schema_discovery() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join(".env");
    let sibling_schema = directory.path().join(".env.schema");
    let explicit_example = directory.path().join("explicit.example");
    fs::write(&target, "VALUE=synthetic-text-secret\n").expect("write target");
    fs::write(
        &sibling_schema,
        "version = 1\n[variables.VALUE]\ntype = \"integer\"\nrequired = true\n",
    )
    .expect("write sibling schema");
    fs::write(&explicit_example, "VALUE=ignored\n").expect("write explicit example");

    let output = run_example(&target, &explicit_example);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("ok: environment validation succeeded"));
    assert!(stderr(&output).is_empty());
    assert_target_values_redacted(&output, &["synthetic-text-secret"]);
}

#[test]
fn us4_explicit_schema_overrides_sibling_example_discovery() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join(".env");
    let sibling_example = directory.path().join(".env.example");
    let explicit_schema = directory.path().join("explicit.schema");
    fs::write(&target, "VALUE=42\n").expect("write target");
    fs::write(&sibling_example, "VALUE=ignored\nMISSING=ignored\n").expect("write sibling example");
    fs::write(
        &explicit_schema,
        "version = 1\n[variables.VALUE]\ntype = \"integer\"\nrequired = true\n",
    )
    .expect("write explicit schema");

    let output = run_schema(&target, &explicit_schema);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("ok: environment validation succeeded"));
    assert!(stderr(&output).is_empty());
    assert_target_values_redacted(&output, &["42"]);
}

#[test]
fn us4_no_discovered_definition_is_exit_three_with_safe_context() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join(".env");
    fs::write(&target, "API_KEY=synthetic-target-secret\n").expect("write target");

    let output = run_target_without_definition_flag(&target);

    assert_eq!(output.status.code(), Some(3));
    assert!(stdout(&output).is_empty());
    let rendered = stderr(&output);
    assert!(rendered.contains("discovery"));
    assert!(rendered.contains(&target.display().to_string()));
    assert!(rendered.contains("no validation definition is available"));
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}

#[test]
fn us4_unreadable_target_is_exit_three_with_path_and_safe_reason() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join("missing.env");
    let example = directory.path().join(".env.example");
    fs::write(&example, "API_KEY=ignored\n").expect("write example");

    let output = run_example(&target, &example);

    assert_eq!(output.status.code(), Some(3));
    assert!(stdout(&output).is_empty());
    let rendered = stderr(&output);
    assert!(rendered.contains(&target.display().to_string()));
    assert!(rendered.contains("required file could not be read"));
}

#[test]
fn us4_unreadable_explicit_example_is_exit_three_with_path_and_safe_reason() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join(".env");
    let example = directory.path().join("missing.example");
    fs::write(&target, "API_KEY=synthetic-target-secret\n").expect("write target");

    let output = run_example(&target, &example);

    assert_eq!(output.status.code(), Some(3));
    assert!(stdout(&output).is_empty());
    let rendered = stderr(&output);
    assert!(rendered.contains(&example.display().to_string()));
    assert!(rendered.contains("required file could not be read"));
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}

#[test]
fn us4_unreadable_explicit_schema_is_exit_three_with_path_and_safe_reason() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join(".env");
    let schema = directory.path().join("missing.schema");
    fs::write(&target, "API_KEY=synthetic-target-secret\n").expect("write target");

    let output = run_schema(&target, &schema);

    assert_eq!(output.status.code(), Some(3));
    assert!(stdout(&output).is_empty());
    let rendered = stderr(&output);
    assert!(rendered.contains(&schema.display().to_string()));
    assert!(rendered.contains("required file could not be read"));
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}

#[test]
fn us4_mutually_exclusive_definition_flags_are_cli_usage_exit_two() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join(".env");
    let example = directory.path().join(".env.example");
    let schema = directory.path().join(".env.schema");
    fs::write(&target, "VALUE=synthetic-target-secret\n").expect("write target");
    fs::write(&example, "VALUE=ignored\n").expect("write example");
    fs::write(
        &schema,
        "version = 1\n[variables.VALUE]\ntype = \"string\"\n",
    )
    .expect("write schema");

    let mut command = Command::cargo_bin("envcheck").expect("compiled envcheck binary");
    let output = command
        .arg(&target)
        .arg("--example")
        .arg(&example)
        .arg("--schema")
        .arg(&schema)
        .output()
        .expect("execute envcheck");

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).is_empty());
    assert!(!stderr(&output).is_empty());
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}

#[test]
fn us4_completed_validation_uses_stdout_and_maps_warning_and_error_exits() {
    let warning_fixture = ExampleFixture::new(
        "KNOWN=synthetic-known-secret\nEXTRA=synthetic-extra-secret\n",
        "KNOWN=ignored\n",
    );
    let warning_output = run_example(&warning_fixture.target, &warning_fixture.example);
    assert_eq!(warning_output.status.code(), Some(0));
    assert!(stdout(&warning_output).contains("warning"));
    assert!(stdout(&warning_output).contains("ok: environment validation succeeded"));
    assert!(stderr(&warning_output).is_empty());
    assert_target_values_redacted(
        &warning_output,
        &["synthetic-known-secret", "synthetic-extra-secret"],
    );

    let error_fixture = ExampleFixture::new(
        "KNOWN=synthetic-known-secret\n",
        "KNOWN=ignored\nREQUIRED=ignored\n",
    );
    let error_output = run_example(&error_fixture.target, &error_fixture.example);
    assert_eq!(error_output.status.code(), Some(1));
    assert!(stdout(&error_output).contains("error"));
    assert!(stdout(&error_output).contains("REQUIRED"));
    assert!(stderr(&error_output).is_empty());
    assert_target_values_redacted(&error_output, &["synthetic-known-secret"]);
}

#[test]
fn us4_dotenv_prevention_uses_stderr_safe_context_and_known_line() {
    let fixture = ExampleFixture::new(
        "API_KEY=synthetic-target-secret\nnot an assignment\n",
        "API_KEY=ignored\n",
    );

    let output = run_example(&fixture.target, &fixture.example);

    assert_eq!(output.status.code(), Some(3));
    assert!(stdout(&output).is_empty());
    let rendered = stderr(&output);
    assert!(rendered.contains("dotenv"));
    assert!(rendered.contains(&fixture.target.display().to_string()));
    assert!(rendered.contains("invalid dotenv syntax"));
    assert!(rendered.contains(":2"));
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}

#[test]
fn us4_unreadable_definition_prevention_uses_stderr_safe_context() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join(".env");
    let missing = directory.path().join("missing.example");
    fs::write(&target, "API_KEY=synthetic-target-secret\n").expect("write target");

    let output = run_example(&target, &missing);

    assert_eq!(output.status.code(), Some(3));
    assert!(stdout(&output).is_empty());
    let rendered = stderr(&output);
    assert!(rendered.contains("input"));
    assert!(rendered.contains(&missing.display().to_string()));
    assert!(rendered.contains("required file could not be read"));
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}

#[test]
fn us4_toml_syntax_prevention_is_exit_four_with_location_and_redaction() {
    let fixture = SchemaFixture::new(
        "VALUE=synthetic-target-secret\n",
        "version = 1\n[variables.VALUE\ntype = \"string\"\n",
    );

    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(4));
    assert!(stdout(&output).is_empty());
    let rendered = stderr(&output);
    assert!(rendered.contains("schema"));
    assert!(rendered.contains(&fixture.schema.display().to_string()));
    assert!(rendered.contains("invalid schema syntax"));
    assert_rendered_schema_location(&rendered, &fixture.schema);
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}

#[test]
fn us4_schema_definition_prevention_is_exit_four_with_safe_context_and_redaction() {
    let fixture = SchemaFixture::new(
        "VALUE=synthetic-target-secret\n",
        "version = 2\n[variables.VALUE]\ntype = \"string\"\n",
    );

    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(4));
    assert!(stdout(&output).is_empty());
    let rendered = stderr(&output);
    assert!(rendered.contains("schema"));
    assert!(rendered.contains(&fixture.schema.display().to_string()));
    assert!(rendered.contains("invalid schema definition"));
    assert_target_values_redacted(&output, &["synthetic-target-secret"]);
}

#[test]
fn us4_representative_validation_is_identical_across_exactly_100_runs() {
    let fixture = SchemaFixture::new(
        "B=not-a-boolean\nA=5\nZ_EXTRA=synthetic-z-secret\nY_EXTRA=synthetic-y-secret\n",
        r#"
version = 1
[variables.A]
type = "integer"
required = true
min = 10

[variables.B]
type = "boolean"
required = true
"#,
    );

    let mut baseline: Option<(Option<i32>, Vec<u8>, Vec<u8>)> = None;
    for iteration in 0..100 {
        let output = run_schema(&fixture.target, &fixture.schema);
        let snapshot = (output.status.code(), output.stdout, output.stderr);
        if let Some(expected) = &baseline {
            assert_eq!(snapshot, *expected, "run {iteration} differed from run 0");
        } else {
            baseline = Some(snapshot);
        }
    }

    let (status, stdout_bytes, stderr_bytes) = baseline.expect("100 runs produce baseline");
    assert_eq!(status, Some(1));
    assert!(stderr_bytes.is_empty());
    let rendered = String::from_utf8(stdout_bytes).expect("stdout must be UTF-8");
    let a_error = rendered.find("error: A:").expect("A error diagnostic");
    let b_error = rendered.find("error: B:").expect("B error diagnostic");
    let y_warning = rendered
        .find("warning: Y_EXTRA:")
        .expect("Y_EXTRA warning diagnostic");
    let z_warning = rendered
        .find("warning: Z_EXTRA:")
        .expect("Z_EXTRA warning diagnostic");
    assert!(a_error < b_error);
    assert!(b_error < y_warning);
    assert!(y_warning < z_warning);
    assert!(rendered.contains("configured minimum"));
    assert!(rendered.contains("declared type"));
    assert!(rendered.contains("variable is not declared in schema"));
    for value in ["not-a-boolean", "synthetic-z-secret", "synthetic-y-secret"] {
        assert!(
            !rendered.contains(value),
            "stdout exposed target value: {value}"
        );
    }
}

#[test]
fn cross_platform_lf_and_crlf_nested_temp_paths_validate_equivalently() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let lf_dir = directory.path().join("portable").join("lf");
    let crlf_dir = directory.path().join("portable").join("crlf");
    fs::create_dir_all(&lf_dir).expect("create LF fixture directory");
    fs::create_dir_all(&crlf_dir).expect("create CRLF fixture directory");

    let lf_target = lf_dir.join(".env");
    let lf_example = lf_dir.join(".env.example");
    let crlf_target = crlf_dir.join(".env");
    let crlf_example = crlf_dir.join(".env.example");

    fs::write(
        &lf_target,
        "APP_NAME=synthetic-portable-app\nPORT=synthetic-portable-port\n",
    )
    .expect("write LF target");
    fs::write(&lf_example, "APP_NAME=ignored\nPORT=ignored\n").expect("write LF example");

    fs::write(
        &crlf_target,
        "APP_NAME=synthetic-portable-app\r\nPORT=synthetic-portable-port\r\n",
    )
    .expect("write CRLF target");
    fs::write(&crlf_example, "APP_NAME=ignored\r\nPORT=ignored\r\n").expect("write CRLF example");

    let lf_output = run_target_without_definition_flag(&lf_target);
    let crlf_output = run_target_without_definition_flag(&crlf_target);

    assert_eq!(lf_output.status.code(), Some(0));
    assert_eq!(crlf_output.status.code(), Some(0));
    assert_eq!(lf_output.stdout, crlf_output.stdout);
    assert_eq!(lf_output.stderr, crlf_output.stderr);
    assert_target_values_redacted(
        &lf_output,
        &["synthetic-portable-app", "synthetic-portable-port"],
    );
    assert_target_values_redacted(
        &crlf_output,
        &["synthetic-portable-app", "synthetic-portable-port"],
    );
}

#[test]
fn cross_platform_cli_variable_identity_remains_case_sensitive() {
    let fixture = ExampleFixture::new("port=synthetic-lower-port\n", "PORT=ignored\n");

    let output = run_example(&fixture.target, &fixture.example);

    assert_eq!(output.status.code(), Some(1));
    let rendered = stdout(&output);
    assert!(rendered.contains("error: PORT:"));
    assert!(rendered.contains("warning: port:"));
    assert!(stderr(&output).is_empty());
    assert_target_values_redacted(&output, &["synthetic-lower-port"]);
}

#[test]
fn security_regression_redacts_target_values_on_completed_and_preventing_paths() {
    let completed = SchemaFixture::new(
        "API_KEY=synthetic-completed-secret\n",
        "version = 1\n[variables.API_KEY]\ntype = \"integer\"\nrequired = true\n",
    );
    let completed_output = run_schema(&completed.target, &completed.schema);
    assert_eq!(completed_output.status.code(), Some(1));
    assert!(!stdout(&completed_output).is_empty());
    assert!(stderr(&completed_output).is_empty());
    assert_target_values_redacted(&completed_output, &["synthetic-completed-secret"]);

    let preventing = ExampleFixture::new(
        "API_KEY=synthetic-preventing-secret\nnot an assignment\n",
        "API_KEY=ignored\n",
    );
    let preventing_output = run_example(&preventing.target, &preventing.example);
    assert_eq!(preventing_output.status.code(), Some(3));
    assert!(stdout(&preventing_output).is_empty());
    assert!(!stderr(&preventing_output).is_empty());
    assert_target_values_redacted(&preventing_output, &["synthetic-preventing-secret"]);
}

#[test]
fn representative_success_and_failure_leave_all_input_files_byte_for_byte_unchanged() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let target = directory.path().join(".env");
    let example = directory.path().join(".env.example");
    let schema = directory.path().join(".env.schema");

    let target_bytes = b"VALUE=synthetic-read-only-secret\r\n".to_vec();
    let example_bytes = b"VALUE=ignored\r\n".to_vec();
    let schema_bytes =
        b"version = 1\n[variables.VALUE]\ntype = \"integer\"\nrequired = true\n".to_vec();

    fs::write(&target, &target_bytes).expect("write target");
    fs::write(&example, &example_bytes).expect("write example");
    fs::write(&schema, &schema_bytes).expect("write schema");

    let success = run_example(&target, &example);
    assert_eq!(success.status.code(), Some(0));
    assert_target_values_redacted(&success, &["synthetic-read-only-secret"]);

    let failure = run_schema(&target, &schema);
    assert_eq!(failure.status.code(), Some(1));
    assert_target_values_redacted(&failure, &["synthetic-read-only-secret"]);

    assert_eq!(
        fs::read(&target).expect("read target after runs"),
        target_bytes
    );
    assert_eq!(
        fs::read(&example).expect("read example after runs"),
        example_bytes
    );
    assert_eq!(
        fs::read(&schema).expect("read schema after runs"),
        schema_bytes
    );
}

#[test]
fn convergence_float_allowed_integer_keeps_exact_numeric_meaning_beyond_two_to_the_53() {
    let fixture = SchemaFixture::new(
        "VALUE=9007199254740992\n",
        r#"
version = 1
[variables.VALUE]
type = "float"
required = true
allowed = [9007199254740993]
"#,
    );

    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("configured allowed set"));
    assert!(stderr(&output).is_empty());
    assert_target_values_redacted(&output, &["9007199254740992"]);
}

#[test]
fn convergence_float_integer_min_keeps_exact_numeric_meaning_beyond_two_to_the_53() {
    let fixture = SchemaFixture::new(
        "VALUE=9007199254740992\n",
        r#"
version = 1
[variables.VALUE]
type = "float"
required = true
min = 9007199254740993
"#,
    );

    let output = run_schema(&fixture.target, &fixture.schema);

    assert_eq!(output.status.code(), Some(1));
    let rendered = stdout(&output);
    assert!(rendered.contains("configured minimum"));
    assert!(rendered.contains("float >= 9007199254740993"));
    assert!(stderr(&output).is_empty());
    assert_target_values_redacted(&output, &["9007199254740992"]);
}

