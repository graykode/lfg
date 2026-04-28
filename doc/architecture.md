# Architecture

packvet is a local install-command reviewer. It runs when the user invokes a
package manager command through packvet, such as `packvet npm install <pkg>`,
or when the user asks for a review-only verdict with `packvet review ...`.

Package-controlled lifecycle scripts such as `package.json` `preinstall`,
`setup.py`, or `build.rs` are not trusted integration points. They are
review evidence.

## System Flow

```text
CLI Entrypoint
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
  ecosystems/rubygems/
                     RubyGems metadata and Ruby release policy glue
  evidence/          archive reading, archive fetching, source diff creation
  managers/bun/      Bun CLI parsing
  managers/cargo/    cargo CLI parsing
  managers/gem/      gem CLI parsing
  managers/npm/      npm CLI parsing
  managers/package_json.rs
                     package.json dependency extraction shared by JS managers
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

## CLI Entrypoint

The explicit CLI wrapper is the normal product entrypoint.

Responsibilities:

- identify the requested manager from explicit packvet arguments
- pass the original arguments to the matching manager integration adapter
- preserve stdin, stdout, stderr, environment, and exit behavior where safe
- run the real package manager only after the install gate allows it
- honor `PACKVET_BYPASS=1` as an emergency path that skips review and runs the
  real package manager directly

Native trusted hooks or package-manager plugins can be added later when an
ecosystem offers a hook that runs before package code. They are secondary
integrations, not the MVP integration path.

## Install Request Parser

The parser turns manager-specific commands into normalized install plans.

Examples:

- `bun add left-pad`
- `npm i left-pad`
- `npm install left-pad@1.3.0`
- `pnpm install`
- `pip install requests==2.32.3`
- `uv pip install -r requirements.txt`
- `uv add requests`

An install plan can contain multiple package requests. For example,
requirements files and workspace commands may expand to many packages.
If packvet cannot confidently understand the install target, it returns `ask`
instead of silently passing.

For JavaScript package managers, bare install commands may read dependencies
from `package.json`. The parser only extracts registry dependencies; local,
workspace, and remote URL dependencies are skipped or returned as `ask` when
they cannot be reviewed through registry metadata. Lockfile-aware exact version
resolution is a future resolver concern, not manager adapter logic.

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
CLI Entrypoint
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
