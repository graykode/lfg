# Policy

This document is the source of truth for lfg's default product behavior.
Other docs may summarize these rules, but should not redefine them.

## Published-Age Threshold

lfg reviews newly published target versions. The default threshold is 24
hours.

The threshold must be user-configurable. Configuration names can change
during implementation, but the policy concept should remain stable.

Default:

```toml
review_age_threshold = "24h"
```

Decision table:

| Condition | Decision |
|---|---|
| target publish time is missing | `ask` |
| target age is within threshold | run LLM diff review |
| target age is older than threshold | skip LLM review and pass with note |
| previous published version is missing | `ask` |

The skip note should be short and concrete, for example:

```text
lfg: skipped LLM review because this version is older than 24h.
```

## Diff Baseline

lfg compares:

```text
previous published version -> target version
```

The target version is the version selected by the package manager request
after normal version resolution.

The previous published version is the nearest earlier release for the same
package by registry publish time. It is not the locally installed version
and is not inferred from semver order alone. If registry timestamps are
missing or ambiguous, lfg returns `ask`.

This baseline keeps the review focused on what changed in the target
release. Local installs vary too much to be the default comparison point.

## First Release Behavior

When no previous published version exists, lfg returns `ask` by default.

Future configuration may allow a full source review for first releases,
but that is not the default. lfg is diff-first, and no baseline means the
main evidence is missing.

## Fail-To-Ask

When review is required, return `ask` for:

- missing provider
- provider failure
- provider timeout
- missing publish metadata
- missing previous published release
- archive fetch failure
- diff failure
- truncated or incomplete review input that cannot be disclosed
- malformed provider output
- ambiguous provider output
- unexpected runtime error after lfg has enough context to form a safe
  verdict

Never silently pass when a review was expected but did not complete.

## Provider Selection

An explicit user-configured provider is an override. If that provider is
unavailable and review is required, lfg returns `ask`.

Without an explicit provider override, lfg uses this default order:

1. local Claude CLI
2. local Codex CLI
3. other local model adapters
4. API provider adapters

The current explicit provider override is `LFG_REVIEW_PROVIDER`.

| Value | Behavior |
|---|---|
| unset or `auto` | use the default provider order |
| `claude` or `claude-cli` | use the local Claude CLI only |
| `codex` or `codex-cli` | use the local Codex CLI only |
| `none` | disable provider execution and return `ask` when review is required |

Provider configuration must not imply a hosted lfg backend. Diffs go from
the user's machine to the selected provider.

## Verdict Exit Codes

| Exit code | Meaning |
|---|---|
| `0` | `pass`: install may proceed. |
| `20` | `ask`: user confirmation is required. |
| `30` | `block`: install must not proceed. |
| `1` | CLI misuse or internal bug before a safe verdict exists. |

Prefer `20` over `1` when lfg can say the review did not complete safely.

## TTY And Non-Interactive Behavior

| Verdict | TTY behavior | Non-interactive behavior |
|---|---|---|
| `pass` | run the real package manager | run the real package manager |
| `ask` | prompt the user | exit with code `20` |
| `block` | stop without running the real package manager | exit with code `30` |

Wrappers, shims, and hooks must still see the same verdict contract.

## Provider Output Contract

Prompts must ask LLM adapters for structured text:

```text
verdict: pass|ask|block
reason: one short paragraph
evidence:
- file/path: concrete signal
```

The parser may accept minor formatting variation. It must return `ask`
for missing, ambiguous, conflicting, or malformed verdicts.

## Diff Priority

When the diff exceeds the provider context budget, keep the highest-signal
content first:

1. package metadata and entry points
2. package-controlled lifecycle scripts and install hooks
3. newly added files
4. executable files and permission changes
5. suspicious network, process, filesystem, obfuscation, or credential
   handling code
6. largest remaining hunks

The prompt and final user output must disclose truncation when it happens.
