# Implementation Plan: Environment File Validation

**Branch**: `chore/init-spec-kit` | **Date**: 2026-09-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/001-env-validation/spec.md` plus the technical direction supplied to `$speckit-plan`.

## Summary

Implement `envcheck` as a single, portable Rust CLI that validates a target `.env` against either a `.env.example` key set or a version-1 TOML `.env.schema`. The first implementation remains bounded to example comparison plus the schema scalar types `string`, `integer`, `float`, and `boolean` and the constraints already specified for those types.

The executable will use `clap` derive for argument parsing, `serde` + `toml` for strict schema deserialization, `regex` for pattern constraints, and a duplicate-preserving dotenv scanner. Parsing, schema handling, domain validation, and terminal presentation remain separate. Validation returns structured diagnostics; only the output layer renders them. No networking, remote schema resolution, secret-manager integration, interpolation, or input-file modification is permitted.

## Technical Context

**Language/Version**: Stable Rust, edition 2024. The MVP does not publish a minimum supported Rust version; CI and development use the stable toolchain.

**Primary Dependencies**:
- `clap` 4.x with the `derive` feature for CLI parsing.
- `serde` 1.x with `derive` for schema model deserialization.
- `toml` 1.x for `.env.schema` parsing.
- `regex` 1.x for schema `pattern` constraints.
- `dotenvx-primitives` using its non-expanding `scan` API for dotenv tokenization, duplicate preservation, comments, quoting, and `export` handling. Envcheck MUST NOT call expansion, evaluation, decryption, or environment-injection APIs.
- Dev-only: `assert_cmd` for compiled-CLI integration tests and `tempfile` for cross-platform test fixtures.

**Storage**: N/A. Input files are read-only and processed in memory.

**Testing**: `cargo test`; module-local unit tests for parsers and validation rules; integration tests under `tests/` using `assert_cmd` to execute the built `envcheck` binary. Quality gates: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.

**Target Platform**: Linux, macOS, and Windows.

**Project Type**: Single executable CLI.

**Performance Goals**: Parsing and validation are linear in the total input size for normal use. Files are read once and validation runs in memory. No latency SLO is introduced for the MVP because correctness and determinism take precedence and the intended workload is local configuration files.

**Constraints**: Offline-only; deterministic; read-only; no ambient environment-variable lookup; no secret values in default diagnostics; no unsafe Rust; stable public CLI semantics once released.

**Scale/Scope**: One target dotenv file and one validation definition per invocation. MVP schema supports only `string`, `integer`, `float`, and `boolean`, with `required`, `allow_empty`, `min`, `max`, `min_length`, `max_length`, `allowed`, and `pattern`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

### Pre-design gate

- **I. Correctness First — PASS**: no network or ambient environment lookup; deterministic result and diagnostic ordering are explicit design goals.
- **II. CLI Stability — PASS**: `contracts/cli.md` defines the initial public argument shape, output classes, and numeric exit-code assignments.
- **III. Clear Separation of Concerns — PASS**: modules isolate CLI parsing, dotenv parsing, schema parsing, validation, and presentation.
- **IV. Test-Driven Behavior — PASS**: every rule is planned with valid/invalid/boundary unit coverage and CLI integration coverage.
- **V. Actionable Errors — PASS**: structured diagnostics carry variable/rule/expectation metadata without carrying printable secret values.
- **VI. Security and Secret Handling — PASS**: domain values never enter default formatted diagnostics; synthetic fixtures only.
- **VII. Minimal Dependencies — PASS WITH DOCUMENTED TRADEOFF**: each dependency maps to a requested capability. `dotenvx-primitives` is accepted specifically because its scan API preserves duplicates without expansion; only that API is used. No generic error-handling/logging framework is introduced.
- **VIII. Cross-Platform Behavior — PASS**: paths use `PathBuf`; tests use `tempfile`; newline handling is covered for LF/CRLF.
- **IX. Local and In-Memory Validation — PASS**: no network/services/databases; one-pass file reads and in-memory structures.
- **X. Rust Quality Gates — PASS**: format, clippy, and tests are mandatory before completion; no unsafe Rust planned.

No constitution violation requires a complexity exception.

## Design Decisions

### CLI and exit codes

Initial command contract:

```text
envcheck <ENV_FILE> [--example <FILE> | --schema <FILE>]
```

`--example` and `--schema` are mutually exclusive. When neither is provided, discovery occurs beside `ENV_FILE`: `.env.schema` first, then `.env.example`.

Stable exit codes for the first public implementation:

- `0` — validation completed without validation errors; warnings are allowed.
- `1` — validation completed and one or more validation errors were found.
- `2` — invalid CLI usage/arguments; aligned with clap argument-error behavior.
- `3` — input/discovery/parsing failure preventing validation: unreadable required file, invalid dotenv syntax, or no validation definition found.
- `4` — malformed or semantically invalid `.env.schema`, including unsupported version, unknown properties, incompatible constraints, or invalid regex.

### Dotenv parsing strategy

Use `dotenvx_primitives::scan` as the mature parsing primitive because it preserves duplicate assignments and scans without expansion or decryption. Envcheck adds a narrow compatibility/strictness adapter around the scan result to enforce its own specification where needed: invalid non-comment lines must fail, names must match `[A-Za-z_][A-Za-z0-9_]*`, case is significant, and inline `#` semantics must match the clarified contract. The adapter must not become a second general-purpose dotenv implementation.

Characterization tests must lock down quoted values, `export`, CRLF, empty values, literal `${NAME}`, duplicate preservation, inline comments, malformed lines, and names before the parser is used by validation.

### Schema parsing strategy

Deserialize version-1 TOML into strict Serde structures with `#[serde(deny_unknown_fields)]` at both the top level and variable-rule level. Use a deterministic ordered map (`BTreeMap`) for variable rules. Syntax parsing and semantic schema validation are separate stages.

Semantic validation checks version support, variable-name syntax, type/constraint compatibility, valid constraint value types, inclusive bound consistency, finite numeric bounds, `min <= max`, `min_length <= max_length`, typed `allowed` entries, and regex compilation. Invalid schema definitions stop target validation and exit with code `4`.

### Validation and diagnostics

The validation layer operates only on parsed domain structures. It does not know about clap, stdout/stderr, ANSI formatting, or process exit. It produces a `ValidationResult` containing structured `Diagnostic` values with severity, stable rule code, optional variable name, and a safe expectation/message. Target values are never copied into diagnostics.

Canonical ordering is deterministic: errors before warnings, then variable name, then stable rule code, with file-level diagnostics ordered before variable-level diagnostics when a validation run can continue. Hash-map iteration order must never determine user-visible output.

### Output channels

- Completed validation reports, including validation errors and warnings, are rendered to stdout.
- CLI usage errors and failures that prevent validation from running (file/discovery/dotenv parsing/schema-definition errors) are rendered to stderr.
- Secret values are never rendered by default.

## Project Structure

### Documentation (this feature)

```text
specs/001-env-validation/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cli.md
│   └── env-schema.md
├── checklists/
│   ├── requirements.md
│   └── validation-contract.md
└── tasks.md                 # created later by $speckit-tasks
```

### Source Code (repository root)

```text
Cargo.toml
Cargo.lock
src/
├── main.rs
├── cli.rs
├── env/
│   ├── mod.rs
│   └── parser.rs
├── schema/
│   ├── mod.rs
│   ├── model.rs
│   └── parser.rs
├── validation/
│   ├── mod.rs
│   ├── validator.rs
│   ├── error.rs
│   └── rules/
│       ├── mod.rs
│       ├── presence.rs
│       ├── value_type.rs
│       ├── numeric.rs
│       ├── length.rs
│       ├── allowed.rs
│       └── pattern.rs
└── output/
    └── mod.rs

tests/
└── cli.rs
```

**Structure Decision**: Use one Cargo binary crate. `main.rs` is an orchestration shell: parse CLI, resolve definition, parse inputs, invoke domain validation, render structured results, and translate the final outcome to the stable exit code. Validation modules do not depend on clap or output formatting. Unit tests remain adjacent to modules; compiled-binary behavior lives in `tests/cli.rs`.

## Phase 0: Research Outcome

Research is consolidated in [research.md](./research.md). It resolves dependency choices, dotenv behavior, strict Serde/TOML parsing, regex semantics, exit codes, deterministic ordering, security handling, and test strategy. No `NEEDS CLARIFICATION` remains.

## Phase 1: Design Artifacts

- [data-model.md](./data-model.md) defines parsed dotenv entries, schema definitions/rules, diagnostics, and validation results.
- [contracts/cli.md](./contracts/cli.md) defines arguments, validator selection, output classes, stable exit codes, and deterministic reporting.
- [contracts/env-schema.md](./contracts/env-schema.md) defines the version-1 TOML schema contract and semantic validation rules.
- [quickstart.md](./quickstart.md) defines end-to-end validation scenarios to run after implementation.

## Post-design Constitution Check

- Correctness/determinism remain preserved by ordered structures and canonical diagnostic sorting.
- The CLI contract is now explicit and versionable, including numeric exit codes.
- Parsing, schema, validation, and output boundaries remain independent in the data model and source layout.
- Unit/integration test responsibilities are explicit in the design and quickstart.
- Diagnostics are structurally incapable of requiring raw values; output examples contain synthetic values only.
- Dependencies remain bounded to requested capabilities plus test-only tooling; the dotenv dependency tradeoff is recorded in research.
- Cross-platform paths/newlines and Windows/macOS/Linux execution are part of acceptance validation.
- No network, persistence, file mutation, unsafe Rust, or ambient environment lookup has been introduced.

**Post-design gate result: PASS.**

## Complexity Tracking

No constitution violations or exceptional complexity require justification.
