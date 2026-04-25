# Goal

lfg is a local pre-install guard for package managers. It stands in front
of normal install commands, reviews risky new package releases, and only
then allows the real package manager to run.

## Product Bet

Many supply-chain attacks arrive as a new package version. For a short
period after release, public reputation signals may be weak or absent.
CVE feeds, advisories, package reputation, and community reports often
arrive later.

lfg focuses on that early-release window by reviewing source diffs before
install. The exact threshold, diff baseline, verdicts, and fallback
behavior are policy decisions owned by `policy.md`.

## Non-Goals

lfg is not:

- a package reputation database
- a CVE feed replacement
- a hosted review proxy
- a malware sandbox
- a guarantee that old package versions are safe
- a tool that trusts package-controlled lifecycle scripts as its own
  execution point

Package lifecycle scripts are review evidence. They are not where lfg
should be installed as a trusted guard.
