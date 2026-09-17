# Envcheck Constitution

## Core Principles

### I. Correctness First
Validation behavior MUST be deterministic, reproducible, and explicitly tested. Given the same
inputs and configuration, Envcheck MUST produce the same validation result, diagnostics, and exit
code. Validation rules MUST have clearly defined semantics and MUST NOT depend on network access,
ambient mutable state, or nondeterministic ordering. Correctness takes precedence over convenience
or premature optimization.

### II. CLI Stability
Public command-line arguments, flags, exit codes, and output semantics are part of Envcheck's
public API. Changes that break existing CLI usage MUST be justified, documented, and treated as
compatibility-impacting changes. New behavior SHOULD preserve existing defaults unless a deliberate
breaking change is approved through the constitution amendment process.

### III. Clear Separation of Concerns
Parsing environment files, parsing schemas, applying validation rules, and presenting CLI output
MUST remain independent components with explicit boundaries. Validation logic MUST NOT depend on
terminal formatting, and parsing components MUST NOT embed presentation behavior. This separation
keeps rules testable and allows formats or presentation to evolve independently.

### IV. Test-Driven Behavior
Every validation rule MUST have unit tests covering valid, invalid, and relevant boundary values.
Integration tests MUST verify public CLI behavior, including arguments, output semantics, error
handling, and exit codes. Behavior changes MUST be accompanied by tests that demonstrate the new or
corrected contract.

### V. Actionable Errors
Validation errors MUST identify the environment variable, the violated rule, and the expected value,
type, or constraint when doing so does not disclose a secret. Diagnostics MUST be precise enough for
a user to correct the configuration without inspecting Envcheck internals. Secret values from `.env`
files MUST NOT appear in diagnostics by default.

### VI. Security and Secret Handling
Values contained in `.env` files MUST be treated as potentially sensitive. Envcheck MUST NOT print,
log, persist, or otherwise expose environment variable values by default. Features that intentionally
reveal values MUST require explicit user action, provide a clear security rationale, and avoid
accidental disclosure through normal error paths.

### VII. Minimal Dependencies
Envcheck SHOULD prefer mature, actively maintained Rust crates with a clear benefit to correctness,
portability, or maintainability. New dependencies MUST be justified when equivalent behavior is
trivial to implement safely within the project. Dependency additions MUST consider maintenance,
security, transitive dependency cost, and cross-platform support.

### VIII. Cross-Platform Behavior
The CLI MUST work on Linux, macOS, and Windows. Platform-specific behavior MUST be isolated and
covered by tests where practical. File handling, paths, line endings, terminal assumptions, and exit
behavior MUST avoid unnecessary platform-specific semantics.

### IX. Local and In-Memory Validation
Validation SHOULD operate in memory for normal Envcheck workloads and MUST NOT require network
access. The tool MUST remain suitable for local development, CI pipelines, and offline execution.
Performance work MUST preserve deterministic behavior and correctness.

### X. Rust Quality Gates
All committed Rust code MUST pass `cargo fmt`, `cargo clippy`, and `cargo test`. Clippy warnings that
are intentionally accepted MUST have a documented rationale close to the suppression. Unsafe Rust
MUST NOT be introduced unless a concrete requirement cannot reasonably be satisfied with safe Rust
and the safety invariants are documented and tested.

## Engineering Constraints

Envcheck is a small, portable, deterministic command-line application written in Rust. The design
MUST favor simple local execution over services, daemons, databases, or network dependencies.
Configuration and validation behavior MUST remain understandable from repository artifacts and CLI
contracts. Implementations SHOULD avoid unnecessary abstraction and MUST not weaken the separation
between parsing, validation, and presentation layers.

Security-sensitive inputs MUST be handled defensively. Tests and fixtures MUST use synthetic values
rather than real credentials. Performance optimizations MUST be supported by an observable need and
MUST NOT compromise diagnostics, determinism, portability, or testability.

## Development Workflow

Specifications and implementation plans MUST be checked against this constitution before coding
begins. Each change MUST identify the CLI contract and validation behavior it affects. Validation
rules MUST be specified in testable terms before or together with their implementation.

Before a change is considered complete, the project MUST pass `cargo fmt`, `cargo clippy`, and
`cargo test`. Changes to public arguments, exit codes, or output semantics MUST include integration
tests and an explicit compatibility justification. Dependency additions MUST include a brief
justification in the relevant plan or review context. Security reviews MUST verify that no normal
execution path exposes `.env` values.

## Governance

This constitution defines the non-negotiable engineering rules for Envcheck and takes precedence
over conflicting implementation preferences. Specifications, plans, tasks, code reviews, and release
changes MUST be checked for compliance with these principles.

Amendments MUST be documented in this file with an updated version and amendment date. Versioning
follows semantic versioning for governance: MAJOR for incompatible principle removals or
redefinitions, MINOR for new principles or materially expanded requirements, and PATCH for
clarifications that do not change obligations. Any amendment that relaxes security, correctness,
CLI compatibility, testing, or portability requirements MUST include explicit rationale and migration
impact before approval.

Compliance MUST be reviewed whenever a specification, implementation plan, or public CLI contract is
changed. Complexity or exceptions that conflict with a principle MUST be justified explicitly; if
the exception is intended to persist, the constitution MUST be amended rather than silently ignored.

**Version**: 1.0.0 | **Ratified**: 2026-09-17 | **Last Amended**: 2026-09-17
