# Data Model: Environment File Validation

This document defines the logical in-memory model for the first Envcheck implementation. It is intentionally independent from clap and terminal rendering.

## 1. Parsed environment document

### `EnvDocument`

Represents one parsed dotenv-style file (`.env` or `.env.example`).

Fields:

- `path`: source path used only for file-level diagnostics.
- `entries`: ordered sequence of `EnvEntry` values in source order.
- `key_index`: deterministic lookup from variable name to one or more entry indexes.

Rules:

- Parsing never reads or writes process environment variables.
- Parsing never interpolates `${NAME}` or `$NAME`.
- The parser preserves enough multiplicity information to detect duplicate keys.
- A malformed non-empty/non-comment line prevents successful parsing.
- Variable names are case-sensitive and must match `[A-Za-z_][A-Za-z0-9_]*`.
- File contents remain read-only.

### `EnvEntry`

Represents one declaration in a dotenv file.

Fields:

- `key: String`
- `value: String`
- `line: usize`

Security constraint:

- `value` is sensitive by default.
- `EnvEntry` values are never formatted directly into user-visible diagnostics.
- Production diagnostic paths should not rely on an automatically derived debug representation that exposes `value`.

Value states:

- Missing variable: no `EnvEntry` exists for the key.
- Empty variable: an entry exists and `value == ""`.
- Non-empty variable: an entry exists and `value != ""`.

These states are semantically distinct.

## 2. Validation definition

### `ValidationDefinition`

Logical enum identifying the selected validation source:

- `Example(EnvDocument)`
- `Schema(SchemaDefinition)`

Selection is handled outside the domain validator. The validator receives one resolved definition and does not perform filesystem discovery itself.

## 3. Schema definition

### `SchemaDefinition`

Represents a successfully deserialized version-1 `.env.schema` before or after semantic validation.

Fields:

- `version: u32`
- `variables: BTreeMap<String, VariableRule>`

Rules:

- `version` must equal `1`.
- `variables` must be present.
- Unknown top-level fields are rejected during deserialization.
- Variable names use the same case-sensitive identifier grammar as dotenv files.
- Semantic validation must complete successfully before the schema can be used to validate a target environment file.

### `VariableType`

Supported variants:

- `String`
- `Integer`
- `Float`
- `Boolean`

No other schema type is part of the first implementation.

### `VariableRule`

Fields:

- `type: VariableType` — required.
- `required: bool` — default `false`.
- `allow_empty: bool` — default `false`.
- `min: Option<SchemaScalar>`
- `max: Option<SchemaScalar>`
- `min_length: Option<u64>`
- `max_length: Option<u64>`
- `allowed: Option<Vec<SchemaScalar>>`
- `pattern: Option<String>`

Unknown rule properties are rejected during deserialization.

### `SchemaScalar`

Preserves the TOML scalar kind of constraint values:

- `String(String)`
- `Integer(i64)`
- `Float(f64)`
- `Boolean(bool)`

This is used for `allowed` and numeric bounds so incompatible schema values can be rejected explicitly rather than coerced silently.

## 4. Schema semantic validation

A parsed `SchemaDefinition` transitions through two logical states:

```text
TOML text
  -> deserialized schema
  -> semantic schema validation
  -> usable schema
```

A schema is unusable if any semantic error is present.

### Constraint applicability matrix

| Constraint | string | integer | float | boolean |
|---|---:|---:|---:|---:|
| `required` | yes | yes | yes | yes |
| `allow_empty` | yes | yes | yes | yes |
| `min` | no | yes | yes | no |
| `max` | no | yes | yes | no |
| `min_length` | yes | no | no | no |
| `max_length` | yes | no | no | no |
| `allowed` | yes | yes | yes | yes |
| `pattern` | yes | no | no | no |

Additional semantic rules:

- Integer `min`/`max` must be integer schema scalars.
- Float `min`/`max` must be finite numeric schema scalars accepted by the schema contract.
- `min <= max` when both numeric bounds exist.
- `min_length <= max_length` when both length bounds exist.
- String lengths are non-negative whole numbers.
- `pattern` must compile with the Rust `regex` crate.
- Every `allowed` item must be compatible with the declared variable type.
- Unknown types or incompatible constraints invalidate the entire schema.

## 5. Parsed target value semantics

Validation converts a non-empty dotenv string into the declared schema type only for validation purposes. The original parsed string remains unchanged.

### String

- Uses the parsed dotenv value after quote handling.
- Length is measured in Unicode scalar values.
- `allowed` uses case-sensitive string equality.
- `pattern` uses Rust regex matching without implicit anchors.

### Integer

- Must parse as a valid signed integer representation accepted by the implementation contract.
- Floating-point notation is invalid for integer variables.
- Numeric bounds are inclusive.

### Float

- Accepts finite decimal and scientific-notation forms.
- Rejects `NaN`, positive infinity, and negative infinity.
- Numeric bounds are inclusive.

### Boolean

- Accepts only `true` or `false`, case-insensitively.
- `1`, `0`, `yes`, `no`, `on`, and `off` are invalid.

### Empty handling

For an existing empty value:

- `allow_empty = false` -> validation error.
- `allow_empty = true` -> accepted and all type/value-specific rules are skipped for that entry.

For an absent value:

- `required = true` -> validation error.
- `required = false` -> valid; no other rules apply.

## 6. Validation diagnostics

### `Severity`

Variants:

- `Error`
- `Warning`

Success is represented by the absence of errors, not by a synthetic success diagnostic inside the domain layer.

### `RuleCode`

Stable internal/public diagnostic identifiers should cover at least:

- `duplicate_key`
- `missing_required`
- `empty_not_allowed`
- `type_mismatch`
- `below_min`
- `above_max`
- `too_short`
- `too_long`
- `not_allowed`
- `pattern_mismatch`
- `additional_variable`

Parser/schema-definition failures use a separate application-error taxonomy because they prevent or invalidate validation rather than representing target-rule failures.

### `Diagnostic`

Fields:

- `severity: Severity`
- `code: RuleCode`
- `variable: Option<String>`
- `rule: String` or equivalent structured rule descriptor
- `expected: Option<String>`
- `line: Option<usize>` when safe and useful

Security invariant:

- No field stores the actual target environment-variable value for presentation.

Deterministic ordering key:

1. severity (`Error` before `Warning`)
2. variable name (`None` before named values if mixed)
3. rule code
4. line number as a tie-breaker

## 7. Validation result

### `ValidationResult`

Fields:

- `diagnostics: Vec<Diagnostic>`

Derived behavior:

- `has_errors()` is true when at least one diagnostic has severity `Error`.
- warnings alone do not make validation fail.
- diagnostics are canonically sorted before presentation.

Outcome mapping:

```text
no errors -> exit 0
one or more domain validation errors -> exit 1
```

Application-level failures map to exit codes `2` through `4` according to the CLI contract and are not represented as successful `ValidationResult` values.

## 8. Example-file comparison model

For `.env.example` validation:

1. Parse target and example into `EnvDocument`.
2. Detect duplicate keys in both documents; duplicates are validation errors.
3. Compare example keys against target keys case-sensitively.
4. Missing example keys are errors.
5. Example values are ignored.
6. Target-only keys are warnings.
7. Ordering in either file has no effect on validity.

## 9. Schema-validation model

For `.env.schema` validation:

1. Parse target into `EnvDocument`.
2. Deserialize and semantically validate schema.
3. Detect duplicate keys in the target.
4. For each schema variable:
   - handle presence/absence;
   - handle empty/non-empty;
   - parse declared scalar type;
   - apply compatible constraints.
5. Emit warnings for target-only variables.
6. Sort structured diagnostics canonically.

No validator may mutate an input file or consult any external configuration source.
