# Soulmate

It verifies what you asked an agent to do and what came back, in the same record.

Soulmate keeps a local record of agent work through review and rework. Inspect
the current attempt, read earlier submissions, and retrieve the next assignment
alongside the coding agent you already use.

[![Rust primary CI](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml/badge.svg)](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/veyndrasystems/soulmate)](https://github.com/veyndrasystems/soulmate/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## What a finished revision looks like

The included example runs one task that fails its check, goes back for rework,
and is accepted only on the second attempt. Summarised recorded outcome, not
literal CLI output:

| Attempt | Worker claim | Reported check | Reviewer | Lead decision | Earlier artifacts |
| --- | --- | --- | --- | --- | --- |
| 1 | completed | failed (exit 1) | approved | acceptance refused, rework requested | kept |
| 2 | completed | passed (exit 0) | approved | accepted | attempt 1 still recorded |

The reviewer approves both attempts; the failed check still prevents the first
acceptance. The earlier attempt remains in the record after rework.

## Who owns what

- Your agent host owns model calls, command execution, and permissions.
  The local record requires no model service or daemon.
- Check results are **caller reports**. `run record-check` accepts a command
  string and an exit status supplied by the host; it does not execute the
  command, authenticate the caller, or prove the host ran anything.
- A reviewer's `approved` outcome is role-scoped evidence. Only the configured
  lead can record `accepted`.
- Recorded artifact bytes must still match disk, or no new run event is written.
  This covers recorded artifacts, not every project file or the test environment.

Complete statement: [SECURITY.md](SECURITY.md) and [the authority boundary](REFERENCE.md#authority-boundary).

## Install and try it

The release is a single Linux x86_64 binary that also runs inside Ubuntu on
WSL 2. It needs no account, API key, or language runtime.

```sh
curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v0.12.0/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
```

The installer downloads a release archive, verifies its checksum, and installs
`soulmate` into `~/.local/bin`. Then run the token-free example:

```sh
soulmate benchmark
```

It builds its own disposable project; no model call, network request, or change
to your project is involved. Its first line and following summary are:

```text
False-completion proof passed (14/14 assertions).

The fixture observed a failed check, refused canonical acceptance, preserved the previous attempt, and accepted a fresh reviewed attempt after repair.
```

A final line reports the synthetic source, the CLI invocation count, and the
automated elapsed time, and states that human interaction time is unmeasured.

The fixture is synthetic and simulates the role submissions. Its frozen check
command validates a prepared configuration input, so the "repair" fixes that
input; no product bug was found or fixed. It establishes the behaviour it
exercises, not prevented losses or saved human time.
`soulmate benchmark --output proof-local` writes a new directory (it refuses to
overwrite one) holding the result, report, scenario, and a hash manifest — a
bundle of documents to open in an editor, not a live project ledger that
`run status` or `run inspect` will necessarily load. See
[the proof and its limits](docs/value-proof-methodology.md).

## What the acceptance check means

In checked runs, Soulmate refuses acceptance when the configured check result is missing or reports failure for the current worker artifact.

Each reported result binds to the frozen check command and the exact current
worker submission, identified by the `eventSha256` that submission returned; a
result recorded against an earlier attempt cannot qualify a new one. The
[claim registry](proof/claims.json) links the claim to its scenario and code.

## Start a checked run in your project

These four commands prepare a project; they do not perform or complete a task.
Replace `YOUR_TEST_COMMAND` with the command you already use to verify this
project. That exact string is frozen for the run.

```sh
soulmate init --mode portable
soulmate brief worker --task "Describe the change" --config soulmate.json
soulmate run start change --goal "Describe the bounded change" \
  --check-command "YOUR_TEST_COMMAND" \
  --ledger .soulmate/runs/run.jsonl --config soulmate.json
soulmate check --config soulmate.json
```

Include `--check-command` as shown; the generic starter suggestion printed by
`init` omits it. Without it, the run has no check-result requirement. `check`
validates configuration, profiles, and declared boundaries, not project tests.

Portable initialization creates `soulmate.json`, agent profiles under
`soulmate/`, private run records under Git-ignored `.soulmate/`, and skill
copies under `.agents/skills/soulmate/` and `.claude/skills/soulmate/`, and
installs no host hooks. [Local mode](REFERENCE.md#repository-modes) keeps
control and state outside the product checkout.

Your host must follow the workflow and submit each result. If it cannot spawn
the required native subagent, the bundled skill returns the pending assignment
to you. See [host setup](docs/onboarding.md). Work one assignment at a time:

```sh
soulmate run next .soulmate/runs/run.jsonl --config soulmate.json
soulmate run submit AGENT .soulmate/runs/run.jsonl --outcome OUTCOME \
  --artifact ARTIFACT_PATH --artifact-root state --config soulmate.json
```

`run next` returns JSON describing the pending assignment and an
`artifactPathHint`. Your host performs it and writes a real result file, using
the hinted path. `--artifact-root state` selects the root used by that hint.
Use a fresh file for each assignment and rework decision: lead `scoped`, worker
`completed`, reviewer `approved` or `rework`, then lead `accepted`, `rejected`,
or `rework`. Previously recorded files must keep their exact bytes.

After each worker submission, have the host run the frozen command and report
its real exit status:

```sh
check_exit=0
YOUR_TEST_COMMAND || check_exit=$?
soulmate run record-check .soulmate/runs/run.jsonl \
  --target WORKER_SUBMISSION_EVENT_SHA256 \
  --check-command "YOUR_TEST_COMMAND" --exit-code "$check_exit" \
  --config soulmate.json
```

`--target` is `event.eventSha256` in that worker submission's JSON output.
Replace both occurrences of `YOUR_TEST_COMMAND` with the frozen command. Full detail:
[first run](REFERENCE.md#first-run-details) and
[checked acceptance](REFERENCE.md#checked-acceptance).

## Inspect what happened

```sh
soulmate run status .soulmate/runs/run.jsonl --config soulmate.json
soulmate run inspect .soulmate/runs/run.jsonl --config soulmate.json
soulmate run report .soulmate/runs/run.jsonl --config soulmate.json
```

- `run status` prints human-readable text about the **current attempt only**:
  the completion claim, artifact condition, reported checks, review, and
  acceptance. `Claim` references the worker submission; `Acceptance` references
  the lead decision document. These references show hashes rather than paths.
- `run inspect` prints JSON for the whole run — every event and submission
  across all attempts — from the same ledger and configuration.
- `run report` prints markdown by default. It omits user-controlled text and
  keeps explicitly synthetic runs separate from local caller reports.
- `run explain` prints text. By default it selects no protection record and
  gives general guidance; pass `--event EVENT_SHA256` for one refusal with its
  exact check evidence. That is optional detailed guidance, not automatic
  diagnosis, and it neither chooses nor performs a repair.

`run next`, `run submit`, `run record-check`, and `run inspect` return JSON.
Raw ledgers and `run inspect` output can contain goals, commands, and paths;
keep them private. Hashes are not anonymization.

## When a transition is refused

For a reported failed check, the acceptance request prints:

```text
soulmate: acceptance refused: configured check evidence is check_failed
```

Start with `run status` for the current state and guidance. Nothing is accepted, but a
checked run can still record *why* it refused: a factual protection record
holding the reason and the exact check evidence, readable later with
`run explain --event`. Not every failed command appends such a record, and
artifact drift appends nothing at all: if recorded artifact bytes no longer
match disk, no new run event is written, the failing command's error names the
affected path, and `run status` flags the artifact as drifted.

Restore the exact recorded bytes when the change was accidental. When a
governing input changed on purpose, keep the predecessor sealed and start one
provenance-bound successor with `run supersede`; a successor does not bypass
artifact drift, and an `accepted` or `rejected` predecessor stays final. Under
a valid changed configuration in the same state root, `run inspect` can still
read the historical records because it does not recompare frozen inputs, while
`run status`, `run next`, and `run explain` can refuse on governing-input drift.
See [run and recovery](REFERENCE.md#run-and-recovery).

## Update or remove

Rerun the installer above to reinstall this version. After upgrading the
binary, refresh owned project skill copies explicitly:

```sh
soulmate init --refresh-skills --root PATH
```

Checked runs use run-event format 3, which older binaries reject; keep a
compatible binary if you may need those ledgers after a rollback. To remove
Soulmate, remove optional project hooks while the binary is still installed,
then remove the binary. Configuration, skill copies, ledgers, receipts, and
artifacts stay on disk until you delete them; see
[removal](REFERENCE.md#removal).

## Beyond the first run

The checked `change` workflow above is one entry point. Advanced features
remain available through `soulmate help advanced` and these guides:

- [Agent profiles and declared boundaries](REFERENCE.md#first-run-details)
- [Role-scoped, expiring, revocable memory](REFERENCE.md#memory-governance)
- [Execution receipts and harness manifests](REFERENCE.md#advanced-integrations)
- [Optional Codex and Claude hooks](REFERENCE.md#optional-codex-and-claude-hooks)
- [Repository modes and directory responsibilities](REFERENCE.md#directory-responsibility-boundaries)
- [All commands, configuration, recovery, and versioning](REFERENCE.md)
- [Codex, Claude Code, other hosts, and optional integrations](docs/onboarding.md)
- [Windows and WSL 2 installation](docs/windows-wsl.md)
- [Proof methodology and limitations](docs/value-proof-methodology.md)
- [Permissions, data, and reporting security issues](SECURITY.md)
- [Build and test workflow](.github/workflows/ci.yml) · [License](LICENSE)
