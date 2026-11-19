# landing: status

2026-11-19. Where temper's code lags or departs from [landing.md](landing.md).



`landing-settings.md`, section 1.1 has no known gap in what a branch's path reads: the
project's `by`, and each repository's gates and tiers with the branch
each follows, as the forge holds them (`forge/change/branch-path.md`).

- **Not yet:** 8: the turn's merge of a member reads no comparison of
  main's tip and its candidate, so a rule by path covers every member.
- **Not yet:** 4.2 (*Drained*): a landing in a draining hold’s scope
  going on through the turn once proving, idle and held while preparing.
  It waits on jig’s drain and project-scoped placed holds
  (`jig/docs/design/domain/tasks/lifecycle.status.md`, 10).
- **Not yet:** 9 (*A due step leaves its change out*, *Each train on
  its own*): a waiting change whose step is due stops its train's
  forming instead of being left out of it. [in flight: change 8933]
- **Not yet:** 9 (*Changes made ready by one event*, *One step at
  most*): every change not yet ready whose step is due holds its
  train's forming for as long as it stays due: one still checking,
  gating with a gate outstanding or kept by a pending dependency, as
  well as one whose due step will make it ready; and that hold is not
  limited to one step.
- **Not yet:** 7.1, by plan `forge-polls-quiet`: a landing polls its
  runs in flight every `poll`, not at the client's `poll_at`.

- **Not yet:** 4.1: `with`; pin-only members; `dependencies_join`
  (`temper-engine-domain-forge/src/landing.rs`). 5.3, 6.1: the project's
  prepare and key commands, as jobs (`worker/jobs.md`). 6.2: the baseline
  at main's tip. 8, 11, 16: GitHub's turn, its
  ref fast-forwarded (kind 19), a squashed member's pull request closed
  there, and its stories on the fake GitHub. 14: `repository receipts`.
- **Departs:** 8: `turn` is refused for every repository on GitHub
  (`temper-engine-domain-forge/src/landing.rs`, `project_landing`), not only
  where temper cannot fast-forward main, because GitHub's turn is not built.
