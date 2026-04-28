#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE' >&2
Usage:
  scripts/sandbox-npm-install.sh --print-prompt <npm-install-arg>...
  scripts/sandbox-npm-install.sh <npm-install-arg>...

Examples:
  scripts/sandbox-npm-install.sh --print-prompt left-pad
  scripts/sandbox-npm-install.sh left-pad
  scripts/sandbox-npm-install.sh left-pad@1.3.0

Environment:
  PACKVET_SANDBOX_IMAGE                         Docker image tag to build/use.
  PACKVET_REVIEW_PROVIDER                       Defaults to none.
  PACKVET_REVIEW_AGE_THRESHOLD_SECONDS          Defaults to 9999999999.
  PACKVET_NPM_REGISTRY_URL                      Optional registry override.
USAGE
}

print_prompt=0
npm_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --print-prompt)
      print_prompt=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      npm_args+=("$@")
      break
      ;;
    *)
      npm_args+=("$1")
      shift
      ;;
  esac
done

if [[ ${#npm_args[@]} -eq 0 ]]; then
  usage
  exit 2
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "packvet: docker is required for sandboxed npm install smoke tests" >&2
  exit 1
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(git -C "$script_dir/.." rev-parse --show-toplevel)"
image="${PACKVET_SANDBOX_IMAGE:-packvet-npm-sandbox:latest}"
build_dir="$(mktemp -d "${TMPDIR:-/tmp}/packvet-sandbox-image.XXXXXX")"

cleanup() {
  rm -rf "$build_dir"
}
trap cleanup EXIT

cat >"$build_dir/Dockerfile" <<'DOCKERFILE'
FROM rust:1-bookworm
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates nodejs npm tar \
  && rm -rf /var/lib/apt/lists/*
ENV PATH="/usr/local/cargo/bin:${PATH}"
RUN command -v cargo >/dev/null
RUN useradd -m sandbox
USER sandbox
WORKDIR /home/sandbox
DOCKERFILE

docker build -q -t "$image" "$build_dir" >/dev/null

review_provider="${PACKVET_REVIEW_PROVIDER:-none}"
if [[ "$print_prompt" -eq 1 ]]; then
  review_provider="claude"
fi

docker_args=(
  run
  --rm
  --cap-drop=ALL
  --security-opt=no-new-privileges
  --pids-limit=256
  --memory=1g
  --cpus=1
  -v "$repo_root:/src:ro"
  -e "PACKVET_REVIEW_PROVIDER=$review_provider"
  -e "PACKVET_REVIEW_AGE_THRESHOLD_SECONDS=${PACKVET_REVIEW_AGE_THRESHOLD_SECONDS:-9999999999}"
  -e "RUST_BACKTRACE=${RUST_BACKTRACE:-1}"
  -e "NPM_CONFIG_IGNORE_SCRIPTS=true"
)

if [[ "$print_prompt" -eq 1 ]]; then
  docker_args+=(
    -e "PACKVET_PRINT_REVIEW_PROMPT=1"
  )
fi

if [[ -n "${PACKVET_NPM_REGISTRY_URL:-}" ]]; then
  docker_args+=(-e "PACKVET_NPM_REGISTRY_URL=$PACKVET_NPM_REGISTRY_URL")
fi

if [[ -t 0 && -t 1 ]]; then
  docker_args+=(-it)
fi

set +e
docker "${docker_args[@]}" "$image" bash -lc '
  set -euo pipefail
  export PATH="/usr/local/cargo/bin:$PATH"

  if [[ "${PACKVET_PRINT_REVIEW_PROMPT:-}" = "1" ]]; then
    mkdir -p /tmp/packvet-provider-bin
    cat >/tmp/packvet-provider-bin/claude <<'"'"'PROVIDER'"'"'
#!/bin/sh
cat >/dev/null
cat <<'"'"'OUTPUT'"'"'
verdict: block
reason: prompt printed before sandbox fake provider
evidence:
- sandbox: prompt print mode
OUTPUT
PROVIDER
    cp /tmp/packvet-provider-bin/claude /tmp/packvet-provider-bin/codex
    chmod +x /tmp/packvet-provider-bin/claude /tmp/packvet-provider-bin/codex
    export PATH="/tmp/packvet-provider-bin:$PATH"
  fi

  mkdir -p /home/sandbox/packvet
  tar -C /src --exclude=target --exclude=.git -cf - . \
    | tar -C /home/sandbox/packvet -xf -

  cd /home/sandbox/packvet
  command -v cargo >/dev/null
  cargo build

  mkdir -p /tmp/app
  cd /tmp/app
  npm init -y >/dev/null

  /home/sandbox/packvet/target/debug/packvet npm install "$@"
' packvet-sandbox "${npm_args[@]}"
docker_status=$?
set -e

exit "$docker_status"
