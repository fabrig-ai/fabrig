# landing: status

2026-11-11. Where temper's code lags or departs from [landing.md](landing.md).

Section 13.1 has no known gap in what a branch's path reads: the
project's `by`, and each repository's gates and tiers with the branch
each follows, as the forge holds them (`forge/change/branch-path.md`).

- **Not yet:** 8: the turn's merge of a member reads no comparison of
  main's tip and its candidate, so a rule by path covers every member.
- **Not yet:** 7.1, by plan `forge-polls-quiet`: a landing polls its
  runs in flight every `poll`, not at the client's `poll_at`.

- **Not yet:** 4.1: `with`; pin-only members; a stack of
  members in one repository, whose second member a later landing takes
  (`temper-engine-domain-forge/src/landing.rs`). 5.3, 6.1: the project's
  prepare and key commands, as jobs (`worker/jobs.md`). 6.2: the baseline
  at main's tip; a failed receipt rerun first when its tests match open
  flake findings (`findings.md`, section 5). 8, 11, 16: GitHub's turn, main's
  ref fast-forwarded (kind 19), and its stories on the fake GitHub. 9: the
  reservation; stale proofs are counted and reserve nothing. 13.1: kinds 4
  and 5 as settings. 13.2: adoption's checks under `turn` of each gate's
  workflow and its triggers, and of the forge's autodetect of a manual
  merge. 14: `repository receipts`.
- **Departs:** 8, 13.1: `turn` is refused for every repository on GitHub
  (`temper-engine-domain-forge/src/landing.rs`, `project_landing`), not only
  where temper cannot fast-forward main, because GitHub's turn is not built.
