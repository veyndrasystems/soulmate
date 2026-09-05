# Soulmate command and protocol reference

It verifies what you asked an agent to do and what came back, in the same
record.

Soulmate is a provider-free local protocol for bounded coding-agent handoffs.
It records task envelopes, hashes, run transitions, and memory-lifecycle
evidence. Native hosts such as Codex and Claude Code still own models,
permissions, execution, and subagents. Soulmate is not an operating system or
process sandbox; only the optional `soulmate away` convenience launches one
pending Codex assignment.

## Quick start

Install the supported release as a single Rust binary. Node.js, npm, Python,
and Cargo are not required after installation:

```sh
curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v0.11.0/install.sh | sh
soulmate init --mode portable
soulmate brief worker --task "Describe the change you want to make" --config soulmate.json
soulmate run start change --goal "Describe the bounded change" --ledger .soulmate/runs/run.jsonl --config soulmate.json
soulmate check --config soulmate.json
```

The v0.11.0 release artifact supports Linux x86_64. Windows is supported through
Ubuntu on WSL 2 using that Linux artifact and keeping the agent, Soulmate, and
project inside the distribution; see the [Windows WSL 2 guide](docs/windows-wsl.md).
There is no native Windows executable or native Windows away runner. macOS is
tested from source in CI, but no macOS binary is published. The installer fails
before downloading on unsupported native platforms; additional native targets
will be added only after their build and installed-path behavior are verified.

That creates `soulmate.json`, canonical profiles and empty public control
directories under `soulmate/`, and private evidence directories under
`.soulmate/`. It also copies the bundled skill into
`.agents/skills/soulmate/SKILL.md` and
`.claude/skills/soulmate/SKILL.md` as host discovery projections, not as
Soulmate identity. Downloading a release binary never copies those files or
changes global host configuration. The default public surface is four command
families: `init` creates the local contract, `brief` renders an assignment,
`run` records its resumable handoff, and `check` validates the current
configuration. The host still performs the returned assignment.

## Authority boundary

- Soulmate owns task envelopes, selected configuration/profile evidence,
  run-state events, artifact hashes, and explicit memory-lifecycle evidence.
- The native host owns provider authentication, model execution, subagent
  spawning, permissions, and enforcement of the declared boundary.
- The configured lead owns scope decisions and final `accepted` authority;
  reviewer `approved` is role-scoped evidence, not acceptance.
- Complete security limitations and non-guarantees live in
  [SECURITY.md](SECURITY.md).

[Native conversation continuity](docs/native-continuity.md) describes the
root-thread preservation rule, its distinction from durable role memory, and
the limits of the advisory hook and regression evidence.

## First run details

`check` validates the configuration, profiles, and declared boundaries. `brief`
renders one bounded task packet without invoking a provider or launching a
model. The following is an abbreviated rendering; the profile hash is omitted
only to keep the example readable:

```text
# Soulmate task envelope: worker

Display name: worker
Native task name: worker
Purpose: Implement one bounded task without redefining architecture.
Task: Describe the change you want to make
Profile: soulmate/agents/worker.md
Profile SHA-256: ...
Requested runtime: host=none, model=none, reasoning effort=none, fallback=none

## Declared boundary

- Observe: none
- Write: none
- Commands: none
- Skills: none
- Memory read: none
- Memory write: none
- Memory review: none
- Memory promote: none
- Memory reject: none
- Memory revoke: none
- Memory expire: none
- Memory forget: none
- Retention: task
- Cross-context: none

> This is a plan-only brief. Runtime fields are requested bindings, not a model invocation or OS sandbox. The lead remains responsible for scope and final acceptance.
```

Replace `worker` and the task text with the role and work you need. The brief's
empty lists are declarations, not permission enforcement; the full boundary is
in [SECURITY.md](SECURITY.md). The installed Rust binary is the sole language
runtime requirement. Inside a Git worktree, initialization and state mutation
also require `git` on `PATH` for the publication preflight.

For a cheap end-to-end local exercise, jump to [Evaluation](#evaluation).

Start the first resumable handoff after initialization:

```sh
soulmate run start change --goal "Describe the bounded change you want to make" --ledger .soulmate/runs/run.jsonl --config soulmate.json
soulmate run next .soulmate/runs/run.jsonl --json --config soulmate.json
# The native host performs the returned assignment and writes its artifact at
# the returned artifactPathHint, then submits the named assignment:
soulmate run submit AGENT .soulmate/runs/run.jsonl \
  --outcome OUTCOME --artifact ARTIFACT_PATH --artifact-root state \
  --config soulmate.json
```

Replace the placeholders with the assignment and role-appropriate outcome.
Repeat `run next`, native execution, and `run submit` for each returned
assignment (including any rework cycle). Only the configured lead can record
the final `accepted` outcome; reviewer `approved` is not acceptance:

```sh
soulmate run submit lead .soulmate/runs/run.jsonl \
  --outcome accepted --artifact ARTIFACT_PATH --artifact-root state \
  --config soulmate.json
```

Submissions are role-scoped evidence, not votes; matching outcomes do not prove
consensus.

Output from an external planner or orientation tool may seed the task, but it
is not assignment authority. Before starting a run, restate exact observe,
write, and command limits in Soulmate configuration or a boundary manifest. If
the producer omits one of those structured fields, treat the handoff as
incomplete; do not recover authority by parsing guide prose or inferring the
producer's rules.

For a project outside the current directory, pass `--root PATH` to `init` and
use the printed `--config` path with later commands. A checkout can also be run
directly with a release binary from GitHub.

To update project skill copies after upgrading the CLI, use the explicit
refresh path. It requires an existing valid `soulmate.json` and only updates
files carrying Soulmate's ownership marker; unowned or conflicting files cause
the command to refuse the update:

```sh
curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v0.11.0/install.sh | SOULMATE_VERSION=v0.11.0 sh
soulmate init --refresh-skills --root PATH
```

The installer verifies the tagged archive checksum and stages the replacement
before switching the binary. It never rewrites project configuration or skill
copies; refresh remains a separate, reviewable project action.

The installer currently verifies the archive's SHA-256 checksum only. Online
GitHub provenance is a separate, optional check for downloaded archives; it
does not prove source correctness or maintainer intent. With GitHub CLI, verify
the release archive and its repository provenance explicitly:

```sh
gh attestation verify soulmate-x86_64-unknown-linux-gnu.tar.gz \
  --repo veyndrasystems/soulmate
```

Coffee is an optional project skill. Install the exact bundled Coffee and
Soulmate copies for both Codex and Claude Code only when you opt in:

```sh
soulmate init --mode portable --with-coffee --root PATH
soulmate init --refresh-skills --with-coffee --root PATH
```

Plain `soulmate init` and refresh never install or refresh Coffee. The
`--with-coffee` flag is the only activation path; there is no `soulmate coffee`
command, MCP server, daemon, provider integration, or automatic tool runner.

Removing the binary or project configuration does not delete these project
skill directories. Remove the selected `.agents/skills/<name>/` and
`.claude/skills/<name>/` directories explicitly when no longer wanted. Optional
Codex/Claude hooks remain a separate, explicit `soulmate hooks apply` choice.

Historical records remain inspectable through committed frozen fixtures;
the installed Rust binary and Rust test suite are the only runtime and
compatibility authority.

## Repository modes

Initialization inside a Git worktree requires an explicit publication choice.
Portable mode keeps `soulmate.json` and the public `soulmate/` control tree,
including canonical profiles under `soulmate/agents/`, with the checkout;
runtime state stays ignored and mutation is refused if Git tracks or stages it:

```sh
soulmate init --mode portable --root /path/to/product
```

Local mode is for separately owned harness and product repositories. It writes
no Soulmate-owned file beneath the product checkout:

```sh
soulmate init --mode local --project-id my-project \
  --root /path/to/product \
  --control-root /path/to/harness/my-project \
  --state-root /private/state/soulmate/my-project
```

`ControlRoot` owns configuration and canonical profiles. Project-scoped skill
copies may also live there as distribution or host projections; they do not
become agent identity. `ProductRoot` owns source and submitted product
artifacts; `StateRoot` owns ledgers, locks, receipts, and private runtime
evidence. The local roots must be existing,
canonical, non-symlinked, non-nested directories. A private machine-local
binding maps the portable project ID to ProductRoot and StateRoot. On another
machine, recreate only that binding with `soulmate bind --config CONFIG --root
PRODUCT --state-root STATE`; Soulmate does not copy or synchronize run state.

Existing projects that still reference `.agents/profiles/` can inspect an
explicit, deterministic migration before changing any bytes:

```sh
soulmate migrate layout --config CONFIG
soulmate migrate layout --apply --config CONFIG
```

The dry run prints the exact source, target, agents, hashes, and before/after
configuration hashes. Apply refuses symlinks, destination collisions, and
tracked or staged Git paths; it repoints only configured legacy profiles and
does not rewrite prior ledgers or receipts.

Existing projects that predate the full public/private directory contract can
inspect, then apply, path migration:

```sh
soulmate migrate paths --config CONFIG
soulmate migrate paths --apply --config CONFIG
```

The dry run reports missing directories and any `copy-retain-legacy` of a root
`harness-manifest.json`. Apply creates the directories and copies reviewed
manifest bytes to `soulmate/harness/harness-manifest.json` while retaining the
legacy file. It never moves ledgers or receipts, never rewrites historical
evidence, and is unchanged when repeated against an already-complete layout.

## Directory responsibility boundaries

Portable mode separates reviewable control material, private runtime evidence,
and optional host discovery copies:

```text
PROJECT/
├── soulmate.json                   # agent-system configuration
├── soulmate/                       # reviewable profiles and boundaries
│   ├── agents/lead.md
│   ├── agents/worker.md
│   ├── agents/reviewer.md
│   ├── boundaries/                 # prepared empty
│   ├── policies/                   # prepared empty
│   └── harness/                    # prepared empty; no manifest yet
├── .soulmate/                      # private runtime evidence
│   ├── .gitignore
│   ├── runs/
│   ├── memory/
│   ├── artifacts/
│   ├── receipts/
│   ├── away/
│   └── locks/
├── .agents/skills/soulmate/        # host discovery copy; not identity
└── .claude/skills/soulmate/        # host discovery copy; not identity
```

These names assign storage responsibilities, not ownership of product source
or host configuration. `soulmate/` is canonical control material;
`.soulmate/` is private runtime evidence; the skill directories are host
projections, never agent identity. `init` does not create `.codex/`,
`agents.toml`, hook settings, or a harness manifest. In local mode the public
tree lives under ControlRoot and private state under StateRoot, leaving
ProductRoot untouched.

## What you get

The primary standalone workflow is:

```text
soulmate init        create a starter contract and canonical soulmate/agents profiles
soulmate check       validate profiles and boundaries without running an agent
soulmate brief       compile one bounded task packet
soulmate plan        map a goal to bounded named-agent handoffs
soulmate run         resume handoff state and record artifact evidence
soulmate verify      check recorded configuration and profile bytes for drift
soulmate memory      record and inspect authorized memory lifecycle evidence
```

The JSON configuration in [examples/soulmate.json](examples/soulmate.json) is a
working fixture for the deterministic core. `soulmate doctor` can report
missing profiles and optional host commands without invoking them. When no
direct dotagents command is on `PATH`, it reports an observed `npx` launcher
and `agents.toml` separately without claiming that the package was invoked.
The optional `soulmate hooks` command manages only explicit project-local
Codex or Claude bindings.

To reuse a project-specific agent brief, audit it first and keep iterating on a
compact portable runtime brief until the audit is clean:

```text
soulmate profile audit ./old-agent --forbid-term OLD_PROJECT --json
soulmate profile import portable_worker ./old-agent --purpose "Carry a bounded portable brief"
```

Audit accepts a regular file or a directory with one of the narrow supported
runtime-brief candidates. It reports only finding categories and line numbers;
detectors cover private-home paths, terminal prompts, private-key material,
credential token signatures and assignments, project-coupled paths, and an
optional forbidden term. `valid`/clean means only that no configured pattern
matched; it does not prove portability, secret-freedom, identity removal, or
publication safety. The audit never prints matched content. Import copies the
audited bytes, grants no observe/write/command/skill/memory rights, retains the
profile for the task, and does not assign it to a workflow. After reviewing the
result, explicitly add the agent to the intended workflow and execution
boundary.

## Memory governance

Memory is an opt-in filesystem protocol. Content stays in the operator-named
project file; one private hash-linked ledger records only its relative
path/hash, scope, actor/profile evidence, lifecycle, and producer.

```text
proposed -> reviewed -> accepted -> revoked
    |          |           `-----> expired
    `----------`----------> rejected
```

`memoryWrite` grants proposal authority only. Review, promotion, rejection,
revocation, expiry, and content-free forgetting evidence require separate
exact-scope rights. Rejected, revoked, expired, changed, malformed, duplicate,
or unauthorized items are not recalled.

```text
soulmate memory propose invariant_keeper docs/invariant.md --scope invariants --ledger .soulmate/memory/invariant.jsonl
soulmate memory review invariant_keeper .soulmate/memory/invariant.jsonl
soulmate memory promote lead .soulmate/memory/invariant.jsonl
soulmate memory resolve invariant_keeper --json --config soulmate.json
```

Recall is disabled without an explicit `memory` object. It scans only shallow
regular `*.jsonl` files under the configured project-relative root and fails
closed on unsafe evidence or strict item/byte budget overflow. Briefs, plans,
and runs carry deterministic content-free references; the native host reads
their exact sources. Semantic retrieval, embeddings, global memory, transcript
capture, and implicit truncation remain out of scope. See
[SECURITY.md](SECURITY.md) for path, expiry, concurrency, and erasure limits.

## Run and recovery

Run state is private operational data beneath StateRoot. Each assignment fixes
the selected profile, requested runtime, declared skills/boundary, memory
references, upstream artifact hashes, and producer evidence. These are
selection/presentation records, not proof that a host or model complied.

Use an exact run boundary to narrow configured maxima without editing config.
This example uses the separate advanced fixture `examples/soulmate.json`; it
is not the starter config created by `soulmate init`:

```sh
soulmate run start change --goal "One bounded change" \
  --ledger .soulmate/boundary-run.jsonl \
  --boundary examples/boundaries/change.json \
  --config examples/soulmate.json
```

Every attempt uses a fresh StateRoot artifact path. Prior artifacts are
immutable; drift, deletion, unsafe paths, tracked/staged private state,
configuration/profile/memory/boundary drift, or an unverifiable lock fails
closed before mutation.

During an attended active session, every implementation worker and reviewer
uses the host's native subagent spawn with the assignment's exact
`nativeTaskName`. If native spawn is unavailable, stop and return the pending
assignment to the operator; do not fall back to shell `codex exec` or
`soulmate away`. This temporary quarantine tracks
[openai/codex#31894](https://github.com/openai/codex/issues/31894), a strong
external symptom match for affected `codex exec` no-result runs, not a proven
root cause. Until a later repository change retires the rule after the upstream
resolution is independently verified on a supported CLI, exclude affected
`codex exec` no-result samples from provider-native completion and
token-efficiency baselines. Historical evidence remains in place; quarantine
does not delete or rewrite it.

When the operator explicitly disconnects during one already-authorized Codex
assignment, `soulmate away` uses a task-specific/private tmux socket and
session on the same host to keep that native process alive. This is a
same-host process handoff, not an OS sandbox or process isolation. It is part
of the single Rust binary and requires no Python, Node.js, daemon, or second
task store. The existing ledger and normal `run submit` remain canonical. The
runner rejects fallback selection, revalidates exact profile and memory bytes,
and refuses a second same-host launch of the same pending assignment.

For evidence-complete handoff, first create
`soulmate/harness/harness-manifest.json` as shown in
[Advanced integrations](#advanced-integrations), then create a receipt-v2 plan,
bind it at run start, and require it at launch. The default starter config has
no assignment with `runtime.host: codex`, so `away` cannot use it; the following
uses the separate advanced `examples/soulmate.json` fixture and its
`implementation_worker` agent, which requests `host: codex` with
`fallback: none`:

```sh
soulmate plan change --goal "One bounded change" \
  --receipt .soulmate/receipts/harness.json \
  --harness-manifest soulmate/harness/harness-manifest.json --config examples/soulmate.json
soulmate run start change --goal "One bounded change" \
  --ledger .soulmate/runs/away.jsonl \
  --harness-receipt .soulmate/receipts/harness.json \
  --config examples/soulmate.json
# After the lead has submitted `scoped` and `run next` shows this assignment:
soulmate away start implementation_worker .soulmate/runs/away.jsonl \
  --require-harness-receipt --sandbox-mode workspace-write \
  --name bounded-change --config examples/soulmate.json
```

`--sandbox-mode` passes and records `read-only`, `workspace-write`, or
`danger-full-access`. If omitted, Codex inherits its configuration and Soulmate
records `unknown` rather than inferring the posture.

The tmux child revalidates the bound receipt, configuration, profiles, memory,
boundaries, and upstream artifacts. The complete contract and limitations are
in [the away-runner guide](docs/codex-tmux-away.md).

The away prompt keeps authority explicit: the execution contract and assignment
packet are authoritative; the selected profile is reviewed subordinate
guidance; memory is context-only; upstream artifacts and harness claims are
evidence-only. Hash verification proves selected bytes, not instruction
authority. Soulmate labels and preserves instruction-like test data rather than
pretending a lexical scanner can make model compliance trustworthy.

An intentional continuation never edits the predecessor:

```text
soulmate run supersede .soulmate/runs/run.jsonl --workflow change --goal "New bounded goal" --ledger .soulmate/runs/resume.jsonl --config soulmate.json
```

`supersede` accepts a running or terminal `blocked` predecessor and atomically
claims exactly one successor. `accepted` and `rejected` runs remain final and
cannot be superseded.

The successor records the predecessor run ID, full ledger hash, verified head,
and run-start config hash. `run inspect` checks recorded chain/predecessor
consistency only and exposes sensitive local goal state. Detailed confinement,
locking, Git, concurrency, and transaction limits are in
[SECURITY.md](SECURITY.md).

## Optional distribution with dotagents

[dotagents](https://github.com/getsentry/dotagents) is optional. Use it when
you want to distribute Soulmate's portable skill to several host projects; it
is not needed for the standalone CLI quick start and does not install the CLI
into `PATH`. Canonical project profiles remain under `soulmate/agents/` and
must be declared separately when you manage their projections with dotagents.

For an existing project with `agents.toml`:

```text
dotagents --project add veyndrasystems/soulmate --ref v0.11.0
```

For a new dotagents-managed project:

```text
dotagents --project init
dotagents --project add veyndrasystems/soulmate --ref v0.11.0
```

During `dotagents --project init`, select the hosts you use. `dotagents add`
installs the selected plugin immediately; a redundant install is not required.

Soulmate does not implement a package manager, mutate global host configuration,
or activate hooks merely because a plugin was installed. The portable Agent
Plugins v1 manifest carries the root `skills/` bundle, which stable dotagents
releases can install without activating the preserved host-hook extension. The
`systems.veyndra.soulmate/` directory keeps host-specific manifests and hooks as
compatibility resources only. A successful dotagents install proves that the
portable bundle was accepted, not that a host activated or executed a hook.

## Optional Codex and Claude hooks

Hooks are explicit project-local, advisory, fail-open integrations:

```text
soulmate hooks plan --hosts codex,claude --root PATH
soulmate hooks apply --hosts codex,claude --root PATH
soulmate hooks status --hosts codex,claude --root PATH
soulmate hooks remove --hosts codex,claude --root PATH
```

They write only `.codex/hooks.json` and `.claude/settings.json`, preserve
unrelated settings, refuse malformed/conflicting/symlinked targets, and require
the expected `soulmate hook-run` protocol on `PATH`. Session context is a
bounded project summary; subagent context presents the exact selected profile
and declared boundary. Hook execution/presentation is not activation or model
compliance proof. POSIX behavior is CI-tested on Linux and macOS; Windows
mutation is refused. See [SECURITY.md](SECURITY.md) for race and transaction
limits.

In local mode, `hook-run` resolves ControlRoot only through the exact private
machine binding whose ProductRoot matches the host `cwd`; it never searches
arbitrary parent directories. Bindings created before ControlRoot was recorded
remain readable for normal configured commands. Repeat the original
`soulmate bind --config CONFIG --root PRODUCT --state-root STATE` command to
add ControlRoot after all three existing roots match exactly.

## Removal

Standalone users remove the installed binary explicitly:

```text
rm "${SOULMATE_INSTALL_PREFIX:-$HOME/.local/bin}/soulmate"
```

The project-scoped skill copies created by `soulmate init` are ordinary project
files and are intentionally not removed with the binary. Delete
`.agents/skills/soulmate/` and `.claude/skills/soulmate/` explicitly if the
project should no longer discover the Soulmate skill. Removing the binary does
not remove project files, ledgers, receipts, hooks, or skills.

If optional hooks or dotagents are enabled, remove project hooks while the CLI
is still available, then remove the optional plugin and CLI:

```text
soulmate hooks remove --hosts codex,claude --root PATH
dotagents --project remove soulmate
rm "${SOULMATE_INSTALL_PREFIX:-$HOME/.local/bin}/soulmate"
```

Reload the host or start a new session after plugin, hook, or skill changes.
Removing Soulmate leaves the underlying host and project usable.

## Evaluation

From a cloned checkout, this creates a disposable one-file sample project,
records the file as a run artifact, changes its bytes, and shows Soulmate
refusing the next transition. It invokes no model, consumes no model tokens,
touches no user project, and removes the temporary directory:

```sh
SOULMATE_BIN=target/debug/soulmate ./scripts/demo-refusal.sh
```

The refusal proves only the artifact-drift check. The authority and
non-guarantee contract is defined once in [SECURITY.md](SECURITY.md).

## Versioning and release contract

Soulmate uses the change in its public contract—not elapsed time or the number
of commits—to choose a version while it remains in `0.x`:

- `0.y.Z` is a backward-compatible defect, security, documentation, packaging,
  or release correction with no new user-visible capability or persisted shape.
- `0.Y.0` changes a product invariant or mechanically rejects a counterexample
  that the preceding minor line accepted. An intentional compatibility break
  also requires a migration decision and backward inspection where the prior
  format promised it.
- A receipt, ledger, configuration, or schema shape change must version that
  persisted format, add frozen compatibility fixtures, and document its
  inspection/migration contract; changing only the package number is insufficient.
- A new language runtime or external executable requirement is admissible only
  when it changes such an invariant; it also needs the same
  native/standard-library justification as a dependency.
- `1.0.0` requires external-use evidence that the documented CLI and persisted
  contracts are stable, recoverable, and supportable. It is not a completeness
  label and is never selected only because the feature list looks substantial.

`0.9.3` is a patch release because it removes an unsupported experimental
observation path from the product binary and corrects local-mode hook discovery
without changing the supported CLI or any receipt, run-event, memory,
configuration, or schema shape. A release tag is created only after its exact
commit passes the declared Linux, macOS, Windows-WSL, dependency-audit,
compatibility, privacy, and release gates.

## Advanced integrations

Runtime `host`, `model`, reasoning, and fallback values are opaque requested
bindings. Soulmate never stores provider credentials, routes models, or
silently substitutes a fallback. An agent's `skills` list is selection intent;
dotagents and the native host own installation, discovery, and invocation.

Coffee may prepare a bounded goal when separately available. Venus or another
cross-session system remains advisory: a human accepts/narrows its Goal before
Soulmate starts a run. Soulmate never ingests raw session history or gains
cross-session authority.

For opt-in dogfooding evidence, place a version-1 harness manifest beneath
ControlRoot and pass it only when creating an existing brief or plan receipt:

```json
{
  "$schema": "https://raw.githubusercontent.com/veyndrasystems/soulmate/v0.11.0/schema/harness-manifest.schema.json",
  "version": 1,
  "project": { "id": "my-project", "session": "codex-2026-08-30" },
  "harness": { "name": "my-harness", "version": "2026.08.30" },
  "activations": [
    { "kind": "skill", "name": "soulmate", "evidence": "configured" },
    { "kind": "perspective", "name": "qa-engineer", "evidence": "presented" },
    { "kind": "ponytail", "name": "ponytail:ponytail", "evidence": "hook_observed" }
  ]
}
```

```sh
soulmate plan change --goal "Bounded goal" \
  --receipt .soulmate/receipts/harness.json \
  --harness-manifest soulmate/harness/harness-manifest.json --config soulmate.json
soulmate verify .soulmate/receipts/harness.json --config soulmate.json
```

The canonical manifest path is fixed beneath ControlRoot. The legacy root
`harness-manifest.json` path remains verifiable for existing receipt-v2 bytes.
This creates receipt version 2 and binds the exact manifest bytes while persisting only SHA-256
identities plus the non-sensitive kind/evidence enums. Raw project, session,
harness, activation, and verifier strings remain in the operator-owned
manifest and are omitted from the receipt; deterministic hashes can still
permit offline confirmation of guessed low-entropy identifiers. Without
`--harness-manifest`, Soulmate continues to create and inspect receipt version
1. `configured`, `presented`, `agent_declared`, and `hook_observed` report
their source level; none proves model compliance. `independently_verified` is
an off-box verifier claim: Soulmate validates its format and binds it, but does
not hash a local artifact or authenticate the verifier. Unknown fields and free-form content
are rejected, so prompts, transcripts, secrets, raw environment values, and
unrelated project content have no manifest field. See
[the design boundary](docs/post-0.3.0-harness-evidence.md). OTel or Langfuse may
mirror the canonical receipt but never replace it as authority.

The manifest `$schema` value is advisory; compatibility is controlled by its
`version`, so a manifest using an older release schema URL remains accepted.

## Identifiers

The intended public identifiers are:

- GitHub: `veyndrasystems/soulmate`
- package namespace, if needed: `@veyndra/soulmate`
- CLI: `soulmate`

The unscoped npm package name `soulmate` is already owned by an unrelated
project, so Soulmate will not claim or depend on it.
