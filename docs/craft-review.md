# What the craft review changed

This review separates gaps in the teaching and installation experience from
limitations the protocol already states and checks. Its fixed baseline is
`243d7817f37a12c208db0bbca5da291a6d16bf12` (package 0.12.1-rc.2).
The changes from that review shipped in `v0.12.1-rc.3` from source snapshot
`4199ebd`. Platform evidence is summarized in the [platform support details](platform-support.md) and [public release workflow](https://github.com/veyndrasystems/soulmate/actions/runs/34094355184).

## Six findings against the baseline

| Review concern | Baseline finding | Response |
| --- | --- | --- |
| First-run vocabulary | Partly confirmed. The setup-free benchmark and four-command core existed, but the real task setup required more explanation. | Keep the existing first experiment; teach the next change in onboarding, and put recovery and optional features behind separate links. No novice usability study is claimed. |
| Platform support | Missing Mac release artifacts confirmed. Linux/WSL support was already explicit, and unsupported platforms already failed before archive retrieval. | Add native Apple Silicon and Intel build/package/install gates plus durable installer regressions. Source builds and modeled host tests do not substitute for native public installation. |
| Historical formats | Reader navigation was missing. The private/public history boundary was already disclosed; it was not hidden ancestry. | Add the [public tag and format-reader map](../CHANGELOG.md#public-tags-and-format-readers), preserving unpublished archives and existing tags. Tagged source support is distinguished from old-binary execution. |
| Who checked the work | Default output already separated claim/check/review/acceptance, but omitted useful command, worker, and lead-outcome detail. | Render the current workers, host-reported check and targets, review, and actual lead decision. A refused transition is not a lead rejection. Preserve machine JSON and check ownership. |
| Teaching layers | Confirmed as a navigation problem; precise reference material remains necessary. | Separate [next change](first-checked-run.md), [repair](repair-a-run.md), and [optional features](optional-surfaces.md). Keep flags and formats in the canonical reference. |
| Skill/binary drift | Missing diagnosis confirmed. Safe explicit refresh already existed, and the preview skill already described the stable JSON fallback. | Compare installed managed bytes with this binary's embedded skill; report version, hashes and an explicit refresh instruction. A difference does not establish age or incompatibility. Old distributed binaries cannot gain new diagnostics retroactively. |

At the pinned baseline, the corresponding owning locations are
`src/cli.rs:747`, `src/run_value.rs:461`, `src/project_commands.rs:126`,
`src/project_skills.rs:84`, `install.sh:5`, and `docs/public-history.md:8`.
For example, inspect a historical source file with
`git show 243d781:src/cli.rs`; later line numbers may differ.

## Additional proposals that need a different decision

The local-owner limitation was already explicit in [SECURITY.md](../SECURITY.md):
an actor that rewrites all local evidence can recompute an unsigned chain.
Adding another disclosure paragraph would not supply external authentication.

The existing [false-completion scenario](../proof/scenarios/false-completion-v1/expected.json)
already requires the repaired, passing, reviewed attempt to be accepted.
Therefore it is not exclusively a failing-case test. It still does not measure
wrongful-refusal frequency in ordinary use.

Running a frozen command inside Soulmate would change the documented
[execution boundary](../REFERENCE.md#authority-boundary). Existing Git or tmux
subprocesses do not establish that product decision. Likewise, a commit ID and
dirty flag would not identify all dirty file bytes or authenticate a check.
No new executor, evidence origin, signing scheme, or MCP surface is included.

Coffee is optional in project initialization; its references to available
preparation tools do not make those tools installation dependencies. Host
marketplace discovery and activation need their own exercised integration
contract. No untested marketplace file or new required runtime is added here.

## The value question remains separate

The [matched Holytail workload](holytail-workload-findings.md) produced correct
changes with both workflows. It exposed extra recording and host-name repair
work; it did not demonstrate installation payback. The follow-on improvement
must reduce that burden while retaining a useful enforced acceptance
constraint. Lower command counts, successful CI, and a clearer README do not
by themselves establish total human savings or a reason for every builder to
install another tool.
