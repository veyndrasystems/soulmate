# Soulmate

It verifies what you asked an agent to do and what came back, in the same
record.

[![Rust primary CI](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml/badge.svg)](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/veyndrasystems/soulmate)](https://github.com/veyndrasystems/soulmate/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Soulmate is a provider-free local ledger that sits beside the coding agent you
already use. The host executes and owns permissions; Soulmate records the
bounded assignment, artifact hashes, and transitions it can validate.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v0.10.0/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
```

The release is a single Linux x86_64 binary and also runs inside Ubuntu on WSL
2. See the [Windows guide](docs/windows-wsl.md) for the supported WSL layout.

## Start with four commands

Run these in the project you want to govern:

```sh
soulmate init --mode portable
soulmate brief worker --task "Describe the change" --config soulmate.json
soulmate run start change --goal "Describe the bounded change" --ledger .soulmate/runs/run.jsonl --config soulmate.json
soulmate check --config soulmate.json
```

`init` creates the local contract, `brief` renders an assignment, `run` records
its resumable handoff, and `check` validates the configuration. Advanced
lifecycle, recovery, hooks, migration, receipts, and the optional away runner
stay under `soulmate help advanced`.

## See one refusal

From a cloned checkout, run the token-free sample with the installed binary:

```sh
./scripts/demo-refusal.sh
```

The script creates a disposable one-file project, records the file, changes one
line, and asks for the next run transition:

```text
soulmate: artifact drift detected: result.txt
```

No model is called, no user project is touched, and the sample is removed. CI
runs the same script. A transcript can say "done"; Soulmate writes no new run
event unless the recorded artifact bytes still match disk.

That refusal proves only artifact-drift detection. Current releases record
declared write scope but do not inspect every product change against it. The
exact authority and trust contract is defined in [SECURITY.md](SECURITY.md).

## Keep your agent host

[The environment guide](docs/onboarding.md) covers Codex, Claude Code, Cursor,
GitHub Copilot, OpenCode, Gemini CLI, Cline, CI, and WSL without claiming that
path discovery proves host execution.

Portable initialization writes the Soulmate skill to `.agents/skills/` and a
Claude projection to `.claude/skills/`. Those are documented discovery paths
for those hosts. Host-specific caveats and the optional
[dotagents](https://github.com/getsentry/dotagents) distribution path stay in
the environment guide.

Choose portable mode when the project should carry reviewed profiles and local
mode when product source must stay untouched.

## Complete the first handoff

`run start` creates the ledger and its first lead assignment. Ask for the exact
pending assignment instead of reconstructing it from prose:

```sh
soulmate run next .soulmate/runs/run.jsonl --json --config soulmate.json
```

The native host performs that assignment and writes its result at the returned
`artifactPathHint`. Submit the named agent, role-appropriate outcome, and exact
artifact:

```sh
soulmate run submit AGENT .soulmate/runs/run.jsonl \
  --outcome OUTCOME \
  --artifact ARTIFACT_PATH \
  --artifact-root state \
  --config soulmate.json
```

The starter `change` workflow advances through these outcomes:

```text
lead scoped
worker completed
reviewer approved | rework
lead accepted | rejected
```

Repeat `run next` and `run submit` for each stage. Every attempt gets a fresh
artifact path, so rework does not overwrite earlier evidence. Only the lead can
record the final `accepted` outcome.

Each assignment binds the selected profile, configuration, declared scope,
memory references, and upstream artifact hashes. `run inspect` checks the
recorded chain; it does not replace a fresh project check.

## When a run refuses

Do not append around drift. Restore the exact recorded bytes when the change was
accidental. When the new state is intentional, keep the predecessor sealed and
start one provenance-bound successor:

```sh
soulmate run supersede .soulmate/runs/run.jsonl \
  --workflow change \
  --goal "Restate the bounded goal" \
  --ledger .soulmate/runs/resume.jsonl \
  --config soulmate.json
```

An `accepted` or `rejected` predecessor stays final. Detailed recovery and lock
behavior live in [REFERENCE.md](REFERENCE.md#run-and-recovery).

## Directory responsibilities

Portable mode keeps reviewable control material separate from private runtime
evidence and host discovery copies:

```text
PROJECT/
├── soulmate.json
├── soulmate/                 # profiles and declared boundaries
├── .soulmate/                # private run and memory evidence
├── .agents/skills/soulmate/  # Codex discovery copy
└── .claude/skills/soulmate/  # Claude discovery copy
```

Product source and host configuration remain outside these responsibilities.
Local mode can put control and state outside the product checkout entirely.

## Build and verify

Soulmate pins Rust 1.75 as its minimum toolchain. The same checks run before a
release:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
SOULMATE_BIN=target/debug/soulmate ./scripts/demo-refusal.sh
```

[CI](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml) tests
the project on Linux, macOS, and Ubuntu under WSL 2, and audits dependencies.
Persisted-format compatibility and release criteria are documented in
[REFERENCE.md](REFERENCE.md#versioning-and-release-contract).

## Detailed reference

- [Commands, workflows, recovery, and versioning](REFERENCE.md)
- [Onboarding by environment](docs/onboarding.md)
- [Control-plane boundary](docs/agent-control-plane.md)
- [Directory layout rationale](docs/soulmate-centered-layout.md)
- [Optional Codex away runner](docs/codex-tmux-away.md)
