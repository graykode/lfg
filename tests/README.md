# Test Layout

Keep tests at the highest useful layer. Prefer tests that protect product
behavior over tests that only restate implementation structure.

## Primary Test Layers

- `contracts/cli`: end-to-end CLI and shim behavior. These tests protect the
  user-facing install guard contract.
- `managers`: package manager argument parsing and real command construction.
  Put option-safety tests here.
- `ecosystems`: registry metadata parsing and HTTP request contracts for npm,
  PyPI, crates.io, and RubyGems.
- `evidence`: archive fetching, source extraction, diffs, and review evidence.
- `providers`: local review provider execution and provider output parsing.
- `core`: shared policy, verdict, outcome, and install assessment behavior.
- `wiring`: built-in registry smoke tests for managers, resolvers, policies,
  and providers.

## Avoid

- Duplicating manager or ecosystem behavior in lower-level contract tests.
- Tests that only prove public modules can be imported.
- Separate tests per ecosystem when a shared core test already covers the
  behavior and wiring only needs to prove the ecosystem is registered.
