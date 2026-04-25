# Architecture

lfg is a local pre-install gate. It runs before the real package manager
executes an install command.

The primary integration path is command interception. The initial version
can install shell aliases or functions such as `npm -> lfg npm` for
convenience. The durable transparent guard is a PATH shim, because it works
outside interactive shell alias expansion and is harder to bypass
accidentally.

In the strongest form, a shim named `npm`, `pip`, `uv`, or another manager
name is placed before the real binary on `PATH`. The shim calls lfg, lfg
reviews the requested install, and only then does lfg execute the real
package manager.

Package-controlled lifecycle scripts such as `package.json` `preinstall`,
`setup.py`, or `build.rs` are not trusted integration points. They are
review evidence.

## System Flow

```text
Command Shim
  -> Install Request Parser
  -> Manager Integration Adapter
  -> Ecosystem Release Resolver
  -> Review Policy
  -> Archive Fetcher
  -> Diff Engine
  -> Prompt Builder
  -> LLM Adapter Registry
  -> Verdict Parser
  -> Install Gate
  -> Real Package Manager
```

## Core Boundary

The core does not know npm, pip, uv, Claude, Codex, or any registry API.
It knows contracts:

- an install command can become one or more install requests
- a release resolver can resolve target and previous published releases
- a review policy can decide `review`, `skip`, or `ask`
- a diff engine can compare two source archives
- an LLM adapter can review a prompt and return raw output
- a verdict parser can convert raw output into `pass`, `ask`, or `block`
- an install gate can decide whether the real manager is executed

## Install Assessment Flow

The install assessment flow separates package assessment into three roles:

- resolver: turns an install target into resolved package releases, including
  the target release, previous release, publish metadata, and archive URLs
- decision evaluator: applies policy to resolved release metadata and decides
  whether to skip, ask, or perform a review
- reviewer: gathers review evidence for releases that require review, asks the
  selected provider when available, and returns the package outcome

## Code Layout

The Rust code is grouped by role:

```text
src/
  core/              shared contracts, policy, verdicts, outcomes, assessment flow
  ecosystems/crates_io/
                     crates.io registry metadata and Rust release policy glue
  ecosystems/npm/    npm registry metadata and npm release policy glue
  ecosystems/pypi/   PyPI registry metadata and Python release policy glue
  evidence/          archive reading, archive fetching, source diff creation
  managers/cargo/    cargo CLI parsing
  managers/npm/      npm CLI parsing
  managers/pip/      pip CLI parsing
  managers/pnpm/     pnpm CLI parsing
  managers/uv/       uv CLI parsing
  managers/yarn/     Yarn CLI parsing
  providers/         provider output parsing and future provider adapters
  builtins.rs        built-in adapter registry wiring
  cli.rs             CLI entrypoint behavior
```

Start in `core/install_assessment.rs` for the manager-neutral assessment flow.
Look in `managers/` for package-manager CLI parsing, and in `ecosystems/`
for registry-specific metadata resolution shared by managers.

## Command Shim

The command shim is the normal product entrypoint.

Responsibilities:

- identify the requested manager from `argv[0]` or explicit lfg arguments
- pass the original arguments to the matching manager integration adapter
- avoid recursion when finding the real manager binary
- preserve stdin, stdout, stderr, environment, and exit behavior where safe
- run the real package manager only after the install gate allows it
- honor `LFG_BYPASS=1` as an emergency path that skips review and runs the
  real package manager directly

Shell aliases or functions are acceptable as convenience setup because they
are simple, reversible, and familiar. They are not a complete security
boundary.

PATH shim setup is explicit and reversible. `lfg shim install --dir <dir>
npm` creates an `npm` shim in the chosen directory, and `lfg shim uninstall
--dir <dir> npm` removes only shims that point back to the current lfg
executable.

Native trusted hooks or package-manager plugins can be added when an
ecosystem offers a hook that runs before package code. They are secondary
integrations. PATH shims remain the portable stronger baseline.

## Install Request Parser

The parser turns manager-specific commands into normalized install plans.

Examples:

- `npm i left-pad`
- `npm install left-pad@1.3.0`
- `pip install requests==2.32.3`
- `uv add requests`

An install plan can contain multiple package requests. For example,
requirements files and workspace commands may expand to many packages.
If lfg cannot confidently understand the install target, it returns `ask`
instead of silently passing.

## Manager Integration Adapter

A manager integration adapter owns CLI semantics for one package manager.
It answers:

- is this command an install command?
- which packages or requirement files are being installed?
- what real command should run after approval?
- which ecosystem resolver should handle the package metadata?

It does not perform LLM review, parse verdicts, or implement registry
metadata rules.

## Ecosystem Release Resolver

An ecosystem resolver owns registry metadata and source archive lookup.
Several managers can share one resolver.

The resolver returns the target release, the previous published release,
publish timestamps, and archive references. `policy.md` defines how the
previous published release is selected.

## Review Policy

The review policy decides whether a request should be reviewed, skipped, or
returned as `ask`. `policy.md` is the source of truth for threshold values,
diff baseline rules, provider selection, verdict behavior, and exit codes.

## Diff Engine

The diff engine compares the previous published source archive with the
target source archive. It produces a normalized source diff, file-level
metadata, and a truncation report.

The diff engine does not decide install behavior. `policy.md` owns the
default diff-priority order used when review input must be reduced.

## LLM Provider Registry

LLM review uses an adapter registry. Built-in adapters and future external
adapters must go through the same contract.

Provider implementations return raw model output and execution metadata.
They do not decide final install behavior or provider preference.

## Verdict Parser

The parser converts provider output into `pass`, `ask`, or `block`.
`policy.md` owns the provider output contract and malformed-output
fallback behavior.

## Install Gate

The install gate aggregates package decisions.

| Decision set | Result |
|---|---|
| any `block` | stop with `block` |
| any `ask` | ask the user in TTY mode, otherwise exit `ask` |
| all `pass` or skipped-pass | execute the real package manager |

When TTY confirmation is available, `ask` shows the review pause reason and
requires an explicit `y` or `yes` before executing the real package
manager. Empty input and any other answer keep the install paused.

The gate is the only component that may execute the real package manager.

## Dependency Direction

```text
Shim / CLI
  -> Core Orchestrator
      -> Manager Integration Registry
      -> Ecosystem Resolver Registry
      -> Review Policy
      -> Archive Fetcher
      -> Diff Engine
      -> Prompt Builder
      -> LLM Adapter Registry
      -> Verdict Parser
      -> Install Gate
```

Adapters depend on core contracts. Core orchestration does not depend on
adapter internals.
