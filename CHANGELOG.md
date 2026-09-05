# Changelog

## Unreleased

## 0.11.0

- Made native conversation preservation an explicit host-guidance invariant:
  profiles, briefs, run records, and governed memory augment the active root
  conversation. They do not authorize replacing it or discarding recent user
  corrections, rejected approaches, or their rationale.
- Session-start hooks present the same bounded advisory distinction. Executable
  hook and generated-skill regression checks cover the behavior Soulmate owns.
- Added a diagnostic guide that separates thread replacement, compaction,
  injected context, and host/client integration hypotheses.
- This is synchronized guidance and tested local hook behavior, not proof of
  model compliance or resolution of a historical host-memory regression.
- Migration: update host guidance with the supported skill-refresh path. No
  CLI command, configuration, persisted format, dependency, or model runtime
  changes are introduced. The minor version records the new product invariant
  under the repository's versioning policy.

## 0.10.0

- Attended implementation workers and reviewers must now use the host's native
  subagent spawn with the assignment's exact native task name; when that spawn
  is unavailable, stop and return the pending assignment to the operator. This
  is synchronized host guidance, not Soulmate enforcement of host execution.
- `soulmate away` remains reserved for an explicit operator-away or disconnect
  continuation; the Rust away runtime is unchanged.
- Corrected hook-output home-path redaction so machine-local paths stay
  redacted while the public source remains safe for publication.
- Migration: update attended workflow handoffs to native spawn. This release
  adds no CLI, configuration, persisted schema or format, dependency, or
  runtime expansion.

## 0.9.3

- Removed the unsupported context-observation collector and report from the
  product tree and release binary. The implementation remains available for
  transparent development only on `experiment/context-observability`; locally
  collected diagnostic data remains private and is never published.
- Local-mode hooks now resolve the exact ControlRoot through the private
  machine binding while confirming that the hook ProductRoot still matches;
  repeating `soulmate bind` upgrades compatible older bindings in place.

## 0.9.2

- Added strict private ingestion of exact host-reported usage for internal
  context dogfooding, correlated to content-free run and assignment identities.
- Reports exact repeated and still-pending assignment invocations without
  treating an invocation as completion, success, or authority to skip work.
- Rejects malformed, duplicated, rerouted, unsafe, unmatched, and concurrently
  stale diagnostic evidence while leaving the default CLI and canonical
  persisted formats unchanged.

## 0.9.1

- Added non-default, private context-accounting dogfood instrumentation that
  measures only Soulmate-owned byte boundaries and keeps model/provider usage,
  host context, and broader optimization claims explicitly unknown.
- Hardened authority classification for mixed payloads and run-scoped
  observations without changing the default CLI or canonical persisted
  formats.

## 0.9.0

- Native away recovery now records the explicit or unknown sandbox posture and
  distinguishes exit codes from signal termination. Completion requires the
  exact stage, attempt, and agent submission event; merely leaving the pending
  set is reported separately.
- Path inputs used by commands, identifiers, or recorded evidence now reject
  non-UTF-8 rendering and unresolved working directories instead of silently
  substituting or collapsing paths. A repository allowlist gate prevents the
  rejected patterns from returning.
- Release and plugin manifests now share one version identity, and tag
  publication runs the release-reference gate before building artifacts.

## 0.8.0

- New projects and explicit `migrate paths` runs prepare the complete public
  `soulmate/` and private `.soulmate/` directory contract while retaining
  legacy manifest and evidence bytes.
- `doctor` now distinguishes a direct dotagents command from uninvoked
  `npx`/`agents.toml` observations.
- Clarified that external task handoffs are incomplete unless exact observe,
  write, and command authority arrives as structured fields rather than guide
  prose.
- Separated authoritative assignment contract, reviewed profile guidance,
  context-only memory, and evidence-only harness/artifact claims in the native
  away prompt, with instruction-like adversarial fixtures and security limits.
- Added an explicit deterministic `migrate layout` dry-run/apply path for
  configured `.agents/profiles/` sources. It refuses symlinks, collisions, and
  tracked or staged mutations while leaving historical evidence untouched.
- `run supersede` now permits one provenance-bound successor from a terminal
  `blocked` run while preserving immutable predecessor bytes and continuing to
  reject `accepted` and `rejected` predecessors.
- New portable and local projects, plus newly imported profiles, now keep
  canonical agent profiles under `soulmate/agents/`; existing configurations
  that reference `.agents/profiles/` remain valid and are never rewritten.
- Documented the Soulmate-centered directory contract while retaining
  `.agents/` for optional distribution and `.codex/`/`.claude/` for host-owned
  projections.

## 0.7.0

- Added opt-in run-event version 2, which binds an existing receipt-v2 harness
  record by StateRoot-relative path and exact SHA-256 while retaining generation
  and inspection of unbound version-1 runs.
- Added native `soulmate away` start/list/show commands. The single Rust binary
  now revalidates config, selected profile/runtime, boundary, memory, upstream
  artifacts, receipt, and exact manifest claims across an isolated tmux launch.
- Removed the project-copied Python away adapter and its Python CI dependency;
  0.6 scripts migrate to `soulmate away` without changing receipt/config shapes
  or the canonical `run submit` evidence path.
- Added fail-closed receipt/manifest drift, mixed-event-version, plan-coverage,
  zero-Codex-launch, recovery-state privacy, and real-tmux regression coverage.

## 0.6.0

- Added an optional, project-scoped Codex+tmux reference adapter that keeps one
  already-authorized pending assignment alive across an operator disconnect.
- Kept normal `run next`/`run submit` ledger evidence canonical; adapter status
  is private process-recovery evidence and no daemon, provider client, second
  JSONL store, persisted schema, or dependency was added.
- Revalidates the exact assignment, profile, memory hashes, runtime choice, and
  fresh StateRoot artifact path before launch; rejects fallback selection and a
  second same-host launch of the same live assignment.

## 0.5.0

- Made generated and published schema references follow the crate version while
  accepting older advisory harness `$schema` values for manifest version 1.
- Clarified that `independently_verified` records an off-box claim whose format
  is validated and bound, without hashing a local artifact or authenticating a
  verifier; SHA-256 inputs must be lowercase hexadecimal.
- Hardened onboarding path errors, documented fixed evidence shapes and the
  canonical-root precondition, and clarified macOS source-test-only support.

## 0.4.0

- Added an opt-in, versioned harness manifest that hash-binds portable project
  and session identifiers, harness identity, and bounded skill, perspective,
  and Ponytail activation evidence into the existing canonical receipt without
  copying raw manifest strings.
- Added receipt version 2 for harness-bound receipts while retaining version 1
  generation and verification when no manifest is supplied.
- Kept evidence levels explicit (`configured`, `presented`, `agent_declared`,
  `hook_observed`, and `independently_verified`) without treating presentation
  or declaration as proof of agent compliance.
- Rejects unknown manifest fields and free-form content; prompts, goals,
  transcripts, secrets, raw environment values, and unrelated project content
  are not accepted or copied into receipts.

## 0.3.0

- Removed the legacy Node implementation and moved publication privacy and
  compatibility coverage into Rust tests with frozen v0.0.8, v0.1.x, and
  v0.2.x ledger fixtures.
- Hardened Git worktree preflight against non-UTF-8 paths, missing Git,
  command failure, and inconsistent tracked/staged failure handling.
- Removed external-input panic paths from boundary, CLI JSON, timestamp, and
  memory path handling; `--json` failures now remain parseable.
- Continued the typed configuration transition for boundary, receipt, runtime,
  and memory selection without changing persisted receipt, ledger, or config
  shapes.
- Reorganized the bundled skill around recovery branches and distinguished
  configured, presented, agent-declared, hook-observed, and independently
  verified harness evidence in existing submitted artifacts.
- Pinned generated schema/install references to the release tag, added Linux
  dependency audit and macOS test coverage, and reduced duplicated README
  limitation prose in favor of the security boundary document.

## 0.2.1

- Fixed empty CLI invocation and added conventional `--help` and `--version`
  entrypoints without changing existing subcommand parsing.
- Added content-free memory-budget refusal diagnostics with the attempted
  item/byte totals at refusal and configured limits; strict no-truncation
  behavior is unchanged.
- Began a gradual typed configuration transition at the validated agent/runtime
  boundary while preserving raw configuration bytes and persisted formats.
- Added GitHub artifact provenance attestation and release-workflow verification
  for the packaged archive, while retaining installer SHA-256 checksum checks.
- Clarified that root Node files are preserved v0.0.8 compatibility/privacy
  fixtures, not a supported installed runtime.
- Fixed skill refresh to restore missing managed copies and report created,
  refreshed, or unchanged destinations truthfully after complete preflight.

## 0.2.0

- Added explicit portable and local repository modes with typed ControlRoot,
  ProductRoot, and StateRoot ownership, private machine-local project bindings,
  and Git preflight refusal for tracked or staged runtime state.
- Added exact run-scoped observe/write manifests that can only narrow configured
  maxima, remain hash-linked across resume, and fail closed on manifest drift.
- Added warning-level placeholder diagnostics and focused local-mode,
  publication-boundary, narrowing, drift, and parallel-assignment tests.
- Added immutable, attempt-scoped StateRoot artifact hints and explicit
  `--artifact-root state` submissions so rework does not overwrite prior
  evidence.
- Added producer version/commit evidence to newly created receipts and ledger
  events while preserving inspection of older records without that field.
- Updated Rust examples and CI, declared Rust 1.75 as the MSRV, and prevented
  release workflows from overwriting existing tag assets.

- Added opt-in, project-scoped recall of accepted memory evidence through a
  shallow bounded ledger root and exact per-agent scope/context filtering.
- Added content-free `memory resolve` output and deterministic memory references
  in briefs, plans, and native-host run assignments; source bytes remain in
  operator-owned project files, while the bundled skill requires exact reads
  immediately before a matching subagent spawn.
- Frozen run references now fail closed on source, lifecycle, ledger-head, or
  selection drift without breaking inspection of existing v0.0.8/v0.1.0
  ledgers.
- Supersession now rolls back only its own newly created predecessor claim when
  successor creation fails, so a corrected retry is not stranded.
- Kept semantic search and embeddings out of core; any future optional ranker
  may operate only after lifecycle and authorization eligibility.

## 0.1.0

- Replaced the user-facing Node.js runtime with a Rust 2021 single binary.
- Added Cargo build metadata, embedded skills, release artifacts, checksums,
  and an initially Linux x86_64 installer.
- Preserved the v0.0.8 JSON configuration, receipt, JSONL run, and memory
  evidence formats; Node files remain only as migration reference fixtures.
- Kept configuration drift fail closed and added explicit `run supersede`
  provenance: a new bounded ledger links to a verified predecessor head while
  leaving the predecessor unchanged and sealed against later submissions.
- Clarified that role outcomes are evidence records, not votes or proof of
  multi-agent consensus; the configured lead retains final acceptance authority.
- Initially support Linux x86_64 release artifacts; unsupported platforms fail
  before download rather than receiving an unverified cross-compiled binary.

## 0.0.8

- Added explicit `soulmate init --with-coffee` and matching refresh support to
  install the bundled Coffee skill for Codex and Claude without changing the
  default Soulmate-only initialization path.
- Added all-selected-destination preflight and regression coverage so Coffee
  opt-in refuses conflicts before writing any selected skill file.
- Documented a narrow Venus interoperability boundary: cross-session
  orientation stays advisory and human-narrowed, while Soulmate owns only the
  active task's bounded handoff and run evidence.

## 0.0.7

- Repair the public CI installed-package smoke test by creating its isolated
  project root before initialization.
- Route CI privacy checks through Soulmate's tested, fail-closed privacy gate
  so adversarial test fixtures are not mistaken for release leaks.

## 0.0.6

- Added resumable `run start|next|submit|inspect` lifecycle evidence with
  role-aware transitions, rework attempts, artifact hashes, drift checks,
  bounded assignment batches, and conservative project-local lock recovery.
- Added bundled native-host adapter guidance: capable hosts perform native
  spawning and review while the CLI remains provider-free and records only
  declared evidence and submitted artifacts.
- Added private `.soulmate/` run state with an ignored contents boundary and a
  repository privacy gate; the gate is a publication check, not proof of
  purging local or untracked data.
- Added portable profile audit/import with redacted findings and conservative
  zero-right defaults for reusing agent briefs outside their original project.
- Added stable, display, and native agent-name mapping without claiming that a
  host-generated thread label or ID is controlled by Soulmate.
- Added strict retention and cross-context values plus content-free local
  forgetting-attestation receipts for already-absent terminal memory items.
- `soulmate init` now installs the bundled skill into project-scoped Codex and
  Claude discovery paths; explicit `--refresh-skills` updates only marked
  Soulmate-owned copies, while npm installation alone changes no host config.

## 0.0.5

- Added role-scoped memory proposal, review, promotion, rejection, revocation,
  and expiry evidence as part of Soulmate's agent-handoff boundary.
- Kept Coffee as an optional portable preparation skill that can feed the
  existing agent `skills`, brief, and plan flow without adding a second
  subsystem.
- Added one-item append-only JSONL ledgers with hash-linked transition events
  referencing operator-owned project files without copying their contents.
- Kept the runtime dependency-free with no Coffee CLI or configuration
  subsystem, daemon, database, embeddings, telemetry, provider client,
  background expiry, or external tool execution.

## 0.0.4

- Registry onboarding and documentation release: npm installation is now the
  primary quick-start path, with the tagged GitHub source retained as a
  fallback for source-based installs.
- Kept dotagents and Codex/Claude hooks optional and advanced; no feature or
  dependency changes.
- Normalized the npm executable path so publication no longer needs manifest
  correction.

## 0.0.3

- Simplified standalone onboarding: install the pinned GitHub release, run
  `soulmate init`, and render a first brief without dotagents or hooks.
- Clarified that Node.js 20+ is the sole runtime requirement and moved dotagents,
  host hooks, and model bindings into optional or advanced integrations.
- Updated `init` next-step output to point to the standalone first-value command.

## 0.0.2

- Added explicit project-local Codex and Claude hook plan/apply/status/remove
  commands with idempotent, per-file atomic, conflict-safe settings merges.
- Added the internal `hook-run` bridge for bounded SessionStart and
  SubagentStart context; it is fail-open when Soulmate is unavailable.
- Added a pre-install hook-protocol handshake and five-second native hook
  timeout so stale commands or a stuck bridge cannot become a startup bottleneck.
- Preserved host settings, rejected malformed or escaping hook paths, and
  documented the bounded multi-file rename limitation.

## 0.0.1

- Added the dependency-free, plan-only CLI for config checks, briefs, plans,
  profiles, onboarding, and opt-in receipt verification.
- Added a valid Agent Plugins v1 portable bundle with role-specific skills;
  preserved SessionStart/SubagentStart host-hook resources are separated under
  a reverse-domain extension namespace and are not activated by installation.
- Added explicit runtime binding metadata without model routing, invocation,
  credentials, or implicit fallback.
- Receipts now contain selected config/profile bytes and requested runtime
  metadata only; task and goal fingerprints are not stored.
- Memory fields remain declarations of policy intent only; no storage,
  promotion, expiry, deletion, or revocation is performed.
