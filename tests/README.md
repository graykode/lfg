# Integration Test Layout

Keep `tests/` for integration tests that execute the user-facing CLI from the
compiled binary. Unit and component tests should live under `src/` next to the
code they exercise.

## Belongs Here

- CLI behavior such as `packvet npm install ...`.
- End-to-end manager execution after policy/provider pass decisions.
- End-to-end fail-to-ask behavior when install review cannot proceed.

## Belongs Under `src/`

- Manager argument parsing.
- Registry metadata parsing and HTTP request contracts.
- Archive extraction, source diffing, and prompt construction.
- Provider output parsing and local provider execution.
- Built-in wiring smoke tests.
