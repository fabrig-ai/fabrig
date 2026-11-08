# ignored-misc: six ignored tests whose causes are product gaps

testing-strategy 8 says an ignored test is a finding its owner runs until it's fixed. Six of temper's ignored tests each name a gap in the product, not in a test world. This plan fixes each gap at its cause, by the design the reason cites, and the tests run again.

## What's wanted

**Each test below loses its `#[ignore]` and passes in the suite that selects it.** If a gap turns out to be a feature larger than a task, the test stays ignored. Its reason then names the missing design section, and the plan's notes say so. A test whose design has moved is rewritten to the design, never deleted.

1. **A restore keeps retained state across a restart.** Two tests:
   - `temper/tests/engine/domain/tests/suite/settings/kinds.rs`: `a_restore_leaves_out_a_deleted_projects_rule`. A reset to an absent application default loses its retained state at restart (`jig/docs/design/domain/engine.md`, 17.5 and 17.10).
   - `temper/tests/client/tests/suite/settings_tools.rs`: `each_kind_through_ctl_proposed_shown_accepted_and_restored`. A restored shared project default loses it too (`temper/docs/design/protocol/configuration.md`, 13.2 and 13.4).

   Find where a restart rebuilds settings from the journal and drops what a restore retained. Fix it there, once, for both.
2. **Story A walks in the one-process world.** Two tests:
   - `temper/tests/one-process/tests/suite/story_a.rs`: `story_a_is_walked_from_the_welcome_to_the_csv_change_landed`. It stalls at Starting after 5 s, with forge 1 Kept and the account and project absent.
   - `temper/tests/one-process/tests/memory.rs`: `story_a_at_its_high_water`. It has no high water to bound until the story walks (`temper/docs/design/playground.status.md`, 9).

   Find why the box's start doesn't finish in that world, whether in the world's wiring or the product, and fix the cause. The memory check's bound then follows `temper/docs/development/playground-memory.md`.
3. **A box starts without DNS.** `temper/crates/temper/tests/process.rs`: `a_default_box_with_a_sign_in_registration_starts_without_dns`. A start without DNS fails while the sign-in endpoints resolve at start. Resolution moves to where the endpoint is first used, or a failed resolution at start stops being fatal, whichever the design for sign-in registration states (`temper/docs/design/protocol/sign-in.md`).
4. **A draining worker leaves placement before its runs end.** `temper/crates/temper/tests/process.rs`: `a_restarted_workstation_keeps_the_directors_question_and_answers_it_on`.
   - Today a retry placed on a stopping worker's still-linked channel is rechecked and journaled as an extra refused attempt: 3 times in 85 runs, all under load.
   - jig takes a worker out of placement as it starts to drain.
   - jig's fleet design states it. Add the sentence if it doesn't.

## Limits

- No budget, timeout or tier changes. No other ignored test is touched.
- Process stories that change are run by name and alone to see them pass, as this repairs findings. Nothing else outside focused and fuzzy runs.
- `walk = false`. Effort medium. Four tasks, as numbered.

## Sources

- `temper/tests/one-process/tests/suite/story_a.rs`
- `temper/tests/one-process/tests/memory.rs`
- `temper/docs/design/playground.status.md`

Read by section:

- `jig/docs/design/domain/engine.md`, 17.5 and 17.10
- `temper/docs/design/protocol/configuration.md`, 13
- `temper/docs/design/protocol/sign-in.md`
- `temper/crates/temper/tests/process.rs`, the two stories named

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands.** The exception is the stories named above, each by name and alone. Never `tools/tiers/run`, nor a browser or live tier. No walk, render check or throwaway driver.
- **Decide, don't ask.** A small edit just outside a task's file list is made, and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
