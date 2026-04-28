# Milestones

Milestones are implementation checkpoints. Each one should leave the repo
in a working, testable state.

The product goal is a local review gate that users invoke explicitly through
packvet install-wrapper commands or review-only commands.

## Milestone 0: Core Contract

Goal: lock the shared contracts before integrating real package managers
or real LLM providers.

Scope:

- Rust binary named `packvet`.
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
adapter.

Scope:

- Explicit wrapper command for npm, such as `packvet npm install <package>`.
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

## Milestone 2: Install Execution And Review-Only UX

Goal: make explicit packvet commands feel clear, observable, and safe.

Scope:

- Preserve original arguments, environment, stdin, stdout, and stderr where
  safe.
- Stream packvet-owned progress output so long review steps are visible.
- Support npm `install` and `i` aliases through explicit packvet invocation.
- Add review-only commands that run resolver, diff, and provider review without
  executing the real package manager.
- Provide a clear opt-out or bypass path for emergency use.

Exit criteria:

- Typing `packvet npm i <package>` invokes the install gate.
- Typing `packvet review npm install <package>` never executes npm.
- The real npm binary only runs after pass or explicit user confirmation.
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
- packvet still ships as a single binary.

## Milestone 5: More Managers

Goal: add more package managers through focused adapters.

Targets:

| Manager | Ecosystem resolver | Primary integration |
|---|---|---|
| `pnpm` | npm registry | explicit wrapper |
| `yarn` | npm registry | explicit wrapper |
| `cargo` | crates.io | explicit wrapper |
| `gem` | rubygems.org | explicit wrapper |

Exit criteria:

- Each adapter has focused parser and command execution tests.
- Resolver logic is shared when ecosystems overlap.
- Core review policy, diff, prompt, provider, and verdict code remain
  manager-agnostic.
