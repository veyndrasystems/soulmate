---
name: coffee
description: Prepare a non-trivial coding task before implementation by selecting useful context methods and returning a bounded readiness brief; skip clear, small requests.
---

<!-- soulmate-managed-skill:v1 -->

Use Coffee as a short, fail-open preparation step, not as a mandatory interview
or execution system.

- Inspect only enough project context to decide whether preparation adds value.
  A clear, bounded request should proceed without ceremony.
- Use already available and independently authorized capabilities. Examples
  include Repomix for a broad repository view, CodeGraph for cross-module
  relationships, Ponytail-style review before adding features, dependencies,
  or abstractions, and a Grill-Me-style interview for material ambiguity.
  Minimization does not decide verification or data representation: keep the
  current agent's Soul and apply the `qa-engineer` perspective from
  `role-perspectives` for the smallest useful evidence set, or its
  `implementation-planner` perspective when internal responsibilities, state,
  or invariants need a bounded slice. These examples are not dependencies or a
  universal recipe. Treat Ponytail's "one runnable check" as a minimum rather
  than a cap when distinct claims need distinct evidence. Respect explicit-only
  invocation rules.
- Research facts with host tools. Ask the user only for decisions that change
  the intended outcome. Preserve intentionally ambitious or experimental goals
  while challenging accidental complexity.
- When Soulmate is configured, inspect the selected agent's declared `skills`
  as intent, then return a readiness brief containing the goal, useful context,
  unresolved decisions, recommended agent/skills, scope boundaries, and
  verification. Feed that brief into the existing `soulmate brief` or
  `soulmate plan` flow; do not create a second recipe or orchestration schema.
- When the brief recommends a subagent, name its context mode. Implementation
  agents `sonic`, `worker`, and `default` may use bounded inherited task turns.
  Reviewers, advisers, auditors, and claim-scoped verifiers use
  `fork_turns="none"` plus a complete packet containing the claim, scope,
  authority boundary, allowed evidence, and output contract. A Ponytail
  subagent matcher filters hook injection; it does not remove modes or history
  inherited from the parent conversation.
- Native hosts own tools, models, subagents, and permissions. Coffee grants no
  execution authority and must not block work when unavailable.

If Venus provides cross-session orientation, use it only as advisory evidence.
A human must accept and narrow the Goal before handing it to Soulmate. Do not
auto-ingest raw session excerpts into Soulmate memory; repository docs and ADRs
remain architecture truth. Venus owns cross-session orientation, while
Soulmate owns the active task's bounded handoff and run evidence.
