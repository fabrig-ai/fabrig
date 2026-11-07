# engine-causes: waiting for the ranked baseline

2026-11-07 UTC, inspected commit `fb2a487033` after fetching origin and
merging `origin/limits-worlds/figures-before` (already up to date at
`5d87b6ef10`). No profiling run or product change was made by this task.

The prerequisite's `temper/docs/plan/limits-worlds/figures/figures-before.md`
records no completed native or Callgrind measurement. It has no inclusive
or self instruction tables and no *Causes, largest first* list. This is an
unobserved baseline, not evidence that no shared engine cause exists or
that the next cause falls below the 2% stop rule.

The answer in `temper/docs/plan/limits-worlds/tasks/figures-before.md`,
under *Answers*, already permits Python's `resource.RUSAGE_CHILDREN` in
place of GNU time for this plan. The prerequisite's figures have not yet
been updated with measurements using that answer.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|
| `temper-real-loop-world`: `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` | before | — | — | — | Prerequisite measurement unobserved |
| `temper-real-loop-world`: `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` | before | — | — | — | Prerequisite measurement unobserved |
| `temper-real-loop-world`: `github_land::a_chats_change_lands_on_connected_github` | before | — | — | — | Prerequisite measurement unobserved |
| `temper-conformance-world`: `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` | before | — | — | — | Prerequisite measurement unobserved |
| `temper-one-process-world`: `public::every_inaccessible_identifier_is_one_unknown_byte_for_byte` | before | — | — | — | Prerequisite measurement unobserved |

## Left

`figures-before` owns the missing profiles and ranked cause list. Once
those figures are pushed, merge that branch and take this task's assigned
causes largest first. No cause was fixed, rejected, or assigned a share
without a profile.

Stopped before cause selection: next cause unknown; share unknown.
