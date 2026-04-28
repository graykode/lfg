#!/usr/bin/env bash
set -euo pipefail

safe_packages=(
  "is-number@7.0.0"
)
default_package="is-number@7.0.0"

usage() {
  cat <<'USAGE'
Usage:
  scripts/smoke-local.sh [package]

Examples:
  scripts/smoke-local.sh
  scripts/smoke-local.sh is-number@7.0.0

Runs a local packvet smoke test in a temporary npm project.

Safe package allowlist:
  is-number@7.0.0

Environment:
  PACKVET_REVIEW_PROVIDER                       Defaults to claude.
  PACKVET_REVIEW_AGE_THRESHOLD_SECONDS          Defaults to 9999999999.
  PACKVET_COLOR                                 Optional color override.
USAGE
}

is_safe_package() {
  local package="$1"
  local safe

  for safe in "${safe_packages[@]}"; do
    if [[ "$package" == "$safe" ]]; then
      return 0
    fi
  done

  return 1
}

print_allowlist_error() {
  local package="$1"

  echo "packvet: package is not in the smoke-test allowlist: $package" >&2
  echo "packvet: allowed packages: ${safe_packages[*]}" >&2
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

if [[ $# -gt 1 ]]; then
  usage >&2
  exit 2
fi

package="${1:-$default_package}"
if ! is_safe_package "$package"; then
  print_allowlist_error "$package"
  exit 2
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(git -C "$script_dir/.." rev-parse --show-toplevel)"
tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/packvet-smoke-local.XXXXXX")"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

cargo build --manifest-path "$repo_root/Cargo.toml" >/dev/null

cd "$tmp_dir"
npm init -y >/dev/null
unset PACKVET_PRINT_REVIEW_PROMPT

echo "packvet: local smoke project: $tmp_dir"
env \
  PACKVET_REVIEW_PROVIDER="${PACKVET_REVIEW_PROVIDER:-claude}" \
  PACKVET_REVIEW_AGE_THRESHOLD_SECONDS="${PACKVET_REVIEW_AGE_THRESHOLD_SECONDS:-9999999999}" \
  NPM_CONFIG_IGNORE_SCRIPTS=true \
  "$repo_root/target/debug/packvet" npm install "$package"

npm audit --omit=dev
