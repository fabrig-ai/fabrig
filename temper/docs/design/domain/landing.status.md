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
- **Not yet:** 7.1, by plan `forge-polls-quiet`: a landing polls its
  runs in flight every `poll`, not at the client's `poll_at`.
- **Not yet:** 4.2: the landing's row, links and holds are made before
  jig answers its task's make, and stay when jig refuses it
  (`landing-forming.md`, 2, 3).
- **Not yet:** 15: a live landing jig never made refuses the start, its
  holds linking a missing task (`landing-forming.md`, 4).

- **Not yet:** 4.1: `with`; pin-only members; `dependencies_join`
  (`temper-engine-domain-forge/src/landing.rs`). 5.3, 6.1: the project's
  prepare and key commands, as jobs (`worker/jobs.md`). 6.2: the baseline
  at main's tip. 8, 11, 16: GitHub's turn, its
  ref fast-forwarded (kind 19), a squashed member's pull request closed
  there, and its stories on the fake GitHub. 14: `repository receipts`.
- **Departs:** 8: `turn` is refused for every repository on GitHub
  (`temper-engine-domain-forge/src/landing.rs`, `project_landing`), not only
  where temper cannot fast-forward main, because GitHub's turn is not built.
