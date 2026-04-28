# Development

## Language

Repository docs, code, comments, tests, fixtures, and commit messages must
be English.

## Secrets

Secrets never commit.

- `.env` and `.env.*` stay ignored.
- API keys come from environment variables or user config.
- Test fixtures must use fake tokens and fake credentials.
- Logs and snapshots must redact secrets.

## Commit Format

Use:

```text
<type>: <description>
```

Allowed types:

- `feat`
- `fix`
- `refactor`
- `docs`
- `chore`
- `test`

Examples:

```text
docs: rewrite architecture contract
test: cover fail-to-ask policy
feat: add npm release resolver
```

## Rust Rules

- No `unwrap()` outside tests.
- Prefer explicit error handling with `?` or typed errors.
- Panic paths undermine fail-to-ask.
- Keep dependencies scarce and auditable.
- Prefer current-thread execution unless a concrete implementation need
  justifies concurrency.
- Avoid background services and long-lived daemons in the MVP.

## Testing Rules

Core behavior needs focused tests:

- verdict parsing
- exit-code mapping
- policy decision table
- fail-to-ask scenarios
- adapter registry lookup
- real command execution behavior

Prompt text and user-facing verdict text are product behavior. Snapshot
them when practical.

Networked tests should be opt-in. Unit tests should not require live
registries, real package managers, or real LLM providers.

## Development Commands

```bash
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt
```

For direct local-provider runs, `PACKVET_PRINT_REVIEW_PROMPT=1` prints the
review prompt to stderr before invoking the provider. Provider reviews are
logged by default under `~/.packvet/reviews/reviews.jsonl`; set
`PACKVET_REVIEW_LOG_DIR` in tests to keep logs inside a temporary directory.

Before finishing implementation work, run at least:

```bash
cargo fmt --check
cargo test
```

If a Rust project or dependency graph does not exist yet, say which
verification commands could not be run and why.
