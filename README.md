# Soulmate

It verifies what you asked an agent to do and what came back, in the same record.

Keep a change inspectable from the agent's first completion claim through a
failed check, rework, review, and final acceptance. Whether this is your first
agent task or part of an established review workflow, the same local record
shows what is pending and which earlier results led to the current attempt.

[![Rust primary CI](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml/badge.svg)](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/veyndrasystems/soulmate)](https://github.com/veyndrasystems/soulmate/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## What a finished revision looks like

The [scripted checked-work example](docs/first-checked-run.md#try-the-complete-example)
repairs a tiny product input. This is its recorded outcome, not literal CLI output:

| Attempt | Worker claim | Reported check | Reviewer | Lead decision | Earlier artifacts |
| --- | --- | --- | --- | --- | --- |
| 1 | completed | failed (exit 1) | approved | acceptance refused, rework requested | kept |
| 2 | completed | passed (exit 0) | approved | accepted | attempt 1 still recorded |

The reviewer approves both attempts; the failed check still prevents the first
acceptance. After rework, a fresh process retrieves the assignment and references
to the earlier artifacts. The fixture scripts all roles and invokes no model.

In checked runs, Soulmate refuses acceptance when the configured check result is missing or reports failure for the current worker artifact.

## Before using it

Your agent host owns model calls, command execution, and permissions. Check
results are **caller reports**: Soulmate binds a supplied command and exit code
to a worker submission; it does not prove that the host executed the check.
Reviewer `approved` and the configured lead's `accepted` remain separate.
See the [authority boundary](REFERENCE.md#authority-boundary).

Recorded artifact bytes must still match disk, or no new run event is written.
This covers recorded artifacts, not every project file or the test environment.
Keep raw ledgers, assignment output, and result documents private: they can
contain goals, commands, and paths. Hashes are not anonymization. The record
recovers task evidence; it does not preserve the host conversation. Read
[SECURITY.md](SECURITY.md) before using real project data.

## Install and try it

The distribution target is one Linux x86_64 binary, also usable inside Ubuntu
on WSL 2. It needs no account, API key, or language runtime after installation.

**Source candidate:** the `--event-id` and `--text` conveniences described here
are unreleased changes. The tagged installer below installs the pinned release;
use the source path below to exercise the new flow until it is released.

```sh
curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v0.12.0/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
```

The installer verifies the release archive checksum and installs `soulmate`
into `~/.local/bin`. Then run the installed, token-free self-test:

```sh
soulmate benchmark
```

Success starts with `False-completion proof passed (14/14 assertions).` This
synthetic fixture validates a prepared configuration input in its own temporary
project. It measures automated execution, not human time or agent effectiveness.
`soulmate benchmark --output proof-local` retains a new evidence bundle; it
refuses to overwrite an existing directory. See [what to inspect and what the
proof establishes](docs/value-proof-methodology.md).

For the complete checked-work journey, from this source checkout with Rust and
standard POSIX shell tools installed:

```sh
cargo build --locked
SOULMATE_BIN=target/debug/soulmate ./scripts/demo-checked-work.sh
```

The script creates and removes a disposable project, executes its real check,
shows one refused acceptance, retrieves the rework assignment in a fresh
process, repairs the input, and finishes with lead acceptance. It touches no
user project and makes no model or network call. [Follow the full example,
then use your own artifacts and check](docs/first-checked-run.md).

## Use it for your next change

The core path is `init -> brief -> run -> check`. Initialization prepares
configuration and profiles; the host still performs the work. Choose the
project's real check before starting a run. The updated starter includes
`--check-command "YOUR_TEST_COMMAND"`; replace it with your command. Omitting
that option starts an unchecked run. `soulmate check` validates Soulmate
configuration, profiles, and declared boundaries; it does not run project tests.

Portable initialization creates `soulmate.json`, reviewable profiles under
`soulmate/`, private Git-ignored state under `.soulmate/`, and project skill
copies for Codex and Claude. It installs no host hooks. Use [local
mode](REFERENCE.md#repository-modes) to keep control and state outside the
product checkout. Set the declared scope and host permissions before real work.

The [first checked run](docs/first-checked-run.md#use-your-own-project) covers
scope, a real worker artifact, capturing the submission ID, executing the
frozen check, review, lead acceptance, and rework. [Host
setup](docs/onboarding.md) explains discovery and native subagent handoffs.

## Inspect or resume

With the updated source binary, from the initialized project:

```sh
soulmate run status .soulmate/runs/run.jsonl --config soulmate.json
soulmate run next .soulmate/runs/run.jsonl --text --config soulmate.json
soulmate run inspect .soulmate/runs/run.jsonl --config soulmate.json
soulmate run report .soulmate/runs/run.jsonl --config soulmate.json
```

`status` shows the current attempt's claim, artifact condition, reported check,
review, and acceptance, with a read-only next action. `next --text` shows the
validated pending assignment, goal, check policy, and earlier/current artifact
references. It does not execute an assignment or display artifact contents.
Default `next` output remains JSON for hosts.

`inspect` returns the whole recorded history as JSON, including earlier
attempts. `report` produces Markdown with user-controlled text omitted and
synthetic runs separated from local caller reports. For one recorded refusal,
use `run explain --event EVENT_SHA256`; it does not choose or perform a repair.

A failed reported check produces this acceptance refusal:

```text
soulmate: acceptance refused: configured check evidence is check_failed
```

Request explicit rework and use fresh artifacts. If a recorded artifact drifts,
restore its exact legitimate bytes; no transition can append while it differs.
For intentionally changed governing inputs, inspect the predecessor and use
explicit supersession where permitted. Accepted and rejected runs stay final.
See [recovery and its limits](REFERENCE.md#run-and-recovery).

## Checks, updates, and removal

Your project's CI still executes and enforces its checks. Soulmate adds the
local binding between a reported result, its worker submission, and acceptance;
it does not replace branch protection or authenticate CI results. This
repository's [CI](.github/workflows/ci.yml) gates its executable public examples
and format/transition checks. Those fixtures establish their exercised cases,
not successful use by external users.

After upgrading a compatible binary, explicitly refresh owned skill copies:

```sh
soulmate init --refresh-skills --root PATH
```

The tagged installer above reinstalls that version. Checked
ledgers use run-event format 3, which older binaries reject; retain a compatible
binary when rolling back. Removing the binary leaves configuration, skill
copies, receipts, ledgers, and artifacts on disk. Remove optional hooks first
while the binary is available; see [removal](REFERENCE.md#removal).

## Go deeper

Advanced capabilities remain optional through `soulmate help advanced`:

- [Complete first checked work](docs/first-checked-run.md) · [Host setup](docs/onboarding.md)
- [Profiles and declared boundaries](REFERENCE.md#first-run-details)
- [Role-scoped, expiring, revocable memory](REFERENCE.md#memory-governance)
- [Receipts and harness manifests](REFERENCE.md#advanced-integrations)
- [Optional Codex and Claude hooks](REFERENCE.md#optional-codex-and-claude-hooks)
- [Storage modes](REFERENCE.md#repository-modes) · [Windows and WSL 2](docs/windows-wsl.md)
- [Commands, recovery, and versioning](REFERENCE.md)
- [Proof methodology and limitations](docs/value-proof-methodology.md) · [Claim registry](proof/claims.json)
- [Security](SECURITY.md) · [License](LICENSE)
