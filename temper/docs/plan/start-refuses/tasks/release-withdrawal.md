+++
id = "release-withdrawal"
part = "release-withdrawal"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/routing/children.rs",
  "jig/crates/jig-domain-people/src/domain/start.rs",
  "jig/tests/core/tests/suite/settings_proposals.rs",
]
effort = "high"
minutes = 120
+++

# A release at a start withdraws its stale proposals without a panic

## Goal

A restart with a run's pending settings proposal and changed defaults crash-loops: the release's withdrawal reaches people's `assert_projection_event` at `Phase::Link`. Make the start finish, with the withdrawal in the release's own decision, as `jig/docs/design/domain/engine.md`, 17.7 now says.

The design settles the choice the request left open. The withdrawal does **not** move to a later commit: 17.6 has it in the decision that makes the revision, and with a commit of its own, a crash between the two would leave a proposal pending against an older revision that no later start withdraws.

Acceptance:

- A failing test first: story 1 below, run on the old code, panics with `people projections require start_project before open`. The report quotes it.
- All four stories pass, and each ends in the restart oracle.
- At `Phase::Link` no activation or relay leaves the core: at the `Phase::Switch` of the crash story, `world.assignments` is what it was before the restart.
- A proposer left due gets exactly one next run after the start.
- People still panic on a projection event while `Start::Loading`.
- `jig/docs/design/domain/engine.status.md`: its "17.7: a release's withdrawal" **Not yet** entry is deleted. **This departs from the implementer's text,** for that entry alone.

## Inputs

- `temper/docs/plan/start-refuses/notes/common.md`.
- `temper/docs/plan/start-refuses/inputs/request.md`: the backtrace.
- `jig/docs/design/domain/engine.md`, 6 (*The start's interface*, *After an upgrade*), 17.6 (*Stale is withdrawn*), 17.7 (the paragraph after "committed before the core admits anything").
- `skein/docs/foundation/testing-strategy.md`, section 1.

## Owns and may change

Owns the files in `files`.

May also change:

- `jig/tests/core/tests/settings/support.rs`: helpers the stories share, added only.
- `jig/crates/jig-domain-people/src/tests/start.rs`: only if a row there asserts a panic at `Start::Linked`. `projection_phase_and_operational_doors_assert`'s rows run while loading, so expect none. Another draft adds lines to this file: touch as little as you can.
- `jig/docs/design/domain/engine.status.md`: the one entry above.

**Leave `jig/crates/jig-domain-core/src/routing/settings/dispatch.rs` alone.** Another draft rewrites `withdraw_pending`'s lines, and this fix needs none of them; that is why the gate sits in `append_tasks`.

Options, pre-approved:

- `routing::children::append_tasks` is reached at `Phase::Link` only from the release's withdrawal (`start_link` steps the tasks child itself). If you find another caller at `Link`, stop and ask.
- Another `tasks::Request` that would act before the Wires comes out of a withdrawal at `Link`: if the start makes it again from the durable records, drop it by the same rule; if not, push it to `core.start.waiting_tasks`, as `start_link` does. Name it in the report.
- Behind the panic, another assertion fires on the same path (`save::save`'s `touch_*`, `public::follow`, `projections::save`). A one-line fix where it fires is yours; name it in the report. One that would change a stored format, a protocol or a public interface is a `Question:`.
- The design's "relayed after that run's next turn" (`routing/run/turns.rs`) proves false in the story: report it, with what does relay it. The design is the lead's.
- Story 2 can't reach a claimed run through `proposing_run` and the world's workers (`jig_core_world::faults::workers`): cover the dropped `Relay` with a unit test of `release_outputs` in `jig-domain-core`'s tests, and say so.
- A story whose claim the start settles opts out of the oracle as its neighbours do, with `world.broken()` and the same named finding (`jig/tests/core/tests/suite/settings.rs`, `a_release_at_a_restart_keeps_the_old_default_for_the_revision_held`). No new reason to opt out.

## Steps

1. Write story 1 in `settings_proposals.rs` and run it on the old code:

   ```rust
   #[test]
   fn a_release_withdraws_a_parked_runs_proposal_and_tells_its_task() {
       let (mut world, signed, task) = proposing_run(true);
       // run_propose, run_proposed; the run parks (world.park_for_message), as the stories in settings.rs do.
       // restart(&mut world, config), run_configuration's with a charter default changed.
       // revision(&world, 2, false).head.by == Decider::Release.
       // The set's row is gone: !rows.contains_key(&task_set(task, proposal)).
       // run_outcome(&world, task, proposal) is SettingsDecision { outcome: Stale { newest: 2 }, .. }.
       // The task's next run is assigned once.
       // Restart again with the same config: newest stays 2, the word is there once.
   }
   ```

2. The fix, in two places:

   ```rust
   // jig-domain-people/src/domain/start.rs
   /// An event's projection lands on a whole census: from the link on
   /// (jig/docs/design/domain/engine.md, 17.7), never while loading.
   pub(super) fn assert_projection_event(&self) {
       assert!(
           self.ready() || self.start == Start::Linked,
           "people projections require start_project before the link"
       );
   }

   // jig-domain-core/src/routing/children.rs, in append_tasks, after step_tasks
   if core.start_phase() == Some(crate::start::Phase::Link) {
       child = release_outputs(child);
   }

   /// The release's withdrawals keep their records in its decision. A run's activation or
   /// relay is the start's to make from those records, after the durable Wires
   /// (jig/docs/design/domain/engine.md, 17.7).
   fn release_outputs(child: Queue<tasks::Request>) -> Queue<tasks::Request>
   ```

   Compare `Phase::Link` exactly: `start_phase()` stays `Some` through `AdoptRuns`, `ReadAfresh`, `SettleOutbox` and `Open`, where `append_tasks` serves runtime routes that must keep their activations. The `match` in `release_outputs` follows `start_link`'s: a scoped `#[expect(clippy::wildcard_enum_match_arm, reason = ...)]`.

   Add a column to the table in `domain/start.rs`'s test: a projection is taken at `NotStarted`, `Linked` and `Open`; `ready()` and `start_lookup()` stay as they are.

3. The other stories, beside the first:

   ```rust
   #[test]
   fn a_release_withdraws_the_proposal_of_a_run_still_claimed() {
       // The run stays claimed through the restart. No panic; the word is in the task's inbox,
       // and the run reads it, by a relay after its next turn or by its next attempt's brief.
   }

   #[test]
   fn a_release_with_a_held_revision_withdraws_a_runs_proposal() {
       // A goal on revision 1, held (held_run's pattern in settings.rs); an owner's change makes 2;
       // a run's proposal against 2; a release makes 3. The inventory's held rows name their
       // holders, the old goal runs on its version, and a restart again changes nothing.
   }

   #[test]
   fn a_crash_after_the_releases_commit_leaves_the_withdrawal_done() {
       // world.hold_switch = true, then world.restart_with(changed config), not support's restart,
       // which asserts ready: the start stops at Switch, its release durable.
       // world.assignments is unchanged. The store holds revision 2,
       // no set, and the word in the task's live row.
       // Restart with the same config (the crash): no third revision, the word there once,
       // Deployment.messages at or past its number, and the run's next run assigned once.
   }
   ```

   `pending_reviews_survive_restart_and_a_release_withdraws_them` (a party's proposal, which has no task row, so it reaches no people event) and `changes_proposals_and_changed_defaults_keep_both_owners_in_step` (`application_settings.rs`: the connector's values dropped in the decision) keep passing unchanged.

4. Break each rule once by hand and see its test fail for that reason: `release_outputs` keeping `Activate`; keeping `Relay`; `assert_projection_event` as it was; the `Phase::Link` comparison widened to any start phase.

5. Run, each through `heavy`:
   - `heavy cargo nextest run -p jig-domain-people --lib`
   - `heavy cargo nextest run -p jig-domain-core --lib`
   - `heavy cargo nextest run -p jig-core-world`

   **This changes what a start does, so its readers' tests run too:**
   - `heavy cargo nextest run -p temper-engine-domain --lib`
   - `heavy cargo nextest run -p temper-engine-domain-world`

6. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p jig-domain-people -p jig-domain-core -p jig-core-world --all-targets -- -D warnings`. Commit and push.

## Report

- The panic on the old code, quoted.
- Each rule broken and the failure seen.
- Every `tasks::Request` you saw come out of a withdrawal at `Link`, and what became of it.
- Any assertion you met behind the first, and its fix.
