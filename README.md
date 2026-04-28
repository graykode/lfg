# packvet

packvet is a local pre-install guard for package managers.

It runs before package manager install commands, reviews risky new package
releases, and only then lets the real package manager run.

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
packvet pip install -r requirements.txt
packvet uv add requests
packvet cargo add serde
```

Install PATH shims so normal package manager commands pass through packvet first:

```bash
packvet shim install --dir ~/.local/bin npm
packvet shim install --dir ~/.local/bin pnpm
packvet shim install --dir ~/.local/bin yarn
packvet shim install --dir ~/.local/bin pip
packvet shim install --dir ~/.local/bin uv
packvet shim install --dir ~/.local/bin cargo
packvet shim install --dir ~/.local/bin gem
```

Make sure the shim directory appears before the real package managers on
`PATH`.

When a provider review runs, packvet writes the prompt, provider output,
parsed verdict, reason, and evidence to `~/.packvet/reviews/reviews.jsonl`.
Provider `pass` verdicts let the real package manager run; `ask` verdicts
pause for local confirmation, and `block` verdicts stop the install.

Use `PACKVET_BYPASS=1` only as an emergency bypass:

```bash
PACKVET_BYPASS=1 npm install left-pad
```

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

MIT
