# Soulmate

It verifies what you asked an agent to do and what came back, in the same record.

**What still needs doing before I can accept this change?**

Keep your configured team’s agreed task, check, review, and decision together.
Use Soulmate when you want to inspect what “done” was based on after a handoff
or rework.

**See it happen:** a worker claims completion, a check fails, and acceptance is
refused. After rework, a fresh checked and reviewed attempt is accepted; the
earlier attempt remains available.

[![Rust primary CI](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml/badge.svg)](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/veyndrasystems/soulmate)](https://github.com/veyndrasystems/soulmate/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Install and see the result

This **preview** targets Linux x86_64 and macOS on Apple Silicon and Intel.
Ubuntu on WSL 2 uses the Linux binary. See the [platform support details](docs/platform-support.md).
The first experiment needs no account, API key, model, project configuration,
or language runtime. Your real work continues in your existing agent host.

```sh
curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v0.12.1-rc.3/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
```

The installer verifies the release archive checksum. Now run the small,
model-free experiment from any directory:

```sh
soulmate benchmark
```

Success starts with `False-completion proof passed (14/14 assertions).`
The experiment actually invokes Soulmate in a disposable project and reports:

```text
Attempt 1:
  Worker claim: completed.
  Host-reported check: failed (exit 1); synthetic caller report.
  Reviewer outcome: approved.
  Lead decision: pending; protocol refusal recorded (not a lead rejection).
Rework: preserved the previous attempt for the next assignment.
Attempt 2:
  Worker claim: completed.
  Host-reported check: passed (exit 0); synthetic caller report.
After the repair check, before final acceptance:
  Current fresh reviewer assignment: pending.
  Lead decision: pending.
  A passing check alone did not accept the run.
After review and lead acceptance:
  Reviewer outcome: approved.
  Lead decision: accepted.
```

It leaves your current project untouched and removes its temporary project.
To keep inspectable records, rerun with `soulmate benchmark --output NEW_DIRECTORY`.
This is a scripted configuration-repair task with real command exit codes;
actors are simulated. It demonstrates the protection, not time saved or agent
quality. [Inspect the experiment and its limits](docs/value-proof-methodology.md).

Your existing host can retrieve that pending review with the current check
and earlier attempt evidence. Carrying it out requires the host’s native
agent tools; if unavailable, the work remains pending. See [host setup](docs/onboarding.md#ask-your-existing-host-to-manage-the-run).

In checked runs, Soulmate refuses acceptance when the configured check result is missing or reports failure for the current worker artifact.

**Agree on the check before work starts.** Soulmate freezes that command and
ties each reported result to the submitted work. A reported failure stays
unresolved even when the worker says “completed” and the reviewer says
“approved.” Rework preserves earlier results; a passing report still needs
review and a lead decision.

## Use it for your next change

If your existing native setup already handles these handoffs reliably, the
extra record may not justify another installation. See the [bounded participation
findings](docs/participation-validation.md) for exercised protections and the
limits of the evidence.

In your project directory, initialize portable setup. This writes reviewable
configuration and profiles to `soulmate.json` and `soulmate/`, private ignored
state to `.soulmate/`, and skills for Codex and Claude. It installs no hooks.
If setup must stay outside your checkout, use [local mode](REFERENCE.md#repository-modes).

```sh
soulmate init --mode portable --root .
```

Initialization prints a handoff with the actual configuration and skill paths.
Give it to your existing coding agent, replacing the task and test command:

> Use Soulmate for **this change**. Check it with **my existing test command**.
> Handle the records and tell me what still needs doing before I can accept it.

Review the declared scope, native agent mapping, and host permissions once.
Your host supplies models and execution; setup does not start agents or grant
permissions. [Host setup](docs/onboarding.md#ask-your-existing-host-to-manage-the-run)
explains discovery and local-mode paths. Before starting, your host checks that
it can invoke the configured native workers and reviewers. You supply the task
and the decisions you own; the host handles assignment lookup, fresh reports,
and check records.
Ordinary single-agent work can proceed without a run.

A useful answer identifies the changed result, the reported check, the review,
and the pending action or lead decision, with references available for inspection.
The core path underneath is `init -> brief -> run -> check`.
`--check-command` selects a checked run; omitting it starts an unchecked run.
`soulmate check` validates configuration, profiles, and declared boundaries;
it does not run your project tests.

For a longer example with a one-file product check, run the
[scripted checked-work example](docs/first-checked-run.md#try-the-complete-example)
from the matching checkout: `SOULMATE_BIN=soulmate ./scripts/demo-checked-work.sh`.
The preview supports its `--event-id` and `--text` flags; stable 0.12.0 supports
the host-managed JSON workflow but does not recognize those two flags.

## When work fails or changes

Give your host the run's ledger path and ask what remains. `run status` shows
the current claim, artifact condition, reported check, review, and acceptance;
`run next` retrieves the validated pending assignment. `run inspect` retains
the full history. [Inspect or recover a run](docs/repair-a-run.md).

A failed check needs explicit rework with fresh artifacts and review. Recorded
artifact bytes must still match disk, or no new run event is written. Restore
legitimate recorded bytes for artifact drift; intentionally changed governing
inputs need explicit supersession where permitted. Accepted and rejected runs
stay final. The record recovers task evidence, not the host conversation.

## What the record establishes

Check results are **caller reports**. Soulmate binds the supplied command and
exit code to a worker submission; the host executes the check. Reviewer
`approved` and the configured lead's `accepted` remain separate. Your tests
and CI still determine what was actually exercised. See the
[authority boundary](REFERENCE.md#authority-boundary).

Artifact checks cover recorded documents, not every product file or its test
environment. A passing check can miss a bug. Keep raw ledgers, assignments,
and result documents private: they can contain goals, commands, and paths;
hashes are not anonymization. Read [SECURITY.md](SECURITY.md) before real work.

## Updates and removal

The installer reinstalls its pinned version. After a compatible binary upgrade,
refresh owned skill copies with `soulmate init --refresh-skills --root PATH`.
The new preview's `check` shows the running binary and managed-skill hashes;
a difference is a prompt to inspect versions, not proof of incompatibility.
Checked ledgers use run-event format 3; use the
[format and reader map](CHANGELOG.md#public-tags-and-format-readers) on rollback.
Removing the binary leaves configuration, skills, receipts, ledgers, and
artifacts. Remove optional hooks first while the binary is available.
[Update, inspect remnants, and remove](REFERENCE.md#removal).

## Go deeper

- [First checked run](docs/first-checked-run.md) · [Host setup](docs/onboarding.md)
- [Repair or resume](docs/repair-a-run.md) · [Translate the terminology](docs/glossary.md)
- [Optional memory, hooks, and receipts](docs/optional-surfaces.md)
- [Windows and WSL 2](docs/windows-wsl.md) · [Commands and versioning](REFERENCE.md)
- [Proof methodology](docs/value-proof-methodology.md) · [Claim registry](proof/claims.json)
- [Security](SECURITY.md) · [License](LICENSE)
