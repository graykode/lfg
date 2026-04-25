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
- command shim recursion protection

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

Before finishing implementation work, run at least:

```bash
cargo fmt --check
cargo test
```

If a Rust project or dependency graph does not exist yet, say which
verification commands could not be run and why.
