# landing: status

2026-11-11. Where temper's code lags or departs from [landing.md](landing.md).

Section 13.1 has no known gap in what a branch's path reads: the
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
- **Not yet:** 5.2, 15, 16: the real-process story
  `a_turn_landing_is_found_after_a_runner_cancellation_and_restarts`
  has no supplied passing result at three threads after update recovery.
  The forge world covers a lost prepare found or made again and landed,
  and a late copy's drift hold; that is not the real-Forgejo restart
  proof. The process trial is reserved for after landing.

- **Not yet:** 4.1: `with`; pin-only members; `dependencies_join`; a stack of
  members in one repository, whose second member a later landing takes
  (`temper-engine-domain-forge/src/landing.rs`). 5.3, 6.1: the project's
  prepare and key commands, as jobs (`worker/jobs.md`). 6.2: the baseline
  at main's tip; a failed receipt rerun first when its tests match open
  flake findings (`findings.md`, section 5). 8, 11, 16: GitHub's turn, its
  ref fast-forwarded (kind 19), a squashed member's pull request closed
  there, and its stories on the fake GitHub. 13.1: kinds 4 and 5 as
  settings. 13.2: adoption's checks under `turn` of each gate's workflow
  and its triggers, and of the forge's autodetect and allowance of a
  manual merge. 14: `repository receipts`.
- **Departs:** 8, 13.1: `turn` is refused for every repository on GitHub
  (`temper-engine-domain-forge/src/landing.rs`, `project_landing`), not only
  where temper cannot fast-forward main, because GitHub's turn is not built.
