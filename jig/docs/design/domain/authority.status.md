# authority: status

2026-11-19. Where jig's code lags or departs from [authority.md](authority.md).

Section 3 has no known gap in the asking family, bit 2,048, or
`ToolKind::Ask` admission. A call without that family is beyond authority.
Section 3's *Notes* has no known gap in a party's scope: no authority's
note scopes name it, and the asking family reaches it for the task's
requester alone (`engine/briefs.status.md`, 2.1).

Section 3's model bits retain the catalogue setting numbers.

Section 6.1: a person's task in project 0 draws on their pool, has no
tools or delegation, is watched by its payer and is absent from the
person's lists. Its restoration leaves cancellation to the application.
`PersonSeed::models()` exposes its allowed models, and `Event::PersonTask`
passes its optional `llm` into the task's ordinary admission and run
checks. No choice keeps the default selection; a choice can hold it to
one account.

## Gaps and departures

- **Not yet:** `ui-ask-m1`'s `proposal-read` part was never designed,
  according to the retirement brief. Its landed run proposal read needs
  design coverage reconciliation against `jig/docs/design/domain/engine/executors-tools.md`,
  1; this is a design gap, not an absent implementation.
- **Design departs:** 6.3, decided 2026-11-19 for red repairs: the tasks
  child doesn't check a seeded goal's `authority` within the task's, since
  it judges no root-authorized authority (`jig/crates/jig-domain-tasks/src/batch.rs`)
  and a delegate's inclusion, with its connectors', is the core's. The
  application narrows the task's own, as temper's `goal` tool does
  (`temper/crates/temper-engine-domain/src/route/goal.rs`); `RepairSeed::goal`
  checks its money, room and depth, and `Make` its closer's kind and `llm`.
