#!/bin/sh
set -eu

fail() {
  printf '%s\n' "$*" >&2
  exit 1
}

equal_version() {
  test -n "$2" && test "$2" = "$3" ||
    fail "$1 version is '$2'; expected '$3'"
}

for source in Cargo.toml Cargo.lock README.md REFERENCE.md CHANGELOG.md install.sh \
  plugin.json systems.veyndra.soulmate/.codex-plugin/plugin.json \
  systems.veyndra.soulmate/.claude-plugin/plugin.json scripts/ci-wsl.sh \
  docs examples schema scripts src
do
  test -r "$source" || fail "missing or unreadable release source: $source"
done

plain=$(awk '
  /^\[package\]$/ { package = 1; next }
  /^\[/ { package = 0 }
  package && /^version = "/ { sub(/^version = "/, ""); sub(/"$/, ""); print }
' Cargo.toml)
test "$(printf '%s\n' "$plain" | wc -l | tr -d ' ')" = 1 ||
  fail 'Cargo.toml must contain one package version'
printf '%s\n' "$plain" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?([+][0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?$' ||
  fail 'Cargo.toml package version is missing or invalid'
current="v$plain"

lock_version=$(awk '
  /^\[\[package\]\]$/ { soulmate = 0 }
  /^name = "soulmate"$/ { soulmate = 1 }
  soulmate && /^version = "/ { sub(/^version = "/, ""); sub(/"$/, ""); print }
' Cargo.lock)
equal_version Cargo.lock "$lock_version" "$plain"

grep -Fxq "curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/$current/install.sh | sh" README.md ||
  fail "README.md is missing the current $current install command"
installer_version=$(sed -n 's/^version="${SOULMATE_VERSION:-\([^}]*\)}"$/\1/p' install.sh)
equal_version install.sh "$installer_version" "$current"
changelog_version=$(awk '/^## [0-9]/ { print $2; exit }' CHANGELOG.md)
equal_version CHANGELOG.md "$changelog_version" "$plain"

for manifest in \
  plugin.json \
  systems.veyndra.soulmate/.codex-plugin/plugin.json \
  systems.veyndra.soulmate/.claude-plugin/plugin.json
do
  version=$(sed -n 's/^[[:space:]]*"version": "\([0-9][^"]*\)",*$/\1/p' "$manifest")
  equal_version "$manifest" "$version" "$plain"
done

wsl_version=$(sed -n 's/^test "$(soulmate version)" = "\([0-9][^"]*\)"$/\1/p' scripts/ci-wsl.sh)
equal_version scripts/ci-wsl.sh "$wsl_version" "$plain"

# Historical CHANGELOG entries are deliberately outside the current-reference scan.
if refs=$(grep -RInE --exclude-dir=.git 'v[0-9]+\.[0-9]+\.[0-9]+' README.md REFERENCE.md install.sh docs examples schema scripts src); then
  :
else
  status=$?
  test "$status" = 1 || fail 'release-reference scan failed'
fi
printf '%s\n' "$refs" | awk -v current="$current" '
  {
    line = $0
    while (match(line, /v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?([+][0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?/)) {
      version = substr(line, RSTART, RLENGTH)
      if (version != current) {
        print "stale release reference (expected " current "): " $0
        failed = 1
        break
      }
      line = substr(line, RSTART + RLENGTH)
    }
  }
  END { exit failed }
' >&2
