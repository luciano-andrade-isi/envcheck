# Quickstart: Validate the Envcheck MVP

This guide defines end-to-end scenarios to run after implementation. It is a validation guide, not implementation code.

## Prerequisites

- Stable Rust toolchain and Cargo.
- Repository checked out on the implementation branch.

Before functional scenarios, the project must pass:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

The compiled executable is named `envcheck`.

## Scenario 1: `.env.example` detects a missing variable

Create `.env.example`:

```dotenv
APP_NAME=
APP_ENV=development
DATABASE_URL=
REDIS_URL=
```

Create `.env`:

```dotenv
APP_NAME=MyApplication
APP_ENV=production
DATABASE_URL=postgres://localhost/app
```

Run:

```bash
cargo run -- .env --example .env.example
```

Expected result:

- `REDIS_URL` is identified as missing.
- Validation exits with code `1`.
- No environment value is printed as part of the error.

## Scenario 2: Additional variables are warnings only

Create `.env.example`:

```dotenv
APP_NAME=
```

Create `.env`:

```dotenv
APP_NAME=MyApplication
LEGACY_FLAG=enabled
```

Run:

```bash
cargo run -- .env --example .env.example
```

Expected result:

- `LEGACY_FLAG` is reported as an additional-variable warning.
- No validation error is produced.
- Process exits with code `0`.

## Scenario 3: Duplicate declarations are errors

Create `.env.example`:

```dotenv
PORT=
```

Create `.env`:

```dotenv
PORT=3000
PORT=4000
```

Run:

```bash
cargo run -- .env --example .env.example
```

Expected result:

- The duplicate `PORT` declaration is reported as an error.
- Neither first-value-wins nor last-value-wins semantics are used for validation.
- Process exits with code `1`.

Repeat with the duplicate in `.env.example`; it must also fail with exit code `1`.

## Scenario 4: Typed schema succeeds

Create `.env.schema`:

```toml
version = 1

[variables.APP_NAME]
type = "string"
required = true
min_length = 1

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
```

Create `.env`:

```dotenv
APP_NAME="My Application"
APP_PORT=8080
DEBUG=True
REQUEST_TIMEOUT=1.5e1
```

Run:

```bash
cargo run -- .env --schema .env.schema
```

Expected result:

- All values satisfy their declared types and constraints.
- Process exits with code `0`.

## Scenario 5: Numeric boundary violation

Using the previous schema, set:

```dotenv
APP_NAME=MyApplication
APP_PORT=0
```

Run:

```bash
cargo run -- .env --schema .env.schema
```

Expected result:

- `APP_PORT` is identified as violating the inclusive minimum of `1`.
- The diagnostic states the variable and expected constraint without printing `0` as the supplied value.
- Process exits with code `1`.

Repeat with `APP_PORT=1` and `APP_PORT=65535`; both boundary values must be valid.

## Scenario 6: Float syntax boundaries

Schema:

```toml
version = 1

[variables.VALUE]
type = "float"
required = true
```

Valid target examples include:

```dotenv
VALUE=1.5
```

and:

```dotenv
VALUE=1e3
```

Invalid target examples include `NaN`, positive infinity, and negative infinity.

Expected result:

- Finite decimal/scientific forms validate.
- Non-finite forms produce a validation error and exit code `1`.

## Scenario 7: Boolean contract

Schema:

```toml
version = 1

[variables.DEBUG]
type = "boolean"
required = true
```

The following target values must be accepted:

```text
true
false
TRUE
FALSE
True
False
```

The following must be rejected:

```text
1
0
yes
no
on
off
```

Accepted cases exit `0`; rejected cases exit `1`.

## Scenario 8: Empty and missing are different

Schema:

```toml
version = 1

[variables.API_KEY]
type = "string"
required = true
allow_empty = false
```

Test separately:

1. `API_KEY` absent -> required-variable error, exit `1`.
2. `API_KEY=` -> empty-not-allowed error, exit `1`.

Then change `allow_empty = true` and test `API_KEY=` again. It must be valid from the required/empty perspective and string-specific rules must be skipped for that empty value.

## Scenario 9: Secret-value redaction

Schema:

```toml
version = 1

[variables.API_KEY]
type = "string"
required = true
min_length = 32
```

Target:

```dotenv
API_KEY=short-secret
```

Run:

```bash
cargo run -- .env --schema .env.schema
```

Expected result:

- Output names `API_KEY` and its minimum-length expectation.
- Output does not contain `short-secret`.
- Process exits with code `1`.

## Scenario 10: Automatic validator discovery

Place `.env`, `.env.example`, and `.env.schema` in the same directory and invoke:

```bash
cargo run -- .env
```

Expected result:

- `.env.schema` is selected.

Remove `.env.schema` and run again:

- `.env.example` is selected.

Remove both validation-definition files and run again:

- Envcheck reports that no validation definition is available.
- Process exits with code `3`.

## Scenario 11: Malformed dotenv input

Create a target containing a non-empty/non-comment line that is not a valid dotenv declaration.

Expected result:

- Parsing fails rather than ignoring the line.
- Error is written to stderr.
- Process exits with code `3`.

Also cover:

- variable names beginning with a digit;
- variable names containing `-` or `.`;
- case-sensitive distinction such as `PORT` vs `port`;
- quoted values containing `#` or `=`;
- unquoted inline comments where `#` is preceded by whitespace;
- literal `${NAME}` with no interpolation;
- common `export NAME=value` syntax.

## Scenario 12: Invalid schema

Each of these should independently invalidate `.env.schema` and exit with code `4`:

- malformed TOML;
- unsupported `version`;
- unknown top-level property;
- unknown variable property;
- unsupported type;
- `min_length` on an integer;
- `min > max`;
- `min_length > max_length`;
- incompatible `allowed` entry;
- invalid regular expression.

Schema-definition failures are written to stderr and target-variable validation is not reported as successful.

## Scenario 13: CLI misuse

Run with both validator flags:

```bash
cargo run -- .env --example .env.example --schema .env.schema
```

Expected result:

- clap reports invalid usage.
- Process exits with code `2`.

## Integration-test execution

The implementation's integration suite must execute the compiled CLI, not call internal validation functions as a substitute for command-level testing.

Run:

```bash
cargo test --test cli
```

The integration suite should cover the exit-code and output-channel matrix in [contracts/cli.md](./contracts/cli.md).

## Cross-platform acceptance

Before the MVP is considered complete, the same public validation semantics must be demonstrated on:

- Linux;
- macOS;
- Windows.

Path separators and LF/CRLF line endings must not alter validation semantics. Test fixtures must contain synthetic values only.
