# Adapters

lfg is adapter-based from the start.

The target architecture is plugin-compatible, but the first implementation
does not need a full external plugin runtime. Built-in adapters should use
the same contracts that future external adapters will use.

## Adapter Types

lfg has three adapter families:

- manager integration adapters
- ecosystem release resolvers
- LLM adapters

Keeping these separate avoids coupling CLI syntax, registry metadata, and
provider execution.

## Manager Integration Adapter

A manager integration adapter understands one package manager's command
line.

Responsibilities:

- identify install commands
- parse install targets
- preserve the real command to run after approval
- choose the ecosystem resolver
- describe commands it cannot safely understand

It must not:

- fetch registry metadata
- build source diffs
- call LLM providers
- parse final verdicts
- decide final install behavior

Conceptual shape:

```rust
trait ManagerIntegrationAdapter {
    fn id(&self) -> ManagerId;
    fn parse_install(&self, argv: &[OsString]) -> Result<InstallPlan, AdapterError>;
    fn real_command(&self, plan: &InstallPlan) -> RealCommand;
}
```

`InstallPlan` may contain more than one package request.

## Ecosystem Release Resolver

An ecosystem resolver understands one registry ecosystem.

Responsibilities:

- resolve the target version
- find the previous published version
- read publish timestamps
- return source archive references
- report missing or ambiguous metadata

Examples:

- npm, pnpm, and yarn can share an npm registry resolver.
- pip and uv can share a PyPI resolver.
- cargo can use a crates.io resolver.
- gem can use a RubyGems resolver.

Conceptual shape:

```rust
trait EcosystemReleaseResolver {
    fn id(&self) -> EcosystemId;
    fn resolve(&self, request: PackageRequest) -> Result<ResolvedRelease, ResolveError>;
}
```

`ResolvedRelease` should include the target release, previous release, and
archive references needed by the diff engine.

## LLM Adapter

An LLM adapter executes one review provider.

Responsibilities:

- detect whether the provider is available
- execute the provider with a prompt and timeout
- return raw provider output
- return provider metadata useful for diagnostics

It must not:

- decide whether review is needed
- parse final verdict semantics beyond provider transport concerns
- execute the real package manager
- decide provider preference

Conceptual shape:

```rust
trait LlmAdapter {
    fn id(&self) -> ProviderId;
    fn detect(&self) -> ProviderAvailability;
    fn review(&self, prompt: ReviewPrompt, timeout: Duration) -> Result<ProviderOutput, ProviderError>;
}
```

Built-in adapters should include Claude CLI and Codex CLI. `policy.md`
owns provider selection order.

## Registries

Each adapter family has a registry.

Registry responsibilities:

- register built-in adapters
- resolve configured adapter ids
- expose available adapters for diagnostics
- return typed errors for missing adapters

The core orchestrator asks registries for adapters. It must not directly
construct manager-specific or provider-specific implementations.

## Future External Adapter Protocol

The extension model prefers external executable adapters.

Rationale:

- lfg can still ship as a single binary.
- External adapters do not require dynamic library loading.
- JSON over stdin/stdout is auditable and testable.
- Adapter failures can be mapped cleanly to `ask`.

External adapters describe the same logical contracts as built-in adapters.
They do not own core install assessment, review policy, diff generation,
provider preference, verdict parsing, command shim behavior, or real package
manager execution.

The process model is one request per process invocation:

- lfg starts the adapter executable.
- lfg writes one JSON request to stdin.
- the adapter writes one JSON response to stdout.
- stderr is diagnostic text only and must not contain secrets.
- timeout, non-zero exit, malformed JSON, unsupported protocol versions, and
  explicit adapter errors map to `ask`.

Dynamic library loading, hosted services, background daemons, and package
manager lifecycle scripts are not the adapter protocol.

## External Adapter Protocol v1

All messages include:

- `type`: request or response type, using kebab-case
- `protocol_version`: integer protocol version, currently `1`

The first request should be a handshake:

```json
{
  "type": "handshake",
  "protocol_version": 1,
  "lfg_version": "0.1.0"
}
```

The adapter responds with its identity and capabilities:

```json
{
  "type": "handshake-accepted",
  "protocol_version": 1,
  "adapter_id": "example-adapter",
  "capabilities": [
    { "kind": "manager-integration", "id": "pnpm" },
    { "kind": "ecosystem-release-resolver", "id": "npm-registry" },
    { "kind": "llm-adapter", "id": "example-llm" }
  ]
}
```

Capability discovery can also be requested directly:

```json
{
  "type": "capabilities",
  "protocol_version": 1
}
```

Response:

```json
{
  "type": "capabilities",
  "protocol_version": 1,
  "capabilities": [
    { "kind": "manager-integration", "id": "pnpm" }
  ]
}
```

### Manager Integration Request

Manager adapters parse package-manager CLI semantics. They do not fetch
registry metadata or decide install behavior.

```json
{
  "type": "parse-install",
  "protocol_version": 1,
  "manager_id": "pnpm",
  "args": ["add", "left-pad"]
}
```

Successful response:

```json
{
  "type": "install-parsed",
  "protocol_version": 1,
  "request": {
    "manager_id": "pnpm",
    "operation": "add",
    "targets": [{ "spec": "left-pad" }],
    "manager_args": ["add", "left-pad"],
    "release_resolver_id": "npm-registry",
    "release_decision_evaluator_id": "npm-release-policy"
  },
  "real_command": {
    "program": "pnpm",
    "args": ["add", "left-pad"]
  }
}
```

### Ecosystem Resolver Request

Ecosystem resolvers return release metadata and source archive references.
They do not build diffs, call LLMs, or execute package managers.

```json
{
  "type": "resolve-release",
  "protocol_version": 1,
  "resolver_id": "npm-registry",
  "target": { "spec": "left-pad" }
}
```

Successful response:

```json
{
  "type": "release-resolved",
  "protocol_version": 1,
  "releases": {
    "package_name": "left-pad",
    "target": {
      "version": "1.1.0",
      "published_at": "1970-01-02T00:00:00.000Z",
      "archive": {
        "url": "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz"
      }
    },
    "previous": {
      "version": "1.0.0",
      "published_at": "1970-01-01T00:00:00.000Z",
      "archive": {
        "url": "https://registry.npmjs.org/left-pad/-/left-pad-1.0.0.tgz"
      }
    }
  }
}
```

### LLM Adapter Request

LLM adapters execute a provider and return raw provider output. lfg still
parses the final verdict according to `policy.md`.

```json
{
  "type": "review",
  "protocol_version": 1,
  "provider_id": "example-llm",
  "prompt": "review prompt text",
  "timeout_seconds": 60
}
```

Successful response:

```json
{
  "type": "review-completed",
  "protocol_version": 1,
  "raw_output": "verdict: pass\nreason: reviewed\n"
}
```

### Error Mapping

Adapters return explicit errors in this shape:

```json
{
  "type": "error",
  "protocol_version": 1,
  "code": "unsupported-command",
  "message": "pnpm command cannot be reviewed safely",
  "ask": true
}
```

Supported error codes:

- `unsupported-protocol-version`
- `invalid-request`
- `invalid-response`
- `unsupported-command`
- `unsupported-option`
- `unavailable`
- `timeout`
- `failed`

All adapter errors are install pauses. lfg must map them to `ask`, never
silent pass. If an adapter omits `ask`, returns `ask: false`, exits
non-zero, times out, writes malformed JSON, or writes a response with the
wrong `protocol_version`, lfg treats the adapter as failed and pauses the
install with `ask`.
