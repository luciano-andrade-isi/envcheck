# Research: Environment File Validation

## Decision: Stable Rust and project baseline

**Decision**: Use the stable Rust toolchain with edition 2024 and Cargo. Do not publish a minimum supported Rust version in the MVP.

**Rationale**: The project is a new portable CLI and has no compatibility obligation to older Rust toolchains yet. Edition 2024 is compatible with the requested stable-Rust direction and current clap releases. Deferring an MSRV avoids creating a public support contract before there is a real consumer need.

**Alternatives considered**:
- Pin a specific rustc version: rejected because the requirement is stable Rust, not a fixed compiler release.
- Publish an MSRV immediately: rejected because it adds maintenance/testing obligations without an MVP requirement.

## Decision: CLI parsing with clap derive

**Decision**: Use `clap` 4.x with the `derive` feature and a single top-level argument struct.

**Rationale**: clap's derive API directly supports typed `PathBuf` arguments, mutually exclusive options, generated help/version output, and conventional usage errors. The public command remains intentionally small: one positional target path plus `--example` or `--schema`.

**Alternatives considered**:
- Manual `std::env::args` parsing: rejected because it would duplicate mature argument-validation behavior and weaken CLI consistency.
- Subcommands: rejected because the MVP has one operation: validate.

Reference: https://docs.rs/clap/latest/clap/_derive/

## Decision: Dotenv scanner

**Decision**: Use the non-expanding `scan` API from `dotenvx-primitives` as the dotenv parsing primitive, wrapped by a narrow Envcheck strictness adapter.

**Rationale**: The scan API is designed to inspect dotenv assignments without applying expansion or decryption and preserves duplicate assignments in source order. Those properties align unusually well with Envcheck's requirements: duplicates must be detectable, `${NAME}` must remain literal, and validation must not consult the process environment. The adapter remains responsible for Envcheck-specific strictness such as rejecting malformed non-comment lines, enforcing `[A-Za-z_][A-Za-z0-9_]*`, and locking down the clarified inline-comment behavior.

The full `parse`/load APIs of dotenv libraries are deliberately avoided because expansion or environment injection would violate deterministic validation. In particular, `dotenvy` is mature but its parser performs substitution and may consult process environment values during substitution; that makes it unsuitable for Envcheck's no-interpolation contract even though its iterator can expose assignments.

**Alternatives considered**:
- `dotenvy`: mature and well maintained, but rejected for this use because its parsing path performs variable substitution and can read the ambient process environment.
- `dotenv-parser`: simple and handles comments/quotes/export, but its public API returns a `BTreeMap`, which erases duplicate declarations before Envcheck can classify them.
- Fully custom dotenv parser: rejected as the primary approach because the project explicitly prefers a mature parser. A small adapter is permitted only for strictness gaps required by the Envcheck specification.

References:
- https://docs.rs/dotenvx-primitives/latest/
- https://docs.rs/dotenvy/latest/src/dotenvy/parse.rs.html
- https://docs.rs/dotenv-parser/latest/dotenv_parser/fn.parse_dotenv.html

## Decision: Strict TOML schema deserialization

**Decision**: Use `serde` derive + `toml` and apply `#[serde(deny_unknown_fields)]` to both the top-level schema structure and per-variable rule structure. Keep TOML syntax parsing separate from semantic schema validation.

**Rationale**: The specification requires unknown properties to fail instead of being ignored. Serde explicitly supports rejecting unknown fields. TOML deserialization gives a mature parser while Envcheck's semantic stage can validate version, type compatibility, contradictory bounds, regex compilation, and allowed-value compatibility.

**Alternatives considered**:
- Deserialize into generic `toml::Value` only: rejected as the main model because it makes unknown-field enforcement and required-field structure less explicit.
- Hand-written TOML parsing: rejected because it duplicates a mature format parser and conflicts with the requested Serde/TOML direction.

References:
- https://serde.rs/container-attrs.html
- https://docs.rs/toml/latest/toml/

## Decision: Schema numeric and scalar representation

**Decision**: Preserve TOML scalar kinds during schema parsing so semantic validation can distinguish string, integer, float, and boolean constraint values. Integer validation uses signed 64-bit integer semantics; float validation uses finite IEEE-754 `f64` values. Non-finite schema bounds are rejected.

**Rationale**: The MVP types are scalar and TOML already distinguishes their source types. Preserving the TOML kind lets Envcheck reject incompatible `allowed`, `min`, and `max` values rather than coercing them silently. `f64` is the conventional Rust floating-point representation and supports the clarified decimal/scientific syntax.

**Alternatives considered**:
- Coerce every numeric schema value to float: rejected because integer-specific constraints could silently lose type information.
- Arbitrary precision numbers: rejected because the MVP does not require them and they would add dependencies/complexity.

## Decision: String length semantics

**Decision**: `min_length` and `max_length` count Unicode scalar values (`str::chars().count()` semantics), not UTF-8 bytes.

**Rationale**: A user-facing string-length constraint should not vary with UTF-8 byte width. Unicode scalar counting is deterministic, dependency-free, and clearer than byte count. Grapheme-cluster counting is not required by the MVP and would add a Unicode-segmentation dependency.

**Alternatives considered**:
- UTF-8 bytes: rejected because non-ASCII strings would have surprising lengths.
- Grapheme clusters: rejected because the MVP does not require user-perceived text segmentation.

## Decision: Regular-expression semantics

**Decision**: Use the Rust `regex` crate. Patterns are compiled when the schema is validated. Matching follows `Regex::is_match` semantics with no implicit anchors; schema authors use `^...$` (or equivalent) when full-string matching is required.

**Rationale**: The user explicitly selected the `regex` crate. Compilation during schema validation ensures invalid patterns fail as schema errors before target values are evaluated. The crate documents linear-time matching guarantees for supported regex syntax.

**Alternatives considered**:
- Implicitly anchor every pattern: rejected because that silently changes the schema author's regular expression.
- PCRE-compatible engine: rejected because it adds complexity and is not requested.

Reference: https://docs.rs/regex/latest/regex/struct.Regex.html

## Decision: Stable exit-code contract

**Decision**:
- `0`: validation completed with no validation errors; warnings allowed.
- `1`: validation completed with one or more validation errors.
- `2`: invalid CLI usage/arguments.
- `3`: input/discovery/dotenv parsing failure that prevents validation.
- `4`: malformed or semantically invalid schema.

**Rationale**: This separates target-configuration invalidity from usage, I/O/parsing, and definition errors while preserving conventional success `0`. Code `2` aligns with clap's argument-error behavior. The assignments are documented before implementation so integration tests can lock them down as a public API.

**Alternatives considered**:
- One generic nonzero exit code: rejected because it gives automation no stable distinction between validation failure and tool/input failure.
- Many rule-specific exit codes: rejected because rule details belong in diagnostics and would make the CLI contract brittle.

## Decision: Structured diagnostics and secret handling

**Decision**: Validation returns structured diagnostics that never require the raw target value. A diagnostic contains severity, stable rule code, optional variable name, and a safe expectation/message. Types that hold parsed `.env` values should not expose those values through default debug formatting in production output paths.

**Rationale**: This makes the constitution's secret-handling rule architectural instead of relying on every formatting call to remember redaction. Output is centralized in `src/output/mod.rs`.

**Alternatives considered**:
- Return formatted strings directly from validation rules: rejected because it couples domain validation to presentation and increases accidental secret-disclosure risk.
- Generic logging of parsed objects: rejected because `.env` values are sensitive by default.

## Decision: Deterministic ordering

**Decision**: Use deterministic collections where ordering affects behavior (`BTreeMap` for schema rule lookup/iteration where appropriate) and explicitly sort diagnostics by severity, variable name, then stable rule code before rendering.

**Rationale**: Hash-map iteration order must not affect output. Canonical sorting makes repeated runs and cross-platform integration tests reproducible.

**Alternatives considered**:
- Preserve arbitrary map iteration: rejected as nondeterministic.
- Preserve only source order: viable for dotenv-originated findings, but insufficient for a unified result assembled from target, example, and schema structures. Canonical sorting is simpler as a public output contract.

## Decision: Testing tools and levels

**Decision**: Use three layers:
1. Unit tests adjacent to dotenv/schema parsers.
2. Unit tests for each validation rule, with valid, invalid, and boundary cases.
3. `tests/cli.rs` integration tests using `assert_cmd` and `tempfile` to execute the compiled `envcheck` binary across realistic files.

**Rationale**: This directly implements the constitution's test-driven requirement. `assert_cmd` specifically supports Cargo-built binary invocation and exit/stdout/stderr assertions, while `tempfile` provides portable temporary directories/files.

**Alternatives considered**:
- Snapshot-only CLI tests: rejected because rule-level failures would be harder to isolate.
- Shell scripts as the primary integration test mechanism: rejected because they are less portable to Windows.

References:
- https://docs.rs/assert_cmd/latest/assert_cmd/
- https://docs.rs/tempfile/latest/tempfile/

## Decision: Dependency restraint

**Decision**: Do not add `anyhow`, `thiserror`, logging frameworks, async runtimes, network clients, serialization formats beyond TOML, or terminal-color libraries for the MVP unless a later task demonstrates a concrete requirement.

**Rationale**: Standard-library error enums and plain deterministic text output are sufficient for this small CLI. This preserves portability, compile-time simplicity, and the constitution's minimal-dependency principle.

**Alternatives considered**:
- Generic application-error crates: useful in larger applications, but not necessary for the bounded error taxonomy here.
- Colorized output library: deferred because color is presentation-only and not required for correctness or usability of the MVP.
