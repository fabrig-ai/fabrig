# agent: status

2026-11-04. Where temper's code lags or departs from [agent.md](agent.md).
The remaining gaps concern a run's settings reads and result delivery.
Change-refusal wording also lags.

- **Not yet:** 4.4: a verdict's follow-up tasks. 8.1: a run's `settings` read of a project's own policy: jig's core answers it unknown (`jig-domain-core/src/reads/settings.rs`), since jig's `setting` read names jig's kinds and an application's alone (jig's `client/domain/contract.md`, 5.1), so the configuring agent proposes a project's policy without reading it first.
- **Not yet:** 4.3.1: a change's refusal omits what authority it lacked and
  which model its charter refused, because
  `temper-engine-smith/src/answers.rs` renders `Authority needs proposal`
  and `model outside the charter` without those details although the typed
  refusal retains them. No reason is recorded.
