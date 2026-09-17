# `.env.schema` Contract: Version 1

This document defines the public TOML contract for Envcheck schema version `1`.

## Top-level structure

A schema MUST be valid TOML and contain exactly these top-level properties:

```toml
version = 1

[variables]
```

`version` is required and MUST equal `1`.

`variables` is required and contains one table per environment variable.

Unknown top-level properties are schema errors.

## Variable declarations

Example:

```toml
[variables.APP_PORT]
type = "integer"
required = true
min = 1
max = 65535
```

Variable names:

- MUST match `[A-Za-z_][A-Za-z0-9_]*`;
- are case-sensitive;
- use the same identity rules as names in `.env` and `.env.example`.

`APP_PORT` and `app_port` therefore represent different variables.

## Supported variable properties

Each variable rule supports only:

- `type`
- `required`
- `allow_empty`
- `min`
- `max`
- `min_length`
- `max_length`
- `allowed`
- `pattern`

Unknown properties are schema errors.

`type` is mandatory. Supported values are:

- `string`
- `integer`
- `float`
- `boolean`

No implicit type inference is performed.

## Defaults

When omitted:

```text
required = false
allow_empty = false
```

`required` controls whether the variable must exist.

`allow_empty` controls whether an existing declaration may contain an empty value.

Missing and empty values are distinct states.

If an existing value is empty and `allow_empty = true`, it is accepted and no type-specific or value-specific constraint is applied to that empty value.

## Constraint applicability

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

A constraint used with an incompatible type invalidates the schema. It is never silently ignored.

## `string`

Example:

```toml
[variables.APP_NAME]
type = "string"
required = true
allow_empty = false
min_length = 1
max_length = 100
```

String rules:

- Values are the parsed dotenv strings after quote handling.
- `min_length` and `max_length` count Unicode scalar values, not UTF-8 bytes.
- Length bounds are inclusive.
- Length constraints MUST be non-negative whole numbers.
- `min_length > max_length` is a schema error.
- String equality inside `allowed` is case-sensitive.
- `pattern` is supported only for strings.

## `integer`

Example:

```toml
[variables.APP_PORT]
type = "integer"
required = true
min = 1
max = 65535
```

Integer rules:

- Target values must use a valid signed integer representation.
- Values requiring floating-point interpretation are invalid integers.
- Schema `min` and `max` MUST be TOML integer values.
- Numeric bounds are inclusive.
- `min > max` is a schema error.
- Integer `allowed` entries MUST be TOML integers.

## `float`

Example:

```toml
[variables.REQUEST_TIMEOUT]
type = "float"
required = false
min = 0.1
max = 60.0
```

Float rules:

- Target values may use finite decimal or scientific notation.
- `NaN`, positive infinity, and negative infinity are invalid.
- Schema `min`, `max`, and `allowed` numeric entries MAY use TOML integer or TOML float values.
- Integer schema values used for float constraints are compared by their numeric meaning.
- All normalized float constraint values MUST be finite.
- Numeric bounds are inclusive.
- `min > max` is a schema error.

## `boolean`

Example:

```toml
[variables.DEBUG]
type = "boolean"
required = false
```

Target boolean parsing is case-insensitive but accepts only the words:

```text
true
false
```

Therefore `TRUE`, `False`, and similar casing variants are valid.

The following are invalid in version `1`:

```text
1
0
yes
no
on
off
```

Boolean `allowed` entries MUST be TOML boolean values (`true` or `false`).

## `allowed`

`allowed` defines a finite set of accepted values.

Example:

```toml
[variables.LOG_LEVEL]
type = "string"
required = true
allowed = ["debug", "info", "warn", "error"]
```

Compatibility rules:

- `string`: every entry MUST be a TOML string; comparison is case-sensitive.
- `integer`: every entry MUST be a TOML integer.
- `float`: entries MAY be TOML integers or finite TOML floats and are compared numerically.
- `boolean`: every entry MUST be a TOML boolean.

An incompatible entry invalidates the entire schema.

An empty `allowed = []` set is valid schema syntax and means that no non-empty value can satisfy the constraint. `allow_empty = true` still permits an existing empty value because empty acceptance occurs before value-specific constraints.

## `pattern`

Example:

```toml
[variables.HOST]
type = "string"
pattern = "^[a-zA-Z0-9.-]+$"
```

Pattern rules:

- `pattern` applies only to `string` variables.
- The expression is compiled using the Rust `regex` crate.
- Invalid expressions invalidate the schema.
- Envcheck does not add implicit anchors. The schema author uses `^` and `$` when full-string matching is intended.

## Strict schema errors

The schema is rejected before target-variable validation if any of these conditions exists:

- TOML syntax is malformed;
- `version` is missing or not `1`;
- `variables` is missing;
- an unknown top-level property exists;
- an unknown variable-rule property exists;
- a variable name is invalid;
- `type` is missing or unsupported;
- a constraint is incompatible with the declared type;
- a constraint has an incompatible TOML value type;
- numeric bounds are non-finite;
- `min > max`;
- `min_length > max_length`;
- an `allowed` entry has an incompatible type;
- `pattern` cannot be compiled.

These failures are schema-definition errors and map to CLI exit code `4`.

## Complete version-1 example

```toml
version = 1

[variables.APP_NAME]
type = "string"
required = true
min_length = 1
max_length = 100

[variables.APP_ENV]
type = "string"
required = true
allowed = ["development", "staging", "production"]

[variables.APP_PORT]
type = "integer"
required = true
min = 1
max = 65535

[variables.DEBUG]
type = "boolean"
required = false

[variables.REQUEST_TIMEOUT]
type = "float"
required = false
min = 0.1
max = 60.0

[variables.API_KEY]
type = "string"
required = true
allow_empty = false
min_length = 32

[variables.HOST]
type = "string"
required = false
pattern = "^[a-zA-Z0-9.-]+$"
```

## Out of scope for version 1

The schema does not define:

- nested configuration values;
- arrays as environment value types;
- variable interpolation;
- remote schemas;
- custom user-defined functions;
- URL validation;
- email validation;
- IPv4/IPv6 validation;
- filesystem-path validation;
- duration validation;
- secret-manager references.

TOML arrays are used only as the representation of the `allowed` constraint; they do not make arrays a supported target environment-variable type.
