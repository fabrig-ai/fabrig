# agent: status

2026-11-05. Where temper's code lags or departs from [agent.md](agent.md).
The remaining gaps concern a run's settings reads and verdict follow-up tasks.
Change-refusal model wording also lags, and a run that cannot deliver keeps
the shell its charter grants.

- **Not yet:** 4.4: a verdict's follow-up tasks. 8.1: a run's `settings` read of a project's own policy: jig's core answers it unknown (`jig-domain-core/src/reads/settings.rs`), since jig's `setting` read names jig's kinds and an application's alone (jig's `client/domain/contract.md`, 5.1), so the configuring agent proposes a project's policy without reading it first.
- **Not yet:** 4.3.1: a change's refusal omits which model its charter
  refused, because `temper-engine-smith/src/answers.rs` renders
  `model outside the charter` without that detail although the typed
  refusal retains it. No reason is recorded.
- **Departs:** 4.3: a run that cannot deliver can still change files by a
  command if its charter grants the shell, where the section says it
  cannot: jig's rail narrows `modify` alone (jig's `domain/hosts.md`,
  section 8, and its status), and
  `an_open_run_cannot_modify_even_when_policy_requests_it` in
  `temper-engine-smith/src/tests.rs` pins the shell staying. The stock
  charters grant no shell; the reference's charter 2 does
  (`../protocol/configuration.md`, 13.9). No reason is recorded.
