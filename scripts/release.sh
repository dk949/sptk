#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ] || [ -z "${1:-}" ]; then
    echo "usage: $0 <level> [message]" >&2
    echo "  level:   patch | minor | major | release | rc | beta | alpha | <x.y.z>" >&2
    echo "  message: omit for a dry run; supply to execute and tag" >&2
    exit 1
fi

level=$1
message=${2:-}

repo_root=$(git rev-parse --show-toplevel)
cd "$repo_root"

if [ -z "$message" ]; then
    exec cargo release "$level"
fi

cargo release "$level" --execute

version=$(awk -F'"' '/^version *=/ {print $2; exit}' Cargo.toml)
tag="v$version"

git tag -s -a "$tag" -m "$message"

echo
echo "Created signed tag $tag."
echo "Push with: git push --follow-tags origin trunk"
