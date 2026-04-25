# lfg Agent Guide

This file is the entry point for Codex agents working in this repository.
Keep it short. Put durable product detail in `doc/`.

## Read First

1. [Overview](doc/overview.md)
2. [Goal](doc/goal.md)
3. [Policy](doc/policy.md)
4. [Architecture](doc/architecture.md)
5. [Adapters](doc/adapters.md)
6. [Milestones](doc/milestones.md)
7. [Development](doc/development.md)

## Current Target

Work on **Milestone 2: Command Shim Install**.

Keep normal package manager commands behind the same install gate proven
by the explicit wrapper. Start with shim detection, real-binary lookup,
and recursion protection before adding convenience setup commands.

## Non-Negotiables

- Follow `doc/development.md` for language, secret handling, commit format,
  dependency discipline, and verification commands.
- Keep document ownership clear:
  - `doc/overview.md` owns the first-read product and workflow summary.
  - `doc/goal.md` owns product intent.
  - `doc/architecture.md` owns system boundaries and flow.
  - `doc/policy.md` owns default behavior and verdict rules.
  - `doc/adapters.md` owns adapter contracts.
  - `doc/milestones.md` owns implementation order.
  - `doc/development.md` owns development rules.
- lfg is a local pre-install guard. It must run before the real package
  manager executes an install command.
- Package-controlled lifecycle scripts are review evidence, not trusted lfg
  integration points.
- Do not introduce a hosted lfg service, background daemon, or npm
  distribution path.
- Follow `doc/policy.md` for diff baseline, published-age threshold,
  provider selection, verdicts, exit codes, and fail-to-ask behavior.
- User-facing output should be warm, short, and concrete.

## Development Commands

Use the commands and verification requirements in `doc/development.md`.
