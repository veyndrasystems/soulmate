# Soulmate

It verifies what you asked an agent to do and what came back, in the same record.

**What still needs doing before I can accept this change?**

Use your existing coding agents to answer that question through a failed check,
rework, review, and a final decision. Soulmate keeps the connections between
those results so your host can show what is pending and retrieve earlier work.

[![Rust primary CI](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml/badge.svg)](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/veyndrasystems/soulmate)](https://github.com/veyndrasystems/soulmate/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Use it for your next change

This **preview release** includes the checked-work conveniences below. Install
the Linux x86_64 binary, also usable inside Ubuntu on WSL 2. It needs
no account, API key, or language runtime after installation. Your existing
agent host still supplies the models, execution, and permissions.

```sh
curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v0.12.1-rc.1/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
```

The installer verifies the release archive checksum. Initialization prepares
configuration and profiles in `soulmate.json` and `soulmate/`, private ignored
state in `.soulmate/`, and project skills for Codex and Claude. It installs no
hooks. Review the declared scope, native worker/reviewer mapping, and host
permissions once; [local mode](REFERENCE.md#repository-modes) keeps setup
outside the product checkout.

```sh
soulmate init --root .
```

In your existing agent conversation, ask:

> Use Soulmate for **this change**, with my configured native worker and
> reviewer. Run **my real project test command**. Handle the handoffs and
> records, then tell me what changed and what still needs doing before the
> lead can accept it.

Replace the bold text with your task and check. Have the host read the generated
Soulmate skill; [host setup](docs/onboarding.md#ask-your-existing-host-to-manage-the-run)
explains discovery. The host handles assignment lookup, fresh reports, and check
records. You supply the task and the decisions you own. Ordinary single-agent
work can proceed without a run.

A useful answer identifies the changed result, the reported check, the review,
and the pending action or lead decision, with references available for inspection.
The core path underneath is `init -> brief -> run -> check`.
`--check-command` selects a checked run; omitting it starts an unchecked run.
`soulmate check` validates configuration, profiles, and declared boundaries;
it does not run your project tests.

## See the protection before using a model

```sh
soulmate benchmark
```

Success starts with `False-completion proof passed (14/14 assertions).`
This token-free synthetic fixture uses a disposable project. It measures
prepared checks, not human time or agent effectiveness. [Inspect its evidence](docs/value-proof-methodology.md).

The [scripted checked-work example](docs/first-checked-run.md#try-the-complete-example)
shows the fuller outcome below. These are recorded results, not literal CLI output:

| Attempt | Worker claim | Reported check | Reviewer | Lead decision | Earlier artifacts |
| --- | --- | --- | --- | --- | --- |
| 1 | completed | failed (exit 1) | approved | acceptance refused, rework requested | kept |
| 2 | completed | passed (exit 0) | approved | accepted | attempt 1 still recorded |

In checked runs, Soulmate refuses acceptance when the configured check result is missing or reports failure for the current worker artifact.

A reviewer approval cannot override that failure. The first acceptance is refused with:

```text
soulmate: acceptance refused: configured check evidence is check_failed
```

The preview binary supports `--event-id` and `--text`. Run the full example
from the matching checkout with `SOULMATE_BIN=soulmate ./scripts/demo-checked-work.sh`;
building from source is optional. The earlier stable 0.12.0 binary supports
the host-managed JSON workflow but does not recognize these two flags.

## When work fails or changes

Give your host the run's ledger path and ask what remains. `run status` shows
the current claim, artifact condition, reported check, review, and acceptance;
`run next` retrieves the validated pending assignment. `run inspect` retains
the full history. [Inspect or recover a run](REFERENCE.md#run-and-recovery).

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

If your existing native setup already handles these handoffs reliably, the
extra record may not justify another installation. Our [validation findings](docs/participation-validation.md)
separate exercised protections from unproven time savings and code-quality claims.

## Updates and removal

The installer reinstalls its pinned version. After a compatible binary upgrade,
refresh owned skill copies with `soulmate init --refresh-skills --root PATH`.
Checked ledgers use run-event format 3; retain a compatible binary on rollback.
Removing the binary leaves configuration, skills, receipts, ledgers, and
artifacts. Remove optional hooks first while the binary is available.
[Update, inspect remnants, and remove](REFERENCE.md#removal).

## Go deeper

- [First checked run](docs/first-checked-run.md) · [Host setup](docs/onboarding.md)
- [Profiles and boundaries](REFERENCE.md#first-run-details) · [Optional memory](REFERENCE.md#memory-governance)
- [Receipts and integrations](REFERENCE.md#advanced-integrations) · [Optional hooks](REFERENCE.md#optional-codex-and-claude-hooks)
- [Windows and WSL 2](docs/windows-wsl.md) · [Commands and versioning](REFERENCE.md)
- [Proof methodology](docs/value-proof-methodology.md) · [Claim registry](proof/claims.json)
- [Security](SECURITY.md) · [License](LICENSE)
