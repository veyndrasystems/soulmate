#!/bin/sh
set -eu

expected=$(grep -v '^[[:space:]]*#' scripts/path-rendering-allowlist.txt |
  sed '/^[[:space:]]*$/d' |
  LC_ALL=C sort)
actual=$(git grep -n -E -e 'to_string_lossy|to_str\(\)\.unwrap_or|unwrap_or\("\."\)|PathBuf::from\("\."\)' -- 'src/*.rs' |
  sed -E 's/^([^:]+):[0-9]+:[[:space:]]*/\1:/' |
  LC_ALL=C sort)

if [ "$actual" != "$expected" ]; then
  echo "path-rendering allowlist mismatch" >&2
  echo "expected:" >&2
  printf '%s\n' "$expected" >&2
  echo "actual:" >&2
  printf '%s\n' "$actual" >&2
  exit 1
fi
