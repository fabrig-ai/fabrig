# authority-checks: status

2026-11-23. Where the tree's code lags or departs from [authority-checks.md](authority-checks.md).

- **Not yet:** 1.3: a delegating task whose half is at or below the deployment's minimum for a run takes what it has left, capped, for its run's share; `run_spend` still gives it half [in flight: change 15]
- **Not yet:** 1.3: a run short of its budget waits while something may still free one, such as a live delegate's unspent reservation; the core holds its task for its budget at once (`runs/state.rs`, `denied`), and holds it so the same where it may delegate.

Section 1.3's host evidence uses endpoint window and output bounds,
without a model list.

Section 1.4 judges a party's widening whole, by the authority it leaves
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
