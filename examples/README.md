# Envcheck examples

These folders are runnable examples for the public CLI.

| Folder | Definitions present | Command | Expected exit | Purpose |
|---|---|---|---:|---|
| `01-example-only-simple-valid` | `.env.example` | `cargo run -- examples/01-example-only-simple-valid/.env` | 0 | Simple example-file success through discovery |
| `02-example-only-missing-key` | `.env.example` | `cargo run -- examples/02-example-only-missing-key/.env` | 1 | Missing key detected from example |
| `03-schema-only-simple-valid` | `.env.schema` | `cargo run -- examples/03-schema-only-simple-valid/.env` | 0 | Simple typed schema success |
| `04-schema-only-complex-valid` | `.env.schema` | `cargo run -- examples/04-schema-only-complex-valid/.env` | 0 | Complex valid schema with all supported scalar types and constraints |
| `05-both-schema-precedence` | both | `cargo run -- examples/05-both-schema-precedence/.env` | 1 | Discovery chooses schema before example |
| `06-both-explicit-example-override` | both | discovery: `cargo run -- examples/06-both-explicit-example-override/.env`; explicit: `cargo run -- examples/06-both-explicit-example-override/.env --example examples/06-both-explicit-example-override/.env.example` | 1 / 0 | Explicit example overrides discovered schema |
| `07-schema-complex-invalid-values` | `.env.schema` | `cargo run -- examples/07-schema-complex-invalid-values/.env` | 1 | Multiple deterministic validation errors plus an additional-variable warning |
| `08-invalid-schema-definition` | `.env.schema` | `cargo run -- examples/08-invalid-schema-definition/.env` | 4 | Semantic schema-definition failure |
| `09-no-definition` | neither | `cargo run -- examples/09-no-definition/.env` | 3 | Discovery failure when no definition exists |

All target values are synthetic. Envcheck should never print them in default diagnostics.
