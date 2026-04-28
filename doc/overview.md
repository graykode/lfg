# Overview

packvet reviews package releases before you install them.

The user runs install commands through packvet, such as `packvet npm i`,
`packvet pip install -r requirements.txt`, or `packvet uv add`. packvet
reviews risky new package releases and only then lets the real package manager
run. The `packvet review ...` command runs the same review path without
executing the package manager.

## User Workflow

1. The user runs an install command through packvet.
2. packvet identifies the target package manager and command.
3. packvet parses the install request into one or more package targets.
4. packvet resolves registry metadata for each target.
5. packvet applies the review policy.
6. If review is required, packvet compares the previous published version with
   the target version and sends the diff to the selected LLM provider.
7. packvet parses the provider verdict.
8. The install gate either runs the real package manager, asks the user, or
   blocks the install.

## Default Policy

packvet focuses on the short window after a package version is published, when
public reputation signals may not exist yet.

By default, packvet reviews target versions published within the configured
age threshold. The default threshold is 24 hours. Older versions skip LLM
review and pass with a short note.

The review baseline is:

```text
previous published version -> target version
```

If packvet cannot safely complete a required review, it returns `ask` instead
of silently passing.

## Architecture

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

The core is adapter-based. Package manager syntax, registry metadata, and
LLM execution are separate concerns.

- Manager integration adapters understand commands such as `npm install`
  or `pip install`.
- Ecosystem release resolvers understand registries such as npm or PyPI.
- LLM adapters execute review providers. The current implementation supports
  local Claude CLI and local Codex CLI.

packvet has no hosted backend. Package diffs go from the user's machine to the
provider selected by local configuration and policy.

## Where Details Live

- `goal.md` explains why packvet exists and what it is not.
- `policy.md` defines default behavior, verdicts, exit codes, and
  fail-to-ask rules.
- `architecture.md` defines system boundaries and component flow.
- `adapters.md` defines adapter contracts.
- `milestones.md` defines implementation order.
- `development.md` defines repository rules and verification commands.
