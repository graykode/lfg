# packvet

packvet reviews package releases before you install them.

Run install commands through packvet, such as `packvet npm install left-pad`,
or use `packvet review ...` when you only want the verdict. For guarded
installs, packvet resolves the target release, reviews risky fresh releases,
and then runs the real package manager, asks for confirmation, or blocks.

## Works with

packvet currently supports local review through:

- Claude Code CLI (`claude`)
- Codex CLI (`codex`)

Direct API-key review providers are not wired yet.

## Install

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/graykode/packvet/releases/latest/download/packvet-installer.sh | sh
```

### Cargo

```bash
cargo install packvet
```

Pre-built binaries are published on the
[GitHub Releases](https://github.com/graykode/packvet/releases) page.

## Usage

Run packvet explicitly:

```bash
packvet npm install left-pad
packvet pip install requests
packvet uv add requests
packvet cargo add serde
```

Review a package manager install request without executing the real package
manager:

```bash
packvet review npm install left-pad
packvet review cargo add serde
```

When a provider review runs, packvet writes the prompt, provider output,
parsed verdict, reason, and evidence to `~/.packvet/reviews/reviews.jsonl`.
Provider `pass` verdicts print a short review summary and then let the real
package manager run; `ask` verdicts pause for local confirmation, and `block`
verdicts stop the install.

Color is enabled automatically on interactive terminals. Set
`PACKVET_COLOR=never` to disable it, `PACKVET_COLOR=always` to force it, or
`NO_COLOR=1` to disable color for tools that honor that convention.

## Policy

packvet focuses on the early window after a package release is published, when
public reputation signals may not exist yet. If packvet cannot safely complete a
required review, it pauses the install instead of silently passing.

See [`doc/`](doc/) for product, policy, architecture, adapter, and development
details.

## Release

Release automation is prepared for:

- GitHub Releases with `cargo-dist`
- shell installer artifacts
- Homebrew formula publishing to `graykode/homebrew-tap`
- crates.io publishing

Before pushing a release tag, configure these repository secrets:

- `CARGO_REGISTRY_TOKEN`
- `HOMEBREW_TAP_TOKEN`

Then tag a version that matches `Cargo.toml`:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## License

MIT. See [LICENSE](LICENSE).
