+++
id = "restart-handover"
part = "handover"
kind = "feature"
needs = ["spend-template"]
files = [
  "temper/crates/temper-service/src/setup/lifecycle.rs",
  "temper/crates/temper-service/src/setup/runtime.rs",
  "temper/crates/temper-service/src/tests/setup_lifecycle.rs",
  "temper/crates/temper/src/tests/box_kinds.rs",
  "temper/crates/temper/tests/suite/startover.rs",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 150
+++

# A restarted box takes its handover, and the five stories pass

## Goal

A box that signs its person in starts `temper serve` before its restored owned group, because preparation's source commands and Forgejo's loading of its source fetch discovery from temper. Serve reads the handover once, at its start, before preparation has made it, so `forge-N/admin_token` stays on disk after every restart of that default box, and temper's older admin tokens stay valid. Serve keeps starting first; it reads the handover once more, after the first `Reconcile` has waited for Forgejo.

This departs from the implementer's text and from a spec's usual rule, on the request's own word: **these five stories, and only these, run by name, alone, through `heavy`, and the report gives each one's result** (`temper/docs/plan/ui-restart-red/inputs/request.md`, *How*). Never the whole tier and no other story, and `temper::live` isn't run. A status line is yours, as acceptance 7 says.

Acceptance:

1. **What happens, proved first.** Before the fix, run the plan story once (Steps, 1). The report says what it found: the failure's line, the box log's `box link: dropped started` line, and whether the forge reads `Connected as temper` while `forge-1/admin_token` exists (`stat` only: never read a token's bytes).
2. **The handover is taken.** On a restart of a box whose `application.machine.owned.sign_in` is set, for a forge that start restored: once its first `Reconcile` is done, before the first repository or the connect, the host reads `admin_token` once more. A file there goes through `TakeOver`, removal and `Reconcile`, then on to the connect; with none, the forge goes on with no second `Reconcile`. The row stays connecting throughout, so the forge reads connected only with the handover gone.
3. **Nothing else changes.** A box with no sign-in, a `temper serve` with no owned sign-in, a fake forge and a forge made after the start read the handover as they do now. Serve's order in the box doesn't move: `sign_in_keeps_serve_before_the_restored_group_with_first_steps` stands. A box's unasked `started N` stays named and dropped.
4. **Focused tests,** each shown red without its fix, named in `## Steps`.
5. **`startover`'s story passes** with its started cut still holding the engine across `started`, and every assertion as it stands.
6. **All five pass, each by name and alone:** `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart`, `playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge`, `a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart`, `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`, `startover::owned_setup_cuts_start_over_and_a_removal_cut_finishes_on_restart`. No assertion of theirs is weakened, least of all that no token file remains.
7. `setup.status.md`'s **Not yet** line that begins `6.5: a box's restart when it signs its person in` is gone.

## Inputs

- `temper/docs/plan/ui-restart-red/notes/common.md`
- `temper/docs/plan/ui-restart-red/inputs/request.md`
- `temper/docs/design/protocol/setup.md`, 6.5, *A box's restart, when it signs its person in*, and 7.1, *At a later start*
- `temper/docs/design/protocol/sign-in.md`, 9.9, *The source's step* and *A moved port*
- `temper/docs/design/protocol/forge.md`, 2.8, *Named and dropped*
- `temper/docs/design/domain/testing.md`, the setup-cut story, under *Owned variants share their setup*
- skein's `skein/docs/design/forge-setup.md`, 4.4, *It waits for the forge*, and 4.7

## Owns and may change

Owned: the header's `files`.

Readers and neighbours:

- `temper/crates/temper-service/src/tests/setup.rs`, `setup_adoption.rs`: a helper's visibility alone (`reconcile_bot`, `record_repository`, `restore_local`), so the new tests can drive a `Reconcile` to its end.
- `temper/crates/temper-service/src/tests/forge_sign_in.rs`, `accounts_host.rs`, `system.rs`: these configure a sign-in. A restored owned forge there now has one more read to answer: answer it with `Ok(None)`, as `restore_local_with_file` does, changing no assertion.
- `Local` is built only in `setup/runtime.rs`, `local`.

**Options, each with its default:**

- If the proof (acceptance 1) shows the forge not connecting at all, where this spec reads that it connects: the second read still comes first. Fix what stays red in the same hook and say so.
- If the red at `startover.rs:142` is a cut other than `Cut::Started`: read its log and repair that cut the same way, keeping its boundary; say so. If the started cut can hold the engine on the default box without a race, prefer that to a box with no sign-in, and say how.
- **`a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` has no known cause:** the request gives no message, and it runs on `support::configuration`'s deployment against a throwaway Forgejo, with no box and no `run_spend`, so neither cause above reaches it. Run it first (Steps, 1) and read its log. If it passes now, say so: `ui-repo-link-2` has landed beneath this task. If a stale walk or oracle trips it (`temper/crates/temper/tests/support/person.rs`, `forgejo.rs`, `configuration.rs`), repair that, keeping what it checks. If it is a product fault, fix it at the lowest tier that shows it, with a focused test shown red, in whichever crate holds it, and name both in the report.
- If `temper_forge_setup::machine::OwnedMachine` in the step world refuses `sign_in` without its client, give the fixture the provider `forge_sign_in.rs` gives its own.

## Steps

1. **Prove it.** Each story is run with:

   ```
   heavy cargo nextest run -p temper --test process --profile process-temper -E 'test(=NAME)'
   ```

   First, unchanged: the plan story, then `a_persons_change_...`. The scratch of a failed story is kept: read its `box/logs` by `grep`, and read no token.

2. **`setup/lifecycle.rs`:**

   ```rust
   /// A restored forge's read after its reconcile (protocol/setup.md, 6.5).
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub(super) enum ReadAgain {
       No,
       AfterReconcile,
       Reading,
   }
   ```

   `Local` gains `pub(super) read_again: ReadAgain`. In `kept`'s `Purpose::LocalOrigin(run)`, once the run is removed and the forge isn't ended, before `create_local`:

   ```rust
   if read_after_reconcile(engine, active.forge) {
       return;
   }
   ```

   `read_after_reconcile` turns `AfterReconcile` into `Reading`, sets `local.file = FileStep::Reading { run: next_run(..), sent: false }` and answers `true`; otherwise it answers `false`. In `file_answer`, `FileStep::Read(Ok(value))`: with `Reading`, set it to `No`, and with `value` none go on as `LocalOrigin` does (`create_local`, else `adopt_local`), not to `start_local`, so there is no second `Reconcile`; otherwise `start_local` as now takes the file over. `local_stopped` and `started` set `No`.

3. **`setup/runtime.rs`, `local`:** `read_again` is `AfterReconcile` when `!restored`, `record.mode == domain::Mode::Owned` and `self.machine.owned.as_ref().is_some_and(|owned| owned.sign_in.is_some())`, else `No`. `restored` there is false for a forge a start loads, true for one made later.

4. **Focused tests, in `temper-service/src/tests/setup_lifecycle.rs`,** beside `restored_owned_forge_reads_a_present_handover_before_using_kept_admin`, each configuring `machine.owned.sign_in` but the last, and using `restore_local` so the start's read finds nothing:

   - `a_restored_box_takes_the_handover_preparation_made_after_serve_started`: with the `Reconcile` done and its origin kept, `forge.starts` is empty and `take_handover_read` gives one more read of `/data/forge/admin_token`. Answered `Some`, the run is `LocalTakeOver`, then its removal, then `LocalReconcile`, and only then `forge.starts` holds the forge; no third read.
   - `a_restored_box_with_no_new_handover_connects_after_its_second_read`: answered `Ok(None)`, the forge starts with no second `LocalReconcile`.
   - `a_box_with_no_sign_in_reads_its_handover_once`: no second read, and the forge starts.

   The first is the red for this cause: without the fix it stops at the second read's `expect`.

5. **`suite/startover.rs`.** With sign-in on, `Cut::Started` stops serve, then waits for `box/started/N` and the log's `started`; but the group's source step, `add-oauth`, fetches discovery from that stopped serve and hangs, so neither comes and `first.checked` stays none at line 142 (`sign-in.md`, 9.9; `setup.md`, 7.1, the record follows the whole group). The started cut moves to a box with no source step, as `testing.md` now says:

   ```rust
   /// Init's document with no `openssl` (protocol/sign-in.md, 9.9): the group has no source
   /// step, which needs the engine this cut holds.
   fn without_sign_in(config: &Path) -> Vec<u8>
   ```

   It sets `engine.clients.provider` and `application.machine.owned.sign_in` to `null` in `temper.json` and returns the file's bytes as they were. `start_over` calls it for `Cut::Started` alone and writes those bytes back with `support::write_private` at its end, so the final setup and the removal cut stay on the default box.

6. **A focused test for that cause,** in `temper/crates/temper/src/tests/box_kinds.rs`: `a_box_with_no_sign_in_has_no_source_step_in_its_group`. It takes `configuration::init_document()` with those two members null, and asserts that `configuration_steps`, given no first steps, makes an owned group with no `Stage::Source` and `before` 0. Show it red by inverting `owned.sign_in.is_some()` in `temper-forge-setup/src/step.rs`, once, by hand. If a test already pins this, name it instead of adding one.

7. **Each new test must bite:** the report names the rule broken and the failure seen.

8. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service -p temper --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-service`; `heavy cargo nextest run -p temper --lib --test suite`. This changes what a restored forge's setup does, so all of `temper-service`'s tests run, not only the new ones.

9. Remove the **Not yet** line (acceptance 7). If what you find departs from `setup.md`, 6.5, say so in the report; the design's other lines aren't yours. Run `python3 -I -B tools/citations/citations.py --base origin/main`.

10. Then the five stories, each by name, alone (acceptance 6). A story's failure gets its log read before any rerun, with no unchanged rerun, except once after exit 137. Commit and push before the last ones, so the work is safe if a story needs long.

## Report

`Done:` first, or `Question:` only if a story can't be made green and you've read why. Then:

- what acceptance 1 proved, in the words of its log;
- each of the five stories, by name, with its result and wall time;
- the cause of `a_persons_change_...` and what fixed it;
- each focused test, its red and its pass;
- the helpers and fixtures you repaired;
- the branch and commit, and your implementation minutes and check minutes apart.
