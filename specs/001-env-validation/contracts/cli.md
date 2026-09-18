# CLI Contract: Envcheck MVP

This contract defines the initial public command-line interface for the first Envcheck implementation. Once implemented and published, argument semantics, output classes, and exit-code meanings are compatibility-sensitive behavior.

## Invocation

```text
envcheck <ENV_FILE> [--example <FILE> | --schema <FILE>]
```

### Arguments

- `<ENV_FILE>`: required path to the target dotenv file.
- `--example <FILE>`: explicitly validate using the supplied `.env.example`-style file.
- `--schema <FILE>`: explicitly validate using the supplied `.env.schema` TOML file.
- `--example` and `--schema` are mutually exclusive.
- Standard clap-generated `--help` and `--version` behavior is part of the executable interface.

The target environment file is never auto-discovered in the MVP.

## Validation-definition selection

Selection is deterministic:

1. If `--schema` is supplied, use that path.
2. Else if `--example` is supplied, use that path.
3. Else inspect the directory containing `<ENV_FILE>`:
   1. use `.env.schema` when it exists;
   2. otherwise use `.env.example` when it exists;
   3. otherwise fail because no validation definition is available.

If both sibling `.env.schema` and `.env.example` exist, `.env.schema` takes precedence.

Explicit validator paths always override automatic discovery.

## Stable exit codes

| Exit code | Meaning |
|---:|---|
| `0` | Validation completed without validation errors. Warnings may be present. |
| `1` | Validation completed and one or more validation errors were found in the target/example comparison or target/schema validation. |
| `2` | Invalid CLI usage or arguments, including mutually exclusive validator flags supplied together. |
| `3` | Validation could not be performed because of input/discovery/dotenv parsing failure, such as an unreadable required file, malformed dotenv input, or no validation definition found. |
| `4` | `.env.schema` is syntactically malformed or semantically invalid, including unsupported version, unknown fields, incompatible constraints, invalid bounds, or invalid regex. |

These numeric assignments are part of the public CLI contract once the first implementation is released.

## Output channels

### Standard output

Used when validation ran to completion:

- validation errors;
- warnings;
- successful-validation summary.

Warnings without validation errors still exit with code `0`.

### Standard error

Used for failures that prevent a validation run from completing:

- CLI usage errors;
- unreadable required files;
- missing auto-discovered validation definition;
- malformed dotenv syntax;
- malformed or semantically invalid schema.

## Diagnostic contract

Completed validation output must distinguish:

- `error` — target/example or target/schema rule violation;
- `warning` — non-failing issue such as an additional target-only variable;
- `ok` — successful validation summary when no validation errors exist.

Variable-level errors identify:

- variable name;
- violated rule;
- expected type or constraint when relevant.

Default diagnostics MUST NOT include the actual target environment-variable value.

Safe examples:

```text
error: APP_PORT: value must be an integer between 1 and 65535
warning: LEGACY_FLAG: variable is not declared by the validation definition
ok: environment validation succeeded
```

A diagnostic for `API_KEY=short-secret` may identify `API_KEY` and its minimum-length rule but must never include `short-secret`.

## Deterministic ordering

For identical files and Envcheck version, completed validation output is ordered deterministically:

1. errors before warnings;
2. variable name ascending;
3. stable rule code ascending;
4. source line as a tie-breaker when required.

A final success/summary line is rendered after diagnostics.

User-visible order must never depend on hash-map iteration order or operating-system environment state.

## Read-only and offline behavior

Invoking Envcheck never:

- modifies `<ENV_FILE>`;
- modifies `.env.example`;
- modifies `.env.schema`;
- adds missing variables;
- contacts a network service;
- reads a secret manager;
- resolves environment-variable interpolation from the machine environment.

## Acceptance matrix

The CLI integration suite must cover these public outcomes after implementation:

| Scenario | Expected class | Exit |
|---|---|---:|
| Explicit example validation succeeds | success | `0` |
| Explicit schema validation succeeds | success | `0` |
| Additional target-only variable only | warning + success | `0` |
| Missing required/example variable | validation error | `1` |
| Duplicate dotenv declaration | validation error | `1` |
| Both `--example` and `--schema` supplied | CLI usage error | `2` |
| Required file unreadable | input error | `3` |
| Invalid non-comment dotenv line | dotenv parse error | `3` |
| No validation definition found | discovery error | `3` |
| Malformed TOML schema | schema error | `4` |
| Unknown or incompatible schema property | schema error | `4` |
| Invalid regex in schema | schema error | `4` |
| Both sibling definitions exist with no explicit flag | `.env.schema` selected | depends on schema validation result |

## MVP exclusions

The CLI contract does not include:

- strict mode for additional variables;
- network or remote-schema options;
- secret-manager options;
- mutation/fix commands;
- custom validator plugins;
- output formats such as JSON;
- color/ANSI semantics as a compatibility guarantee.

These may only be introduced through later specifications without silently changing the behavior defined here.
