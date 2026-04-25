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

The long-term extension model should prefer external executable adapters.

Rationale:

- lfg can still ship as a single binary.
- External adapters do not require dynamic library loading.
- JSON over stdin/stdout is auditable and testable.
- Adapter failures can be mapped cleanly to `ask`.

Future protocol pieces:

- protocol version handshake
- adapter identity and capabilities
- JSON request and response schema
- timeout handling
- stderr diagnostic policy
- secret redaction rules

Dynamic library loading should not be the default extension path.
