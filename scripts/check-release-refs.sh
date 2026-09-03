#!/bin/sh
set -eu

current="v$(sed -n 's/^version = "\([0-9][^"]*\)"$/\1/p' Cargo.toml | head -n 1)"
refs=$(grep -RInE --exclude-dir=.git 'v0\.[0-9]+\.[0-9]+' README.md REFERENCE.md install.sh docs examples schema scripts src 2>/dev/null || true)
stale=$(printf '%s\n' "$refs" | awk -v current="$current" '
  {
    line = $0
    while ((start = index(line, current)) > 0) {
      line = substr(line, 1, start - 1) substr(line, start + length(current))
    }
    if (line ~ /v0\.[0-9]+\.[0-9]+/) print
  }
' || true)
if [ -n "$stale" ]; then
    printf '%s\n' "stale release references (expected $current):" "$stale" >&2
    exit 1
fi

plain=${current#v}
for manifest in \
  plugin.json \
  systems.veyndra.soulmate/.codex-plugin/plugin.json \
  systems.veyndra.soulmate/.claude-plugin/plugin.json
do
  version=$(sed -n 's/^[[:space:]]*"version": "\([0-9][^"]*\)",*$/\1/p' "$manifest")
  if [ "$version" != "$plain" ]; then
    echo "$manifest version is '$version'; expected '$plain'" >&2
    exit 1
  fi
done

wsl_version=$(sed -n 's/^test "$(soulmate version)" = "\([0-9][^"]*\)"$/\1/p' scripts/ci-wsl.sh)
if [ "$wsl_version" != "$plain" ]; then
  echo "scripts/ci-wsl.sh version is '$wsl_version'; expected '$plain'" >&2
  exit 1
fi
