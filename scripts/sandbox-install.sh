#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE' >&2
Usage:
  scripts/sandbox-install.sh [--allow-install] [--rebuild] <manager> <package-arg>...

Examples:
  scripts/sandbox-install.sh npm left-pad
  scripts/sandbox-install.sh pnpm left-pad
  scripts/sandbox-install.sh yarn left-pad
  scripts/sandbox-install.sh --allow-install npm left-pad
  scripts/sandbox-install.sh --rebuild npm left-pad

Managers:
  npm                         Runs packvet npm install <package-arg>...
  pnpm                        Runs packvet pnpm add <package-arg>...
  yarn                        Runs packvet yarn add <package-arg>...

Environment:
  PACKVET_SANDBOX_IMAGE                         Docker image tag to build/use.
  PACKVET_REVIEW_PROVIDER                       Defaults to none with --allow-install.
  PACKVET_REVIEW_AGE_THRESHOLD_SECONDS          Defaults to 9999999999.
  PACKVET_NPM_REGISTRY_URL                      Optional registry override.
USAGE
}

allow_install=0
rebuild=0
manager=""
manager_args=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --allow-install)
      allow_install=1
      shift
      ;;
    --rebuild)
      rebuild=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      break
      ;;
    -*)
      echo "packvet: unknown sandbox option: $1" >&2
      exit 2
      ;;
    *)
      manager="$1"
      shift
      manager_args=("$@")
      break
      ;;
  esac
done

if [[ -z "$manager" || ${#manager_args[@]} -eq 0 ]]; then
  usage
  exit 2
fi

case "$manager" in
  npm)
    manager_command=(npm install)
    ;;
  pnpm)
    manager_command=(pnpm add)
    ;;
  yarn)
    manager_command=(yarn add)
    ;;
  *)
    echo "packvet: unsupported sandbox manager: $manager" >&2
    exit 2
    ;;
esac

if ! command -v docker >/dev/null 2>&1; then
  echo "packvet: docker is required for sandboxed install smoke tests" >&2
  exit 1
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(git -C "$script_dir/.." rev-parse --show-toplevel)"
image="${PACKVET_SANDBOX_IMAGE:-packvet-install-sandbox:latest}"
build_dir="$(mktemp -d "${TMPDIR:-/tmp}/packvet-sandbox-image.XXXXXX")"

cleanup() {
  rm -rf "$build_dir"
}
trap cleanup EXIT

mkdir -p "$build_dir/src"
cp "$repo_root/Cargo.toml" "$repo_root/Cargo.lock" "$repo_root/README.md" "$build_dir/"
tar -C "$repo_root/src" -cf - . | tar -C "$build_dir/src" -xf -

cat >"$build_dir/Dockerfile" <<'DOCKERFILE'
FROM rust:1-bookworm
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates nodejs npm tar \
  && rm -rf /var/lib/apt/lists/*
RUN npm install -g pnpm yarn
ENV PATH="/usr/local/cargo/bin:${PATH}"
WORKDIR /opt/packvet
COPY Cargo.toml Cargo.lock README.md ./
RUN mkdir src \
  && printf 'pub fn placeholder() {}\n' > src/lib.rs \
  && printf 'fn main() {}\n' > src/main.rs \
  && cargo build \
  && cargo clean -p packvet \
  && rm -rf src
COPY src ./src
RUN cargo build \
  && cp target/debug/packvet /usr/local/bin/packvet
RUN command -v packvet >/dev/null
RUN useradd -m sandbox
USER sandbox
WORKDIR /home/sandbox
DOCKERFILE

docker_build_args=(build -q -t "$image")
if [[ "$rebuild" -eq 1 ]]; then
  docker_build_args+=(--no-cache)
fi

docker "${docker_build_args[@]}" "$build_dir" >/dev/null

review_provider="${PACKVET_REVIEW_PROVIDER:-none}"
if [[ "$allow_install" -eq 0 ]]; then
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
  -e "PACKVET_REVIEW_PROVIDER=$review_provider"
  -e "PACKVET_REVIEW_AGE_THRESHOLD_SECONDS=${PACKVET_REVIEW_AGE_THRESHOLD_SECONDS:-9999999999}"
  -e "RUST_BACKTRACE=${RUST_BACKTRACE:-1}"
  -e "NPM_CONFIG_IGNORE_SCRIPTS=true"
)

if [[ "$allow_install" -eq 0 || -n "${PACKVET_PRINT_REVIEW_PROMPT:-}" ]]; then
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

  mkdir -p /tmp/app
  cd /tmp/app
  npm init -y >/dev/null

  packvet "$@"
' packvet-sandbox "${manager_command[@]}" "${manager_args[@]}"
docker_status=$?
set -e

exit "$docker_status"
