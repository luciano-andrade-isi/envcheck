---

description: "Dependency-ordered implementation tasks for Envcheck environment validation"
---

# Tasks: Environment File Validation

**Input**: Design documents from `specs/001-env-validation/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: Tests are required by the feature plan and project constitution. Within each user-story phase, test tasks must be completed before the implementation tasks they specify.

**Organization**: Tasks are grouped by user story so each story can be implemented and validated as an independently useful increment.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it targets different files and does not depend on incomplete work in the same phase
- **[Story]**: User story traceability label (`US1`–`US4`)
- Every task names the file or files it changes

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize the Rust binary project and requested source layout without implementing feature behavior yet.

- [X] T001 Initialize the `envcheck` Cargo binary package with Rust edition 2024 and declare runtime/dev dependencies (`clap` with `derive`, `serde` with `derive`, `toml`, `regex`, `dotenvx-primitives`, `assert_cmd`, `tempfile`) in `Cargo.toml`
- [X] T002 Create the planned module/file skeleton in `src/main.rs`, `src/cli.rs`, `src/env/mod.rs`, `src/env/parser.rs`, `src/schema/mod.rs`, `src/schema/model.rs`, `src/schema/parser.rs`, `src/validation/mod.rs`, `src/validation/validator.rs`, `src/validation/error.rs`, `src/validation/rules/mod.rs`, `src/validation/rules/presence.rs`, `src/validation/rules/value_type.rs`, `src/validation/rules/numeric.rs`, `src/validation/rules/length.rs`, `src/validation/rules/allowed.rs`, `src/validation/rules/pattern.rs`, `src/output/mod.rs`, and `tests/cli.rs`

**Checkpoint**: Cargo project and source layout exist; no user-story validation behavior is required yet.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared parsing, diagnostic, CLI-shape, and result infrastructure required by every user story.

**⚠️ CRITICAL**: Complete this phase before implementing any user story.

### Tests for shared foundations

- [X] T003 [P] Add dotenv parser characterization tests covering blank/comment lines, single/double quotes, `export`, CRLF, empty values, literal `${NAME}`, duplicate preservation, invalid non-comment lines, `[A-Za-z_][A-Za-z0-9_]*` names, case sensitivity, inline `#` semantics, and preservation of the known source line for Envcheck-detected malformed dotenv input without exposing parsed values in `src/env/parser.rs`
- [X] T004 [P] Add unit tests for canonical diagnostic ordering (`Error` before `Warning`, then variable name, rule code, line) and warning-only `has_errors() == false` semantics in `src/validation/mod.rs`
- [X] T005 [P] Add CLI-argument unit tests for required `<ENV_FILE>` and mutual exclusion of `--example`/`--schema` in `src/cli.rs`

### Shared implementation

- [X] T006 Implement `EnvEntry { key, value, line }` and `EnvDocument { path, entries, key_index }` in `src/env/mod.rs`, preserving duplicate multiplicity and keeping raw values out of user-visible diagnostic types
- [X] T007 Implement the narrow dotenv parsing adapter over the non-expanding `dotenvx_primitives::scan` API in `src/env/parser.rs`, enforcing invalid-line failure, `[A-Za-z_][A-Za-z0-9_]*`, case sensitivity, no interpolation/environment lookup, quoted values, `export`, CRLF, and clarified inline-comment behavior; preserve source path plus parser-provided location when available, and retain the known line number when Envcheck itself rejects a malformed dotenv line, without placing raw target values in the error representation
- [X] T008 [P] Define `Severity`, stable `RuleCode` values (`duplicate_key`, `missing_required`, `empty_not_allowed`, `type_mismatch`, `below_min`, `above_max`, `too_short`, `too_long`, `not_allowed`, `pattern_mismatch`, `additional_variable`), and value-redacted `Diagnostic` fields in `src/validation/error.rs`
- [X] T009 Implement `ValidationResult { diagnostics }`, `has_errors()`, and deterministic diagnostic sorting in `src/validation/mod.rs`
- [X] T010 [P] Implement the clap derive argument model for `envcheck <ENV_FILE> [--example <FILE> | --schema <FILE>]` using `PathBuf` and mutually exclusive validator flags in `src/cli.rs`
- [X] T011 [P] Implement centralized rendering primitives for `error`, `warning`, final `ok` summaries, and structured preventing failures, rendering preventing failures to stderr from safe category/path/reason/optional-line/optional-column fields without accepting raw target values in `src/output/mod.rs`
- [X] T012 Create the orchestration shell and stable exit-code constants (`0` success/warnings, `1` validation errors, `2` CLI usage, `3` input/discovery/dotenv failure, `4` schema failure) in `src/main.rs`, including a structured application-level preventing-error representation with failure category, affected `PathBuf`, safe reason, optional line, and optional column, and no field capable of carrying a raw target value; do not add story-specific validator selection logic yet

**Checkpoint**: Shared parser, domain diagnostic structures, CLI argument shape, output primitives, and exit-code constants are ready.

---

## Phase 3: User Story 1 - Validate required variables from an example file (Priority: P1) 🎯 MVP Part 1

**Goal**: Validate a target `.env` against an explicit `.env.example` by key presence, with duplicate errors and target-only warnings.

**Independent Test**: Run `envcheck <ENV_FILE> --example <FILE>` against fixtures where required keys are present/missing, keys are reordered, example values differ, duplicates occur in either file, and target-only keys exist; missing/duplicate keys must fail while additional keys warn only.

### Tests for User Story 1

- [X] T013 [P] [US1] Add unit tests for example comparison covering missing keys, reordered keys, ignored example values, duplicate keys in target/example, empty example file, case-sensitive key matching, and target-only warnings in `src/validation/rules/presence.rs`
- [X] T014 [US1] Add compiled-CLI integration tests for explicit `--example` success, missing `REDIS_URL` exit `1`, duplicate declaration exit `1`, warning-only exit `0`, malformed target dotenv exit `3`, and malformed example dotenv exit `3`; for preventing parse failures assert stderr includes the affected path, a safe reason, and the known line when available, and assert actual target values appear in neither stdout nor stderr in `tests/cli.rs`

### Implementation for User Story 1

- [X] T015 [US1] Implement example-presence rules in `src/validation/rules/presence.rs`: every example key must exist, example values are ignored, key order is irrelevant, duplicates in either document are errors, and target-only keys are `additional_variable` warnings
- [X] T016 [US1] Implement example-file validation orchestration and structured diagnostics in `src/validation/validator.rs` using only `EnvDocument` inputs and returning `ValidationResult`
- [X] T017 [US1] Wire explicit `--example` file reading/parsing, example validation, stdout rendering for completed validation, and structured preventing failures for unreadable/malformed target or example files to stderr with exit `3`, preserving affected path, safe reason, and available source location without target-value disclosure in `src/main.rs`

**Checkpoint**: Explicit `.env.example` comparison is fully functional and independently testable.

---

## Phase 4: User Story 2 - Validate typed environment values from a schema (Priority: P1) 🎯 MVP Part 2

**Goal**: Validate target values against a valid version-1 TOML schema using `string`, `integer`, `float`, and `boolean` plus the specified MVP constraints.

**Independent Test**: Run `envcheck <ENV_FILE> --schema <FILE>` with a valid version-1 schema containing all four scalar types and each applicable constraint; valid boundary values must succeed and invalid values must produce value-redacted variable-level errors.

### Tests for User Story 2

- [X] T018 [P] [US2] Add schema-model deserialization tests for `SchemaDefinition`, `VariableRule`, `VariableType`, and `SchemaScalar`, including defaults `required = false` and `allow_empty = false`, in `src/schema/model.rs`
- [X] T019 [P] [US2] Add presence/empty-state unit tests proving: absent + `required = true` is an error; absent + `required = false` is valid; existing empty + `allow_empty = false` is an error; existing empty + `allow_empty = true` is accepted and skips all type/value constraints in `src/validation/rules/presence.rs`
- [X] T020 [P] [US2] Add scalar type tests in `src/validation/rules/value_type.rs` covering Rust `i64` integer semantics (`i64::MIN` and `i64::MAX` accepted, below-minimum and above-maximum representations rejected, decimal and scientific notation rejected for integer variables) and Rust `f64` float semantics (finite decimal and finite scientific notation accepted; `NaN`, positive infinity, negative infinity, and overflow/non-finite parse results rejected), plus string parsing and case-insensitive `true`/`false` with rejection of `1`, `0`, `yes`, `no`, `on`, `off`
- [X] T021 [P] [US2] Add inclusive numeric boundary tests for integer/float `min` and `max`, including exact-boundary, just-below, and just-above values, preserving integer comparisons as `i64` and float comparisons as finite `f64` without routing integer comparison through `f64`, in `src/validation/rules/numeric.rs`
- [X] T022 [P] [US2] Add string length tests using Unicode scalar-value counting, inclusive `min_length`/`max_length`, and exact-boundary/one-character-outside cases in `src/validation/rules/length.rs`
- [X] T023 [P] [US2] Add typed `allowed` tests in `src/validation/rules/allowed.rs` covering case-sensitive TOML strings, TOML integer/`i64` equality without `f64` coercion, finite-`f64` numeric equality including compatible finite TOML integer values for float rules, and TOML boolean meaning; verify incompatible scalar kinds are not silently coerced
- [X] T024 [P] [US2] Add `pattern` tests using Rust `regex::Regex::is_match` semantics with no implicit anchors in `src/validation/rules/pattern.rs`
- [X] T025 [US2] Add compiled-CLI integration tests for explicit `--schema` validation covering all four types, required/optional behavior, `allow_empty`, inclusive numeric bounds, string lengths, typed `allowed`, patterns, additional-variable warnings, quoted values, `export`, and literal `${NAME}`; include representative rejection of an integer outside the `i64` range and a float representation that overflows to a non-finite `f64` in `tests/cli.rs`

### Implementation for User Story 2

- [X] T026 [P] [US2] Implement `SchemaDefinition { version: u32, variables: BTreeMap<String, VariableRule> }`, `VariableType::{String,Integer,Float,Boolean}`, `VariableRule` fields, and `SchemaScalar::{String,Integer,Float,Boolean}` with defaults in `src/schema/model.rs`; preserve TOML scalar kinds so constraint values are not silently coerced
- [X] T027 [US2] Implement valid-schema TOML deserialization and variable-name checking in `src/schema/parser.rs`, producing a usable version-1 schema for compatible inputs without consulting external state; preserve safe TOML parser location metadata in structured parse failures when available
- [X] T028 [P] [US2] Extend presence logic for schema `required`/`allow_empty` semantics in `src/validation/rules/presence.rs`, with `allow_empty = true` short-circuiting type/value-specific rules for an existing empty value
- [X] T029 [P] [US2] Implement declared scalar conversion and type-mismatch diagnostics in `src/validation/rules/value_type.rs`: strings use parsed dotenv text; integers parse directly to Rust `i64`, must fit `i64::MIN..=i64::MAX`, reject decimal/scientific syntax and overflow/underflow, and MUST NOT use `f64` coercion; floats parse to Rust `f64`, accept finite decimal/scientific forms only, and reject `NaN`, infinities, and overflow/non-finite results; booleans accept only case-insensitive `true`/`false`
- [X] T030 [P] [US2] Implement inclusive integer/float `min` and `max` validation in `src/validation/rules/numeric.rs`, preserving integer bounds/comparison as `i64` and float bounds/comparison as finite `f64` without coercing integer validation through `f64`
- [X] T031 [P] [US2] Implement string `min_length`/`max_length` validation using Unicode scalar values (`chars().count()`) in `src/validation/rules/length.rs`
- [X] T032 [P] [US2] Implement typed finite-set `allowed` comparison for string/integer/float/boolean values in `src/validation/rules/allowed.rs`: integer values and integer-rule entries remain `i64` with no `f64` coercion; float values use finite `f64` semantics and may compare against compatible finite TOML integer or float entries; string equality remains case-sensitive and incompatible scalar kinds are never implicitly coerced
- [X] T033 [P] [US2] Implement string `pattern` evaluation with `regex::Regex::is_match` and no implicit anchors in `src/validation/rules/pattern.rs`
- [X] T034 [US2] Implement schema-based target validation orchestration in `src/validation/validator.rs`: duplicate target keys, presence/empty handling, scalar conversion, applicable constraints, target-only warnings, structured safe diagnostics, and deterministic sorting
- [X] T035 [US2] Wire explicit `--schema` reading/parsing/validation into `src/main.rs`, using stdout for completed validation and exit `0`/`1` while reserving schema-definition failures for exit `4`

**Checkpoint**: Both P1 capabilities—explicit `.env.example` comparison and valid-schema typed validation—work independently. This is not a release boundary: US3 MUST complete before any public MVP or release so invalid schemas cannot be silently accepted.

---

## Phase 5: User Story 3 - Detect invalid validation definitions safely (Priority: P2)

**Goal**: Reject malformed, unsupported, or semantically incompatible `.env.schema` definitions before validating target values.

**Independent Test**: Supply malformed TOML, unknown fields, unsupported versions, incompatible constraint/type pairs, contradictory bounds, incompatible `allowed` entries, invalid variable names, invalid regex, and non-finite float constraints; every case must fail as a schema-definition error with exit `4` and never be silently ignored.

### Tests for User Story 3

- [X] T036 [P] [US3] Add schema syntax/strictness tests for malformed TOML, unknown top-level fields, unknown variable-rule fields, missing required schema structure, unsupported types, and unsupported version in `src/schema/parser.rs`; malformed TOML failures must retain the affected schema path, a safe reason, and parser-provided line/column when available, without carrying target values
- [X] T037 [P] [US3] Add semantic schema tests for incompatible constraints, wrong constraint scalar kinds, `min > max`, `min_length > max_length`, incompatible typed `allowed` entries (including explicit rejection of coercion between string/integer/float/boolean kinds), invalid variable identifiers, and non-finite float constraints in `src/schema/parser.rs`
- [X] T038 [P] [US3] Add invalid-regex schema tests proving regex compilation failure is a schema error rather than a target validation error in `src/validation/rules/pattern.rs`
- [X] T039 [US3] Add compiled-CLI integration tests for malformed TOML and invalid schema definitions including unknown properties, unsupported version, incompatible constraints, contradictory bounds, incompatible `allowed`, and invalid regex; assert exit `4`, stderr contains the affected schema path and a safe reason plus line/column when supplied by the parser, and actual target values appear in neither stdout nor stderr in `tests/cli.rs`

### Implementation for User Story 3

- [X] T040 [US3] Apply strict Serde deserialization (`deny_unknown_fields`) to top-level schema and variable-rule structures in `src/schema/model.rs`
- [X] T041 [US3] Implement complete schema semantic validation in `src/schema/parser.rs`: version must equal `1`; names match `[A-Za-z_][A-Za-z0-9_]*`; `min`/`max` only integer/float; `min_length`/`max_length` only string and non-negative whole numbers; `pattern` only string; every `allowed` item is type-compatible with no implicit scalar coercion; contradictory bounds invalidate the whole schema; schema-definition failures must retain affected path, safe reason, and any available parser location without target-value data
- [X] T042 [US3] Compile schema regex constraints during semantic schema validation and return a schema-definition failure for invalid patterns in `src/schema/parser.rs` while keeping runtime matching in `src/validation/rules/pattern.rs`
- [X] T043 [US3] Map malformed or semantically invalid schema failures to the structured preventing-error representation, preserving schema path, safe reason, and available line/column, render them to stderr with stable exit code `4`, and ensure target validation does not proceed or expose target values in `src/main.rs`

**Checkpoint**: Invalid validation definitions fail explicitly and cannot silently weaken validation.

---

## Phase 6: User Story 4 - Select a validation definition and consume safe results (Priority: P2)

**Goal**: Provide deterministic validator discovery, stable public exit semantics, safe output channels, and reproducible diagnostics for local and CI use.

**Independent Test**: Exercise explicit paths and automatic discovery with both sibling definitions, only `.env.example`, no definition, unreadable files, invalid CLI usage, warning-only validation, validation errors, malformed schemas, and a secret-bearing invalid input; selection, stream, exit code, and diagnostic ordering must match the CLI contract.

### Tests for User Story 4

- [X] T044 [US4] Add compiled-CLI tests for automatic discovery beside `<ENV_FILE>` (`.env.schema` before `.env.example`), explicit-path override, no-definition exit `3`, unreadable target/example/schema file exit `3`, and mutually exclusive flags exit `2`; unreadable-file diagnostics must identify the affected path and a safe reason on stderr in `tests/cli.rs`
- [X] T045 [US4] Add compiled-CLI tests asserting stdout is used for completed validation and stderr for preventing failures; representative dotenv parse, unreadable-file, TOML parse, and schema-definition failures must render structured safe file context (path, safe reason, optional line/column when available), warnings alone exit `0`, validation errors exit `1`, schema failures exit `4`, and actual target environment values never appear in either stream in `tests/cli.rs`
- [X] T046 [US4] Add deterministic-output integration coverage by running the same representative validation exactly 100 times and asserting identical exit code, stdout, stderr, diagnostic content, and diagnostic order across all runs in `tests/cli.rs`

### Implementation for User Story 4

- [X] T047 [US4] Implement validator-definition resolution relative to the target file directory in `src/main.rs`: explicit `--schema`/`--example` override discovery; otherwise choose sibling `.env.schema`, then sibling `.env.example`, otherwise return input/discovery failure
- [X] T048 [US4] Complete application-level preventing-error classification and stable exit mapping in `src/main.rs`: clap usage `2`, unreadable file/discovery/dotenv prevention `3`, schema parse/definition prevention `4`, completed validation errors `1`, success/warnings `0`; every file/input/schema preventing error must carry category, affected path, safe reason, optional line, and optional column, with parser location preserved when available and no raw target value field
- [X] T049 [US4] Finalize safe deterministic rendering in `src/output/mod.rs`: completed-validation errors before warnings, variable name ascending, rule code ascending, line tie-breaker, then final success/summary line; preventing failures render their safe category/path/reason/optional-line/optional-column context to stderr, and no rendering path accepts or emits actual target values
- [X] T050 [US4] Complete orchestration in `src/main.rs` so file reads are read-only, all validation remains local/in-memory, no networking/remote schemas/secret managers/environment interpolation are invoked, and the domain validator stays independent of clap/output

**Checkpoint**: The complete MVP CLI contract is deterministic, automation-safe, read-only, and value-redacted.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Close cross-platform, quality-gate, security, and end-to-end gaps after all desired user stories are implemented.

- [X] T051 [P] Add cross-platform regression cases for LF/CRLF, temporary paths, case-sensitive variable identity, and portable file handling in `src/env/parser.rs` and `tests/cli.rs`
- [X] T052 [P] Add a GitHub Actions CI matrix in `.github/workflows/ci.yml` that runs the Envcheck test suite on `ubuntu-latest`, `macos-latest`, and `windows-latest`, so SC-007 is demonstrated on all three supported operating systems before release
- [X] T053 Review all test fixtures for synthetic-only environment values and add explicit redaction plus byte-for-byte read-only regression assertions in `tests/cli.rs`, proving `.env`, `.env.example`, and `.env.schema` remain unchanged after representative successful and failing invocations
- [X] T054 Execute every scenario documented in `specs/001-env-validation/quickstart.md` and update only that file if observed command examples or expected outcomes need correction
- [ ] T055 Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`, fixing any violations in `Cargo.toml`, `src/`, and `tests/` without changing the published contracts
- [ ] T056 Audit dependency/API usage against the design constraints in `Cargo.toml` and `src/env/parser.rs`, ensuring Envcheck uses only dotenv scanning/tokenization and never dotenv expansion, evaluation, decryption, environment injection, or network-capable behavior

**Checkpoint**: All constitution quality gates and documented end-to-end scenarios pass.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 — Setup**: No dependencies.
- **Phase 2 — Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3 — US1 (P1)**: Depends on Phase 2.
- **Phase 4 — US2 (P1)**: Depends on Phase 2; it may proceed in parallel with US1 after the shared foundation is complete, but both modify `src/main.rs`, `src/validation/validator.rs`, `src/validation/rules/presence.rs`, and `tests/cli.rs`, so those tasks require coordination if implemented concurrently.
- **Phase 5 — US3 (P2)**: Depends on the schema model/parser introduced by US2; therefore US2 must complete first.
- **Phase 6 — US4 (P2)**: Depends on both validator modes and schema failure classification; complete US1, US2, and US3 first.
- **Phase 7 — Polish**: Depends on all user stories selected for the release.

### User Story Dependencies

```text
Setup -> Foundation -> US1 --------------------┐
                    \-> US2 -> US3 ------------+-> US4 -> Polish
```

- **US1 (P1)**: Independent after Foundation.
- **US2 (P1)**: Independent after Foundation for valid explicit schema flows.
- **US3 (P2)**: Extends US2's schema parser/model with strict invalid-definition handling and is a mandatory correctness/release gate; no public MVP or release may stop after US2.
- **US4 (P2)**: Integrates the two validator modes into final discovery, exit, stream, security, and deterministic-output semantics.

### Within Each User Story

- Complete the story's test tasks first and observe failure before implementation.
- Implement data/model/parser prerequisites before rule orchestration.
- Implement individual rules before the story-level validator/orchestration task.
- Complete compiled-CLI integration wiring after domain behavior exists.
- Do not mark the story checkpoint complete until its Independent Test can run without relying on unfinished later stories.

### Parallel Opportunities

- In Phase 2, T003, T004, and T005 can be authored in parallel; T008, T010, and T011 target separate files and can proceed in parallel once their shared interfaces are understood.
- In US1, T013 can be authored while T014 is prepared because they target different files.
- In US2, T018–T024 are parallel test-authoring opportunities across separate modules; T026, T028–T033 are parallel implementation opportunities after their respective tests and shared model contracts are settled.
- In US3, T036/T037 and T038 target separate parser/rule concerns and can proceed in parallel before the schema-validation implementation is consolidated.
- Cross-platform regression work T051 and CI-matrix setup T052 can proceed in parallel; security/read-only regression T053 follows once representative CLI flows exist.

---

## Parallel Example: User Story 1

```text
Task T013: Add example-comparison unit tests in src/validation/rules/presence.rs
Task T014: Add explicit-example compiled-CLI tests in tests/cli.rs
```

After those tests are in place, implement T015–T017 in dependency order.

## Parallel Example: User Story 2

```text
Task T018: Schema model/deserialization tests in src/schema/model.rs
Task T020: Scalar type tests in src/validation/rules/value_type.rs
Task T021: Numeric boundary tests in src/validation/rules/numeric.rs
Task T022: String length tests in src/validation/rules/length.rs
Task T023: Allowed-value tests in src/validation/rules/allowed.rs
Task T024: Pattern tests in src/validation/rules/pattern.rs
```

After the relevant tests fail, implement T026 and T028–T033 across their separate modules, then consolidate with T027, T034, and T035.

---

## Implementation Strategy

### MVP First

The requested first implementation centers on both P1 capabilities, but correctness requires US3 before any public release:

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: US1 — explicit `.env.example` comparison.
4. Complete Phase 4: US2 — explicit valid `.env.schema` validation for `string`, `integer`, `float`, and `boolean`.
5. Complete Phase 5: US3 — strict rejection of malformed, unsupported, or semantically invalid schemas.
6. **STOP AND VALIDATE** US1, US2, and US3 independently before treating the result as a release candidate.

US1 and US2 remain the feature-focus of the first implementation, while US3 is a mandatory safety/correctness gate rather than optional later hardening. US4 then completes automatic discovery and the full public CLI integration behavior.

### Incremental Delivery

1. Setup + Foundation -> parser/diagnostic/CLI foundations ready.
2. US1 -> `.env.example` comparison demonstrable independently.
3. US2 -> typed schema validation demonstrable independently.
4. US3 -> invalid schemas reliably rejected.
5. US4 -> final discovery, streams, exit codes, redaction, and determinism contract complete.
6. Polish -> cross-platform and quality gates verified.

### Parallel Team Strategy

After Foundation:

- Developer A can implement US1.
- Developer B can begin US2 on schema-specific files.
- Coordinate shared edits to `src/main.rs`, `src/validation/validator.rs`, `src/validation/rules/presence.rs`, and `tests/cli.rs` rather than editing those files concurrently.
- US3 follows US2, and US4 follows completed validator modes.

---

## Notes

- `[P]` tasks touch distinct files and have no dependency on incomplete same-file work.
- `[US1]`–`[US4]` map directly to the four user stories in `spec.md`.
- Test tasks are mandatory because the constitution requires unit coverage for every validation rule and integration coverage for CLI behavior/exit codes.
- Target `.env` values must never be copied into default diagnostic structures or rendered output.
- Inputs remain read-only and validation remains offline/local throughout implementation.
- Commit after each task or coherent dependency group.
- `$speckit-implement` should treat this file as the dependency-ordered execution plan and must not alter reviewer-owned checklist markers.