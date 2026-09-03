#!/bin/sh
set -eu

bin=${1:?usage: onboarding-smoke.sh BINARY}
case "$bin" in
  /*) ;;
  *) bin="$(cd "$(dirname "$bin")" && pwd)/$(basename "$bin")" ;;
esac
test -x "$bin"

root=$(mktemp -d)
cleanup() { find "$root" -depth -delete; }
trap cleanup EXIT HUP INT TERM
mkdir -p "$root/home" "$root/project" "$root/bindings"
config="$root/project/soulmate.json"
ledger=.soulmate/runs/onboarding.jsonl

invoke() {
  env -i HOME="$root/home" SOULMATE_BINDINGS_DIR="$root/bindings" \
    PATH=/nonexistent "$bin" "$@"
}

next=$(invoke init --mode portable --root "$root/project")
case "$next" in
  *"soulmate brief"*"soulmate run start"*"soulmate check"*) ;;
  *) echo "onboarding smoke: init did not print the core path" >&2; exit 1 ;;
esac

test -f "$root/project/.agents/skills/soulmate/SKILL.md"
test -f "$root/project/.claude/skills/soulmate/SKILL.md"
invoke brief worker --task "First bounded handoff" --config "$config" >/dev/null
invoke run start change --goal "First bounded handoff" \
  --ledger "$ledger" --config "$config" >/dev/null
invoke run inspect "$ledger" --config "$config" >/dev/null
invoke check --config "$config" >/dev/null

printf '%s\n' "onboarding smoke passed"
