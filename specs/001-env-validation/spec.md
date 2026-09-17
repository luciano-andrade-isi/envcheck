# Feature Specification: Environment File Validation

**Feature Branch**: `chore/init-spec-kit`

**Created**: 2026-09-17

**Status**: Draft

**Input**: User description: "Create a command-line application called envcheck that validates a target .env file against either .env.example or .env.schema, with deterministic validation, safe diagnostics, stable exit semantics, and a bounded MVP."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Validate required variables from an example file (Priority: P1)

A developer wants to verify that a target environment file contains every variable declared by an
example file before running an application or CI job. The developer does not care about the example
values; only the presence of each key matters.

**Why this priority**: This is the smallest useful validation flow and solves the most common drift
problem between `.env.example` and `.env` files.

**Independent Test**: Provide a target `.env` and a `.env.example`, run validation, and verify that
missing keys and duplicate declarations are errors while target-only keys are warnings.

**Acceptance Scenarios**:

1. **Given** `.env.example` declares `APP_NAME`, `APP_ENV`, `DATABASE_URL`, and `REDIS_URL`, and the
   target `.env` lacks `REDIS_URL`, **When** validation runs, **Then** `REDIS_URL` is reported as a
   missing-variable error and the result is unsuccessful.
2. **Given** all keys from `.env.example` exist in the target `.env` in a different order and with
   different values, **When** validation runs, **Then** validation succeeds for required-key presence.
3. **Given** the target `.env` contains a key not declared in `.env.example`, **When** validation
   runs, **Then** the extra key is reported as a warning and does not by itself cause failure.
4. **Given** either file declares the same variable more than once, **When** validation runs, **Then**
   the duplicate is reported as a validation error and no first-value-wins or last-value-wins rule is
   applied.
5. **Given** the example file contains comments, blank lines, or example values, **When** validation
   runs, **Then** comments and blank lines are ignored and example values do not affect validity.

---

### User Story 2 - Validate typed environment values from a schema (Priority: P1)

A developer wants stronger validation than key presence and uses `.env.schema` to declare which
variables are required, which scalar type each variable accepts, and which supported constraints
apply to those variables.

**Why this priority**: Schema validation is a core product capability and provides the main value
beyond a simple example-file comparison.

**Independent Test**: Provide a target `.env` and a valid version-1 `.env.schema` containing each MVP
scalar type and constraint, then verify valid values succeed and invalid values produce precise
variable-level errors without exposing the actual values.

**Acceptance Scenarios**:

1. **Given** a required integer variable has `min = 1` and `max = 65535`, **When** the target contains
   `1` or `65535`, **Then** the value is valid because numeric bounds are inclusive.
2. **Given** a boolean variable contains any casing of `true` or `false`, **When** validation runs,
   **Then** the value is accepted; values such as `1`, `0`, `yes`, `no`, `on`, and `off` are rejected.
3. **Given** a required variable is absent, **When** validation runs, **Then** a required-variable
   error is reported.
4. **Given** an existing variable is empty and `allow_empty = false` or omitted, **When** validation
   runs, **Then** an empty-value error is reported.
5. **Given** an existing variable is empty and `allow_empty = true`, **When** validation runs, **Then**
   the empty value is accepted without applying type-specific or value-specific constraints.
6. **Given** a string variable defines an `allowed` set, **When** its value is outside that set,
   **Then** a validation error is reported; string matching is case-sensitive.
7. **Given** a string variable defines a valid regular-expression pattern, **When** its value does not
   match, **Then** a pattern violation is reported.
8. **Given** a variable contains a quoted dotenv value, **When** validation runs, **Then** validation
   uses the parsed value without surrounding quote characters.
9. **Given** a dotenv line uses a supported `export` prefix, **When** validation runs, **Then** the
   prefixed variable is treated as the declared environment variable.
10. **Given** a value contains `${NAME}` syntax, **When** validation runs, **Then** the literal parsed
    value is validated and no interpolation is performed.

---

### User Story 3 - Detect invalid validation definitions safely (Priority: P2)

A developer wants configuration mistakes in `.env.schema` to fail clearly rather than silently
weakening validation.

**Why this priority**: A validator cannot be trusted if malformed or incompatible rules are ignored.
Rejecting invalid definitions protects correctness and prevents false-success results.

**Independent Test**: Provide malformed schemas, unsupported properties, incompatible constraints,
invalid patterns, and unsupported schema versions, and verify each is rejected as a schema error
before target-variable validation is considered successful.

**Acceptance Scenarios**:

1. **Given** `.env.schema` is not valid TOML, **When** validation starts, **Then** a schema error is
   reported and the command fails.
2. **Given** a schema contains an unknown property, **When** validation starts, **Then** the schema is
   rejected rather than ignoring the property.
3. **Given** an integer variable declares `min_length`, **When** validation starts, **Then** the schema
   is rejected because the constraint is incompatible with the declared type.
4. **Given** a string variable declares an invalid regular expression, **When** validation starts,
   **Then** the schema is rejected rather than ignoring the pattern.
5. **Given** a schema declares a version other than the supported MVP version, **When** validation
   starts, **Then** the schema is rejected as unsupported.

---

### User Story 4 - Select a validation definition and consume safe results (Priority: P2)

A developer or CI pipeline wants predictable validator selection, severity classification, and exit
semantics so validation can be used reliably in automation.

**Why this priority**: Deterministic selection and stable success/failure semantics are required for
repeatable local and CI usage.

**Independent Test**: Exercise explicit and automatic definition selection with successful results,
warnings, validation errors, invalid arguments, unreadable files, and malformed schemas, then verify
the selected definition, diagnostic severity, and process success/failure semantics.

**Acceptance Scenarios**:

1. **Given** no validator path is explicitly supplied and both `.env.schema` and `.env.example` exist
   beside the target file, **When** validation runs, **Then** `.env.schema` is selected.
2. **Given** no validator path is explicitly supplied and only `.env.example` exists beside the
   target file, **When** validation runs, **Then** `.env.example` is selected.
3. **Given** neither validation definition exists, **When** validation runs, **Then** the command
   reports that no validation definition could be found and fails.
4. **Given** validation produces warnings but no errors, **When** the command completes, **Then** the
   command reports successful validation and returns successful process status.
5. **Given** validation errors, invalid command arguments, unreadable input files, or a malformed
   schema, **When** the command completes, **Then** it returns a non-success process status.
6. **Given** a secret value violates a rule, **When** the diagnostic is displayed, **Then** the
   variable name and violated expectation are shown but the actual environment value is not shown.

### Edge Cases

- A target `.env` exists but is empty.
- `.env.example` is empty: target variables are all additional variables and therefore warnings by
  default, unless the target itself contains duplicates.
- An optional schema variable is absent.
- A required schema variable exists with an empty value.
- A quoted value contains spaces, `#`, or `=` characters that are part of the value.
- A variable name occurs multiple times with different values; duplication remains an error.
- Integer and float values occur exactly at minimum or maximum boundaries.
- Integer input contains decimal notation and must not be accepted as an integer.
- Float input uses a valid floating-point numeric representation.
- A string is shorter or longer than the declared length boundaries by exactly one character.
- An `allowed` set contains values incompatible with the variable's declared type.
- `min` is greater than `max`, or `min_length` is greater than `max_length`.
- A schema constraint uses an invalid value type, such as a negative string length.
- A schema contains a variable table without a recognized type.
- A schema contains an unknown top-level field or unknown variable-rule field.
- A schema uses a duplicate TOML key or otherwise violates TOML syntax.
- A file cannot be opened or read.
- Both an explicit example path and an explicit schema path are supplied at the same time.
- Validation is repeated with identical files and application version.

## Requirements *(mandatory)*

### Functional Requirements

#### Input and validator selection

- **FR-001**: The command MUST accept an explicit path to the target environment file.
- **FR-002**: The command MUST allow an explicit `.env.example` path to select example-file
  validation.
- **FR-003**: The command MUST allow an explicit `.env.schema` path to select schema validation.
- **FR-004**: Supplying both an explicit example path and an explicit schema path in the same
  invocation MUST be rejected as invalid command usage.
- **FR-005**: When no validation definition is explicitly supplied, the command MUST first look for
  `.env.schema` in the same directory as the target environment file.
- **FR-006**: When no explicit definition is supplied and `.env.schema` is absent, the command MUST
  look for `.env.example` in the same directory as the target environment file.
- **FR-007**: When neither definition can be found, the command MUST report that no validation
  definition is available and fail.
- **FR-008**: When both auto-discovered files exist, `.env.schema` MUST take precedence.
- **FR-009**: Unreadable target or validation-definition files MUST cause a non-success result with
  an actionable file-level diagnostic.

#### Dotenv parsing

- **FR-010**: Environment and example files MUST ignore blank lines and comment-only lines.
- **FR-011**: The parser MUST distinguish a missing variable from a declared variable with an empty
  value.
- **FR-012**: Duplicate variable declarations in the target `.env` MUST produce a validation error.
- **FR-013**: Duplicate variable declarations in `.env.example` MUST produce a validation error.
- **FR-014**: Duplicate declarations MUST NOT use first-value-wins or last-value-wins semantics for
  validation.
- **FR-015**: Common single-quoted and double-quoted dotenv values MUST be interpreted without their
  surrounding quote characters.
- **FR-016**: Common dotenv `export` prefixes MUST be accepted and the following name MUST be treated
  as the variable key.
- **FR-017**: The command MUST NOT interpolate variable references or consult the machine's current
  environment when interpreting dotenv values.

#### Example-file validation

- **FR-018**: Every key declared in `.env.example` MUST exist in the target `.env`.
- **FR-019**: Key order MUST NOT affect example-file validity.
- **FR-020**: Values declared in `.env.example` MUST NOT affect validation; only key existence matters.
- **FR-021**: A key present only in the target `.env` MUST produce a warning by default and MUST NOT
  cause failure by itself.

#### Schema structure and validity

- **FR-022**: `.env.schema` MUST use TOML syntax.
- **FR-023**: The MVP schema MUST declare `version = 1` and a `variables` table.
- **FR-024**: A malformed TOML schema MUST produce a schema error and MUST NOT be partially applied.
- **FR-025**: An unsupported schema version MUST produce a schema error.
- **FR-026**: Unknown top-level schema properties and unknown variable-rule properties MUST produce a
  schema error rather than being ignored.
- **FR-027**: Every variable rule MUST declare one of the supported types: `string`, `integer`,
  `float`, or `boolean`.
- **FR-028**: The MVP schema MUST recognize only these rule properties: `type`, `required`,
  `allow_empty`, `min`, `max`, `min_length`, `max_length`, `allowed`, and `pattern`.
- **FR-029**: `required` MUST define whether the variable must exist; when omitted it MUST default to
  false.
- **FR-030**: `allow_empty` MUST define whether an existing variable may be empty; when omitted it
  MUST default to false.
- **FR-031**: If an existing variable is empty and `allow_empty = true`, the empty value MUST be
  accepted and type-specific or value-specific constraints MUST not be applied to that empty value.
- **FR-032**: If an existing variable is empty and `allow_empty = false`, the command MUST report an
  empty-value validation error.
- **FR-033**: `min` and `max` MUST be valid only for `integer` and `float` variables.
- **FR-034**: For an `integer` variable, `min` and `max` constraint values MUST themselves represent
  integers; for a `float` variable they MUST represent numeric values.
- **FR-035**: Numeric `min` and `max` bounds MUST be inclusive.
- **FR-036**: `min_length` and `max_length` MUST be valid only for `string` variables and MUST be
  non-negative whole numbers.
- **FR-037**: `pattern` MUST be valid only for `string` variables.
- **FR-038**: An invalid regular expression in `pattern` MUST make the schema invalid.
- **FR-039**: `allowed` MAY restrict any supported scalar type, and each allowed entry MUST be
  compatible with the variable's declared type.
- **FR-040**: String comparisons against `allowed` MUST be case-sensitive.
- **FR-041**: A schema MUST be rejected when a constraint is incompatible with the declared variable
  type, has an invalid constraint value type, or defines contradictory bounds such as `min > max` or
  `min_length > max_length`.

#### Type and value validation

- **FR-042**: String variables MUST be validated as parsed dotenv text and MUST honor applicable
  length, allowed-value, and pattern constraints.
- **FR-043**: Integer variables MUST accept only valid integer representations and MUST reject values
  that require floating-point interpretation.
- **FR-044**: Float variables MUST accept valid floating-point numeric representations.
- **FR-045**: Boolean variables MUST accept only `true` and `false`, matched case-insensitively.
- **FR-046**: Boolean variables MUST reject numeric and convenience aliases including `1`, `0`,
  `yes`, `no`, `on`, and `off`.
- **FR-047**: A required schema variable that is absent from the target `.env` MUST produce a
  validation error.
- **FR-048**: An optional schema variable that is absent from the target `.env` MUST be valid.
- **FR-049**: A target variable not declared by the schema MUST produce a warning by default and MUST
  NOT cause failure by itself.

#### Diagnostics, exit semantics, and safety

- **FR-050**: Validation output MUST distinguish errors, warnings, and successful validation.
- **FR-051**: Variable-level validation errors MUST identify the variable name, violated rule, and
  expected type or constraint when applicable.
- **FR-052**: Validation diagnostics MUST NOT display the actual target environment-variable value by
  default.
- **FR-053**: Warnings alone MUST result in successful process status.
- **FR-054**: One or more validation errors MUST result in non-success process status.
- **FR-055**: Invalid command-line arguments, unreadable required files, and malformed or invalid
  schemas MUST result in non-success process status.
- **FR-056**: Exact numeric exit-code assignments MAY be chosen during technical planning, but once
  published their meanings MUST remain stable.
- **FR-057**: Diagnostic ordering MUST be deterministic for identical input files and application
  version.

#### Safety, determinism, and scope boundaries

- **FR-058**: The application MUST be read-only and MUST NOT modify the target `.env`,
  `.env.example`, or `.env.schema`.
- **FR-059**: Validation MUST NOT add missing variables or rewrite any input file automatically.
- **FR-060**: Validation MUST NOT require network access, remote services, secret managers, machine
  environment variables, or external configuration sources.
- **FR-061**: Given identical input files and application version, validation MUST produce the same
  validity classification, diagnostics, warning/error severities, and process success/failure
  semantics.
- **FR-062**: The MVP MUST NOT implement variable interpolation, remote schemas, secret-manager
  integration, automatic environment-file modification, custom user-defined validation functions,
  nested configuration objects, arrays as dedicated environment value types, URL validation, email
  validation, IP-address validation, filesystem-path validation, or duration validation.

### Key Entities

- **Target Environment File**: The read-only dotenv file containing the environment-variable
  declarations to validate. A declaration has a key, an existing/empty distinction, and a parsed
  literal value.
- **Example Definition**: A dotenv-style definition whose declared keys form the required key set;
  its values do not constrain target values.
- **Schema Definition**: A versioned validation definition containing a set of variable rules.
- **Variable Rule**: A schema entry for one variable, containing its scalar type, presence/empty-value
  policy, and any compatible constraints.
- **Diagnostic**: A deterministic error or warning containing enough information to identify the
  variable or file-level problem and expected rule without exposing target values.
- **Validation Result**: The aggregate success/failure state and diagnostics used to determine
  process success or non-success semantics.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In acceptance tests covering all supported schema types and constraints, 100% of valid
  boundary values are accepted and 100% of invalid boundary values are rejected according to this
  specification.
- **SC-002**: In example-file acceptance tests, every missing required key and every duplicate
  declaration is detected, while target-only keys produce warnings without causing failure.
- **SC-003**: In schema-quality acceptance tests, malformed syntax, unknown properties, incompatible
  constraints, invalid regular expressions, contradictory bounds, and unsupported versions are all
  rejected rather than silently ignored.
- **SC-004**: Across 100 repeated validations using identical files and application version, the
  validation classification, diagnostic content/order, and process success/failure result are
  identical on every run.
- **SC-005**: Across diagnostics produced for representative secret-bearing inputs, zero actual target
  environment-variable values appear in default output.
- **SC-006**: A user can validate a target file with either an explicit definition path or automatic
  local discovery using a single command invocation and without any network or remote-service
  dependency.
- **SC-007**: The MVP acceptance suite demonstrates the same documented validation semantics on
  Linux, macOS, and Windows for supported file content and command behavior.

## Assumptions

- The target environment file path is explicit in the MVP; automatic discovery applies to the
  validation definition, not to the target file itself.
- Automatic `.env.schema` / `.env.example` discovery is relative to the target environment file's
  directory, making behavior independent of the process's current working directory.
- Schema version `1` is the only supported schema version in the MVP; other versions fail explicitly.
- `required` defaults to false when omitted, while `allow_empty` defaults to false as explicitly
  defined by this feature.
- When `allow_empty = true`, an empty existing value is accepted as an intentional empty value and
  type/value constraints are skipped for that value.
- `allowed` uses the declared scalar type for comparison; string entries are case-sensitive, numeric
  entries use numeric equality, and boolean entries use boolean meaning after accepted boolean
  parsing.
- Dotenv parsing follows common semantics for comments, blank lines, quoted values, and `export`
  prefixes, while interpolation is deliberately disabled.
- Exact numeric exit codes and exact command flag names are deferred to technical planning, provided
  the success/non-success semantics defined here are preserved.
- Strict handling of additional variables is intentionally excluded from the MVP; additional
  variables remain warnings only.
