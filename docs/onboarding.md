# Onboarding by environment

The first successful outcome is the same everywhere: inspect a completion
claim, its check and review, the lead decision, and earlier results after
rework. [Ask your existing host](#ask-your-existing-host-to-manage-the-run) to
handle a real change, or inspect the model-free
[checked-work example](first-checked-run.md). Model execution remains in your
existing host. Read the [authority boundary](../REFERENCE.md#authority-boundary)
and [privacy limits](../SECURITY.md) before using real project data.

## Choose the storage mode

Use portable mode when the team wants profiles and declared boundaries reviewed
with the project:

```sh
cd PROJECT
soulmate init --mode portable
```

Use local mode when the product checkout must remain untouched. Control and
state directories must already exist outside its Git worktree:

```sh
mkdir -p "$HOME/.local/share/soulmate/my-project/control"
mkdir -p "$HOME/.local/state/soulmate/my-project"
soulmate init --mode local \
  --project-id my_project \
  --root PROJECT \
  --control-root "$HOME/.local/share/soulmate/my-project/control" \
  --state-root "$HOME/.local/state/soulmate/my-project"
```

Portable mode prepares project-local Codex and Claude discovery copies. Local
mode keeps those copies with ControlRoot instead of adding them to ProductRoot.

## Ask your existing host to manage the run

After initialization, have your host read the generated Soulmate skill and
check that the project's worker and reviewer map to the native agents you
already use. Review that project configuration once; keep their existing role
definitions and host permissions. In local mode, give the host the path to
ControlRoot's `soulmate.json` and skill because they are outside the product.

### Review the starter setup once

The starter intentionally grants no file or command access. Before `run start`,
have your host propose edits to the existing `soulmate.json` for this task:

| Field under each selected `agents` entry | What to put there |
| --- | --- |
| `profile` | Keep the generated profile or name the role profile you actually reviewed. |
| `observe` | Project-relative files or directories needed to understand and review this change. |
| `write` | Only the product/result paths that role is authorized to change; reviewers normally have no product writes. |
| `commands` | The real project check and other authorized commands needed for the task. |
| `nativeName` | The exact usable host task name, if it differs from the logical `worker` or `reviewer` ID. |

Keep the starter workflow and lead unless you intend to change their meaning.
Review these edits and the host's own permissions, then run
`soulmate check --config CONFIG` with the actual configuration path. It checks
configuration, profiles, and declarations; **it does not run project tests**.
A declaration is not a host permission grant or proof that all edits stayed
inside it. Leave memory rights empty for this first task.

Before `run start`, the coordinating context must verify that the actual host
session exposes the native worker and reviewer spawn tools required by the
selected workflow. A profile, task name, or plan-only brief is not native capability
evidence. If either tool is unavailable, return the limitation and pending work
to the existing lead or operator without starting a substitute executor. Do
not repeat a diagnostic brief, use shell `codex exec` or `soulmate away`, rename
the task, impersonate a role, or add a new permission. This is host guidance;
Soulmate does not detect or mechanically enforce this preflight.

Native names must be usable in the current host session. If the host keeps used
task names, resolve that before freezing a new run; see
[resuming with an existing host](repair-a-run.md). This setup should reuse the
roles you already trust. It does not require a new agent host or a model account
for Soulmate.

### Give the host the change

Then describe the work in your existing conversation, for example:

> Use Soulmate for **this change**. Check it with **my existing test command**.
> Handle the records and tell me what still needs doing before I can accept it.

The host can retrieve assignments, carry submission IDs, use fresh report
paths, execute the configured check, and record its actual result. These are
host operations, not instructions for you to copy JSON or maintain a second
task list. Scope changes and decisions still follow the configured
[authority boundary](../REFERENCE.md#authority-boundary); missing permission
or an unavailable native agent requires a real resolution.

This path works with the stable 0.12.0 JSON commands and with the preview,
without a Rust build. The bundled skill describes the handoff; your host
executes it. The preview adds optional `--event-id` and `--text` conveniences;
they are not prerequisites for host-managed work. Configuration and skill discovery alone do not prove
that an agent ran: inspect the actual native result and the recorded check.

For an existing failed run, give the host its ledger path and ask it to inspect
the pending work and recover within the approved scope. A failed check needs
fresh work and review; a changed artifact or governing input needs its
[specific recovery procedure](repair-a-run.md). Finish with
the result, remaining blocker if any, and the next decision you actually own.
Ordinary single-agent work can continue without a run.

## Keep the host you already use

`soulmate init` creates the first two paths below. The other rows are based on
the linked host's official discovery contract. "Path-compatible" means that the
host documents a path Soulmate already creates; it does not mean Soulmate ran
the host or proved that the model followed the skill.

| Existing host | Discovery path | Evidence and entry |
|---|---|---|
| [OpenAI Codex](https://developers.openai.com/codex/skills) | `.agents/skills/soulmate/` | Generated by `init`; keep Codex authentication, sandbox, and subagent settings. |
| [Claude Code](https://code.claude.com/docs/en/slash-commands) | `.claude/skills/soulmate/` | Generated by `init`; keep Claude authentication and permissions. |
| [Cursor](https://prod.cursor.com/docs/skills) | `.agents/skills/soulmate/` | Official-path compatible; host execution is not exercised by Soulmate CI. |
| [GitHub Copilot](https://docs.github.com/en/copilot/concepts/agents/about-agent-skills) | `.agents/skills/soulmate/` | Official-path compatible across Copilot's documented agent surfaces. |
| [OpenCode](https://opencode.ai/docs/skills) | `.agents/skills/soulmate/` | Official-path compatible; OpenCode remains the executor. |
| [Gemini CLI](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/using-agent-skills.md) | `.agents/skills/soulmate/` | Official-path compatible; Gemini retains its own activation consent. |
| [Cline](https://docs.cline.bot/customization/skills) | `.cline/skills/soulmate/` | Manual and experimental: `init` does not generate this host-specific path. |

For Linux shell or CI use, no coding host is required. On Windows, keep the
project, agent host, and Soulmate inside one Ubuntu WSL 2 distribution; the
published Linux binary follows the same paths exercised by WSL CI.

Host releases still matter. A [reported Copilot CLI 1.0.5
issue](https://github.com/github/copilot-cli/issues/2040) shows explicit skill
invocation working interactively but not in `-p` prompt mode; do not assume
interactive discovery proves headless automation. Gemini skills first appeared
in its [0.24.0 release
line](https://github.com/google-gemini/gemini-cli/blob/main/docs/changelogs/index.md),
so older installations need an update.

[dotagents](https://github.com/getsentry/dotagents) is optional when one project
needs projections for several hosts. In a disposable project, version 3.0.1
installed the local Soulmate plugin for Claude, Cursor, Codex, VS Code,
OpenCode, Pi, and Grok, and `dotagents doctor` passed. That verifies generated
plugin links, not host activation. Soulmate itself does not need dotagents.

## Verify before using a real project

Start with the installed binary, from any directory:

```sh
soulmate benchmark
```

No checkout, model, or project initialization is required. The disposable
configuration-repair fixture observes a real command failure, refused acceptance,
preserved earlier work, and a freshly checked and reviewed accepted attempt.
Its temporary project is removed; your current project stays untouched.
Use `soulmate benchmark --output NEW_DIRECTORY` to retain the inspectable
[bounded proof](value-proof-methodology.md). Human time and agent quality are
not measured by this scripted experiment.

For the longer one-file product example, use the matching source checkout:

```sh
SOULMATE_BIN=soulmate ./scripts/demo-checked-work.sh
```

The preview includes the `--event-id` and `--text` options used here. To build
from source instead, run `cargo build --locked` and use
`SOULMATE_BIN=target/debug/soulmate`. The script records a failing product check,
observes refused acceptance, requests rework, retrieves the assignment in a
fresh process, then repairs, checks, reviews, and accepts. Actor documents are
scripted and its temporary project is removed. The focused
[artifact-drift example](../scripts/demo-refusal.sh) remains available.

Next initialize a second disposable project and confirm the host sees the
generated skill; discovery controls differ by host release. A pending
assignment still needs the native host to execute it. If the required native
worker or reviewer spawn tool is unavailable, the skill returns the assignment
to the operator; it does not silently switch executors or use a plan-only brief
as evidence that the tool exists.

Follow [your first real checked run](first-checked-run.md#use-your-own-project)
for explicit scope, result snapshots, submission-ID capture before the host
check, review, lead acceptance, and rework. `run next --text` can retrieve the
recorded assignment later, with prior artifact references; it does not restore
the conversation or prove the host followed the skill.

After a binary update, explicitly refresh only owned project skill copies:

```sh
soulmate init --refresh-skills --root PATH
```

In the new preview, `check` reports the invoking binary version and hashes of
its bundled and installed skill copies. A managed difference produces an
actionable warning, including on stderr with `check --json`; the JSON result
still describes configuration validity. Missing optional copies or a
third-party skill do not make a valid configuration invalid. Diagnosis does
not refresh files or identify whether arbitrary different bytes are newer,
older, or incompatible. Use the intended binary for the explicit refresh.
Already-distributed 0.12.0 binaries cannot show these new diagnostics; inspect
`soulmate version` and update the binary before using preview-only guidance.

Reload the host or start a new session to discover the refreshed skill. This
refresh does not install hooks or change the host's authentication/permissions.
Keep a compatible binary for existing checked ledgers during rollback; see
[update and removal](../REFERENCE.md#removal).
