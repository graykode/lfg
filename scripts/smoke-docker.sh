#!/usr/bin/env bash
set -euo pipefail

safe_packages=(
  "is-number@7.0.0"
)
default_package="is-number@7.0.0"

usage() {
  cat <<'USAGE'
Usage:
  scripts/smoke-docker.sh [--rebuild] [package]

Examples:
  scripts/smoke-docker.sh
  scripts/smoke-docker.sh is-number@7.0.0
  scripts/smoke-docker.sh --rebuild

Runs packvet inside a disposable Docker npm project. Docker smoke tests use a
fake local provider that returns pass, so this verifies install isolation and
packvet CLI UX without requiring Claude inside the container.

Safe package allowlist:
  is-number@7.0.0

Environment:
  PACKVET_SMOKE_DOCKER_IMAGE                    Docker image tag to build/use.
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

rebuild=0
package=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --rebuild)
      rebuild=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    -*)
      echo "packvet: unknown docker smoke option: $1" >&2
      exit 2
      ;;
    *)
      if [[ -n "$package" ]]; then
        usage >&2
        exit 2
      fi
      package="$1"
      shift
      ;;
  esac
done

package="${package:-$default_package}"
if ! is_safe_package "$package"; then
  print_allowlist_error "$package"
  exit 2
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "packvet: docker is required for docker smoke tests" >&2
  exit 1
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(git -C "$script_dir/.." rev-parse --show-toplevel)"
image="${PACKVET_SMOKE_DOCKER_IMAGE:-packvet-smoke:latest}"
build_dir="$(mktemp -d "${TMPDIR:-/tmp}/packvet-smoke-image.XXXXXX")"

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

docker run \
  --rm \
  --cap-drop=ALL \
  --security-opt=no-new-privileges \
  --pids-limit=256 \
  --memory=1g \
  --cpus=1 \
  -e "PACKVET_REVIEW_PROVIDER=claude" \
  -e "PACKVET_REVIEW_AGE_THRESHOLD_SECONDS=${PACKVET_REVIEW_AGE_THRESHOLD_SECONDS:-9999999999}" \
  -e "PACKVET_COLOR=${PACKVET_COLOR:-auto}" \
  -e "NPM_CONFIG_IGNORE_SCRIPTS=true" \
  "$image" bash -lc '
    set -euo pipefail

    mkdir -p /tmp/packvet-provider-bin
    cat >/tmp/packvet-provider-bin/claude <<'"'"'PROVIDER'"'"'
#!/bin/sh
cat >/dev/null
cat <<'"'"'OUTPUT'"'"'
verdict: pass
reason: docker smoke fake provider approved this allowlisted package
evidence:
- smoke: allowlisted package and lifecycle scripts disabled
OUTPUT
PROVIDER
    chmod +x /tmp/packvet-provider-bin/claude
    export PATH="/tmp/packvet-provider-bin:$PATH"

    mkdir -p /tmp/app
    cd /tmp/app
    npm init -y >/dev/null

    packvet npm install "$1"
    npm audit --omit=dev
  ' packvet-smoke "$package"
