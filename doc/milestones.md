# Milestones

Milestones are implementation checkpoints. Each one should leave the repo
in a working, testable state.

The product goal is transparent pre-install guarding through command
shims. The early implementation can use explicit wrapper commands because
they are easier to test and debug.

## Milestone 0: Core Contract

Goal: lock the shared contracts before integrating real package managers
or real LLM providers.

Scope:

- Rust binary named `lfg`.
- Verdict and exit-code model from `policy.md`.
- Review policy decision table from `policy.md`.
- Structured LLM output parser.
- Adapter registries for manager integration adapters, ecosystem
  resolvers, and LLM adapters.
- Unit tests for verdict parsing, exit codes, policy decisions, registry
  lookup, and fail-to-ask behavior.

Exit criteria:

- `cargo fmt --check` passes.
- `cargo test` passes.
- Required review failures map to `ask` according to `policy.md`.

## Milestone 1: Explicit Wrapper Vertical Slice

Goal: prove the full review loop with one manager and one local LLM
adapter before installing command shims.

Scope:

- Explicit wrapper command for npm, such as `lfg npm install <package>`.
- npm manager integration adapter.
- npm registry release resolver.
- Archive diff using the baseline defined in `policy.md`.
- Published-age threshold policy from `policy.md`.
- Prompt builder and structured verdict parser.
- One local CLI LLM adapter, preferring Claude CLI or Codex CLI based on
  local availability.
- Real npm execution only after the install gate allows it.

Exit criteria:

- npm requests exercise the core policy cases from `policy.md`.
- The npm adapter does not duplicate diff, policy, provider, or verdict
  logic.

## Milestone 2: Command Shim Install

Goal: make normal user commands pass through lfg before the real package
manager runs.

Scope:

- Provide reversible shell aliases or functions as convenience setup.
- Implement PATH shim support as the durable transparent guard.
- Detect the manager from `argv[0]`.
- Locate the real package manager binary without recursive shim calls.
- Preserve original arguments, environment, stdin, stdout, and stderr
  where safe.
- Support `npm install` and `npm i` through the shim path.
- Provide a clear opt-out or bypass path for emergency use.

Exit criteria:

- Typing `npm i <package>` can invoke lfg first through the configured
  interception path.
- The real npm binary only runs after pass or explicit user confirmation.
- The docs distinguish convenience aliases/functions from PATH shims.
- Recursion protection is tested.
- Failure to locate the real package manager returns `ask` or a clear CLI
  misuse error, never silent pass.

## Milestone 3: Python Ecosystem

Goal: cover common Python install flows.

Scope:

- pip manager integration adapter.
- uv manager integration adapter.
- PyPI release resolver shared by pip and uv.
- Initial support for `pip install <package>`, `pip install -r
  requirements.txt`, and `uv add <package>`.
- Multi-package decision aggregation.

Exit criteria:

- Requirements files produce reviewable install requests when pinned or
  resolvable.
- Unclear requirement resolution follows `policy.md`.
- pip and uv share resolver logic.
- Manager adapters stay thin.

## Milestone 4: Adapter Protocol

Goal: make the architecture ready for external adapters without moving
core logic into plugins.

Scope:

- Document external executable adapter protocol.
- JSON request and response contracts.
- Protocol version handshake.
- Capability discovery.
- Error mapping to `ask`.

Exit criteria:

- Built-in adapters and external executable adapters can be described by
  the same logical contract.
- Dynamic library loading is not required.
- lfg still ships as a single binary.

## Milestone 5: More Managers

Goal: add more package managers through focused adapters.

Targets:

| Manager | Ecosystem resolver | Primary integration |
|---|---|---|
| `pnpm` | npm registry | command shim |
| `yarn` | npm registry | command shim |
| `cargo` | crates.io | command shim |
| `gem` | rubygems.org | command shim |

Exit criteria:

- Each adapter has focused parser and command execution tests.
- Resolver logic is shared when ecosystems overlap.
- Core review policy, diff, prompt, provider, and verdict code remain
  manager-agnostic.
