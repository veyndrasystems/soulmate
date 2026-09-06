# Value baseline

It verifies what you asked an agent to do and what came back, in the same
record.

## Baseline being compared

The baseline is public commit
[`2773dfc`](https://github.com/veyndrasystems/soulmate/commit/2773dfc98e07d59d2a1763cdc02c646d296325f9),
whose product behavior matches the preceding 0.11.0 release. The intervening
changes affect contributor release checks and documentation.

Source inspection establishes these existing boundaries:

| Question | Existing evidence | Classification |
| --- | --- | --- |
| Can a worker record completion? | `run submit` records a role-scoped `completed` event. | Control-plane evidence |
| Does that prove tests ran or passed? | No command-result field exists in run-v1/v2; receipts keep `runtime.observed` null. | Observation gap |
| Is reviewer approval already separate from acceptance? | `run_state` assigns different outcomes and reserves acceptance for the lead. | Existing protection |
| Are previous artifacts protected from drift? | `run_artifact::assert_current` checks recorded hashes before mutation. | Existing protection |
| Are rework attempts distinguishable? | Assignment paths include stage/attempt; rework advances the attempt while retaining old submissions. | Existing protection |
| Does a declaration prove all writes stayed in scope? | The current declared boundary is not a measured full-project write set. | Execution limitation |

The source insertion points are [run coordination](../src/run.rs),
[canonical state reduction](../src/run_state.rs),
[artifact checks](../src/run_artifact.rs), and
[receipt validation](../src/receipt.rs). New observation evidence belongs beside
run transitions; changing receipt assertions into execution proof would erase
an existing distinction.

## Controlled comparison

The [versioned scenario](../proof/scenarios/false-completion-v1/) retains a
competent ordinary baseline: a deterministic command returns a nonzero exit
status that its caller can see. The caller can already use that status to stop.
The baseline workflow can nevertheless record worker completion and later lead
acceptance, because it has no configured check result to validate.

The checked path adds an explicit policy and binds the supplied result to the
current worker submission. It refuses acceptance after a reported failure,
retains a factual refusal record, then permits recovery through a fresh attempt.
The previous attempt remains inspectable.

This is a deliberately seeded comparison. It does not measure how often Codex,
Claude, or another host ignores failures in ordinary use. No transcript is
needed to reproduce either path. Actual host behavior, recurring confusion,
user-confirmed value and false-positive frequency require separate observations.

## Recorded baseline sample

The [machine-readable baseline](../proof/baselines/checked-acceptance-baseline.json)
records one controlled local debug-build run:

| Observation | Result |
| --- | --- |
| External command exit available to caller | 1 (failure) |
| Canonical result after the four role submissions | accepted |
| Prior artifact hashes | unchanged |
| Existing artifact-drift demo | passed |
| CLI invocations, including orientation and inspection | 12 |
| Manual JSON edits to declare the command | 1 |
| Automated execution elapsed | 702 ms |
| Human interaction time | unmeasured |

This single sample is not a minimum command count or a latency distribution.
The binary hash identifies a local debug build; it does not assert byte identity
with the published release asset. The JSON edit declared the command in the
worker's existing command list, which is a permission declaration rather than
an executed check policy. The scenario intentionally continued after the known
failure to expose the acceptance-record gap.

## What to measure next

Keep future observations in distinct categories:

- Deterministic fixture: command exits, canonical transitions, preserved hashes,
  command counts and automated elapsed time.
- Local host report: which result was supplied, for which exact target, and what
  Soulmate recorded or refused.
- User observation: time spent, whether the explanation was useful, and whether
  an avoided consequence or false positive was explicitly confirmed.

The fixture's automated elapsed time is not active user time. A reproducible
command count is not a novice-completion rate. Until real participants provide
those measurements, onboarding speed and broad recurring-loss claims remain
unestablished.

The proposed next investigations are ordinary host workflows, real use on
multiple active projects, and first use by people who have not read the
architecture. Automatic hooks, concurrency guards and memory automation should
follow the incident evidence they are meant to address. They are not prerequisites
for reproducing this first scoped acceptance check.

See [the proof methodology](value-proof-methodology.md) for source labels,
privacy, compatibility and interpretation limits.
