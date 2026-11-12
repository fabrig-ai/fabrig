# landing: status

2026-11-11. Where temper's code lags or departs from [landing.md](landing.md).

Section 13.1 has no known gap in what a branch's path reads: the
project's `by`, and each repository's gates and tiers with the branch
each follows, as the forge holds them (`forge/change/branch-path.md`).

- **Not yet:** 4.1, 9, 12: one landing per train, formed level-triggered
  from the waiting changes in the queue's order; a unit of more than one
  change refused by a hold naming its changes; sealed landings; the
  train's branch on `Landing` (`temper-engine-domain-forge/src/landing.rs`,
  `forge_route/landing_route.rs`: a landing forms per ready change, joins
  a cross-repository closure, and waits for one project-wide turn)
  [in flight: change 824].
- **Not yet:** 6.1, 7.1, 13.1: a gate's `base`, key version 2 and
  `temper_base`; `gate_timeout` and `gate_retries` from the project's
  settings (`gate_retries` is the literal 2) [in flight: change 825].
- **Not yet:** 3 (landed at its prefix), 5.1, 5.2, 8, 10, 11, 13.1
  (`combine`), 15: rounds on the landing's branch `temper/<root>/l<task>`
  prepared by the worker's prepare job, merged or squashed; the round's
  pull request fast-forwarded; a squashed member's pull request recorded
  merged; the purpose's opening, closing, deletion and record operations
  (the change rebases its own branch, kind 11, and the member's pull
  request is fast-forwarded) [in flight: change 830].
- **Not yet:** 3 (judging at its prefix), 4.2 (holds, news, drained),
  4.3, 7.1 (*Bounded*), 7.3: judgement at the prefix, leaving with its
  reasons and its place kept, the holds past each bound, and the goal's
  news at each step [in flight: change 833].
- **Not yet:** 14: `landing show` and `landing list` with the train,
  round, branch, candidate and combining; the changes view per train; a
  change's panel naming its landing and way [in flight: change 834].
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
