# authority: status

2026-11-18. Where jig's code lags or departs from [authority.md](authority.md).


- **Not yet:** 7: a standing agent task's budget renewed by period (`tasks/standing.md`, section 4).
- **Not yet:** 7, 8.3: a delegating task whose half is at or below the deployment's minimum for a run takes what it has left, capped, for its run's share; `run_spend` still gives it half [in flight: change 15]
- **Not yet:** 8.3: a run short of its budget waits while something may still free one, such as a live delegate's unspent reservation; the core holds its task for its budget at once (`runs/state.rs`, `denied`), and holds it so the same where it may delegate.

Section 3 has no known gap in the asking family, bit 2,048, or
`ToolKind::Ask` admission. A call without that family is beyond authority.
Section 3's *Notes* has no known gap in a party's scope: no authority's
note scopes name it, and the asking family reaches it for the task's
requester alone (`engine/briefs.status.md`, 2.1).

Section 3's model bits retain the catalogue setting numbers. Section 8.3's
host evidence uses endpoint window and output bounds, without a model list.

Section 6.1: a person's task in project 0 draws on their pool, has no
tools or delegation, is watched by its payer and is absent from the
person's lists. Its restoration leaves cancellation to the application.
`PersonSeed::models()` exposes its allowed models, and `Event::PersonTask`
passes its optional `llm` into the task's ordinary admission and run
checks. No choice keeps the default selection; a choice can hold it to
one account.

Section 8.4 judges a party's widening whole, by the authority it leaves
the task, and that is what took 11-17's amendment past its maker's
direct bounds: a project owner's widening of another party's goal,
which ctl restates whole (`temper/crates/temper-client-domain-forge/src/amend.rs`,
`Draft::of`). The offer's ceiling covered it, so the owner's pool had
the goal's whole new budget left. What its first bound lacked was one
of two fields of role 0's authority under the stock policy
(`temper/crates/temper-engine-service-stock/src/stock.rs`, `project_role`):
the goal's delegation kinds, since a goal a holder of role 1 or 2 makes
through a template keeps that role's kind (`jig/crates/jig-client/src/shapes.rs`,
`fill`) and each role delegates to its own alone; or its budget, past
role 0's spend for a task, one chat's, where the ceiling allows a
pool's. The repository can't say which: the proposal's record shows
the authority before and after, once `temper/docs/design/draft/proposal-reach.md`
makes it readable. Either is beyond the owner's role, not their
funding, so it is still proposed, and role 0's acceptance of it is
judged by the same role (`jig/crates/jig-domain-authority/src/check.rs`,
`give`).

## Gaps and departures

- **Not yet:** `ui-ask-m1`'s `proposal-read` part was never designed,
  according to the retirement brief. Its landed run proposal read needs
  design coverage reconciliation against `jig/docs/design/domain/engine/executors.md`,
  3; this is a design gap, not an absent implementation.
- **Not yet:** 6.3: a task's goal made by its application, `RepairSeed::goal` and `SeededGoal` [in flight: change 10349].
- **Design departs:** 6.3, decided 2026-11-19 for red repairs: the tasks
  child doesn't check a seeded goal's `authority` within the task's, since
  it judges no root-authorized authority (`jig/crates/jig-domain-tasks/src/batch.rs`)
  and a delegate's inclusion, with its connectors', is the core's. The
  application narrows the task's own, as temper's `goal` tool does
  (`temper/crates/temper-engine-domain/src/route/goal.rs`); `RepairSeed::goal`
  checks its money, room and depth, and `Make` its closer's kind and `llm`
  [in flight: change 10349].


