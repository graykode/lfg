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

## Sandboxed Smoke Tests

Use Docker for smoke tests that may execute a real package manager. The
container is disposable, disables npm lifecycle scripts, and runs the package
manager from a temporary project directory. The sandbox image builds the local
`packvet` binary into `/usr/local/bin/packvet`; later runs reuse Docker build
cache unless source files changed.

By default, the sandbox prints the exact prompt that would be sent to a local
review provider and blocks before the real install command runs:

```bash
scripts/sandbox-install.sh npm left-pad
scripts/sandbox-install.sh pnpm left-pad
scripts/sandbox-install.sh yarn left-pad
```

To allow the real package manager install after packvet pauses, opt in
explicitly:

```bash
scripts/sandbox-install.sh --allow-install npm left-pad
```

Force a clean sandbox image rebuild with:

```bash
scripts/sandbox-install.sh --rebuild npm left-pad
```

For direct local-provider runs, `PACKVET_PRINT_REVIEW_PROMPT=1` prints the
review prompt to stderr before invoking the provider. Provider reviews are
logged by default under `~/.packvet/reviews/reviews.jsonl`; set
`PACKVET_REVIEW_LOG_DIR` in tests to keep logs inside a temporary directory.

This script is opt-in because it requires Docker and live network access.

Before finishing implementation work, run at least:

```bash
cargo fmt --check
cargo test
```

If a Rust project or dependency graph does not exist yet, say which
verification commands could not be run and why.
