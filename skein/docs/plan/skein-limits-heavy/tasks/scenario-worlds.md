+++
id = "scenario-worlds"
part = "scenario-worlds"
kind = "feature"
needs = []
files = [
  "skein/tests/fail-stop/tests/suite/fail_stop.rs",
  "skein/tests/http/tests/suite/transcripts.rs",
  "skein/tests/http/tests/suite/requests.rs",
  "skein/tests/http/tests/suite/fuzzy/client.rs",
  "skein/tests/http/tests/suite/fuzzy/server.rs",
  "skein/tests/protocol/tests/suite/worlds.rs",
  "skein/tests/world/tests/suite/end_to_end.rs",
  "skein/tests/conformance/sim/src/lib.rs",
  "skein/tests/conformance/sim/tests/suite/conformance.rs",
  "skein/tests/conformance/real/src/lib.rs",
  "skein/testing/skein-conformance/src/run.rs",
  "skein/docs/plan/skein-limits-heavy/figures/scenario-worlds.md",
]
effort = "medium"
minutes = 140
+++

# Six rows of scenarios: the seeds that show a behaviour, and what a wait costs

## Goal

Six rows of 2.94 CPU s between them: `skein-conformance-real::suite` (0.76), `skein-world-tests::suite` `end_to_end` (0.69), `skein-http-world::suite` `transcripts`, `requests` (0.44), `skein-conformance-sim::suite` (0.39), `skein-fail-stop-tests::suite` (0.36), `skein-protocol-world::suite` `worlds` (0.30). `plan.md`, *Belonging*, has decided where each test belongs: two move to fuzzy, and the rest stay. Lower what's decided below, then find and fix each row's cause, with the stop rule on each.

Acceptance:

- `skein-http-world`'s `transcripts::a_transcript_cut_short_anywhere_decodes_as_the_reference_reads_its_prefix` is `fuzzy::client::a_transcript_cut_short_anywhere_decodes_as_the_reference_reads_its_prefix`, and `requests::a_request_transcript_cut_short_anywhere_comes_to_what_the_reference_reads_of_its_prefix` is `fuzzy::server::a_request_transcript_cut_short_anywhere_comes_to_what_the_reference_reads_of_its_prefix`. Each keeps its count and every check.
- `fail_stop.rs` runs its fixture three times, and every form it asserts is still asserted: three `RUST_BACKTRACE` values, three payloads, an unnamed and a named thread. `assert_report` doesn't change.
- `repeated_stderr_overflow_unwinds_without_aborting` runs three unwinds, each asserting the exact diagnostic.
- `skein/tests/conformance/sim/src/lib.rs`'s `CALM` is 4 or the smallest that shows what step 4 asks. `SMOKE`, `CHAOS` and `LATE_RESETS` stay.
- `skein-protocol-world`'s `worlds` runs the seeds of step 5, and every aggregate assertion stays, `reset > 3` among them.
- A wait that has already arrived arms no `Deadline`. `PATIENCE` and 2.10's rule stay, and `skein/tests/conformance/real/tests/suite/deadline.rs` passes.
- Each smaller count bites, as the common note asks.
- `skein/docs/plan/skein-limits-heavy/figures/scenario-worlds.md` has each row's costliest and median tests, before and after, *Counts* lines, and what fuzzy gains.
- In each row, the next cause's gain is under about 0.05 CPU s, or what's left is under *Found and not fixed*.

## Inputs

- `skein/docs/plan/skein-limits-heavy/inputs/request.md`, items 1, 3 and 4, and these rows.
- `skein/docs/plan/skein-limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `skein/docs/plan/skein-limits-heavy/plan.md`, *Belonging*.
- `skein/docs/foundation/testing-strategy.md`, 2.10 (the deadline's rule, which doesn't change), 8 and 8.1.
- `skein/docs/foundation/programming-model.md`, 2: what `fail_stop.rs` holds.

## Owns and may change

Owns the files in the header.

- **`skein/testing/skein-conformance/src/run.rs` serves real and simulated conformance:** a change there runs both, and their fuzzy tests. `each_seed`, `RACES` and the racing helpers also serve `skein/tests/conformance/sim/tests/suite/fuzzy/conformance.rs`, so they stay as they are.
- **`skein/testing/skein-world/src/deadline.rs` and `deadline/process.rs`** are one mechanism for every tree (2.10). A small change that makes `arm` or `watch_process` cheaper may be made there if the profile puts the cost there, if what a starved or failed wait reports is the same. Run `skein-world-tests` too, and check each tree's reader with `heavy cargo check --tests`. A larger one goes under *Found and not fixed*. `Wire` and `skein_heap` don't change.
- **`skein/tests/protocol/src/scenario.rs`'s generators** serve its fuzzy sweep, so they stay. Lower how many worlds a focused test runs, not what each is.

Options, each decided:

- **Which `RUST_BACKTRACE` row goes with which payload** is yours, if the set of step 2 holds. Each test keeps its name, so the comment on a row says what it adds.
- **A run of the fail-stop fixture costs what a backtrace's symbols cost:** it stays, and its count goes under *Found and not fixed*. The product's hook captures on purpose.
- **The real conformance's waits mostly arrive only after the backend's `pass`:** then reaping first saves little. Let `Real::wait_begins` take one turn that doesn't block before it arms, if the wait's bound and its report are the same. A test that holds a wait unanswered still arms.
- **`Run::start` copying each submitted box, or `transfer` recopying the source per short send:** fix a copy that nothing reads. A contract check that needs its copy keeps it.
- **`every_transcript_decodes_to_its_expectation_whatever_the_cuts` and `every_request_transcript_comes_to_its_expectation_whatever_the_pace`** stay, with their seeds and paces, unless the profile puts more than 0.05 CPU s on one. `SEEDS` is then 2, with a *Counts* line: each fixture still decodes to its golden. `every_event_stream_reads_...`'s four chunk sizes stay, because they're named.
- **`supervised_stderr_overflow_keeps_a_bounded_prefix_and_drains_a_natural_tree`'s 262,144 zeros** stay, unless you can say from the code what a pipe's capacity is there, and keep it past that with room.
- **A process test's cost is fixture processes:** they're what those tests are for. Give the count under *Found and not fixed*. What a process costs before its test starts is another plan's, and so is the protocol binary's floor of 17 ms: say what the profile shows of it, and leave it.

## Steps

1. **Profile each row before changing it,** with `--test suite`, or `--trace-children=yes` where a test starts processes:
   - `-p skein-conformance-real`: `conformance::ring::a_file_is_made_written_at_offsets_read_back_and_stated` (or as `cargo nextest list` names it), and its backpressure test.
   - `-p skein-world-tests`: `end_to_end::repeated_stderr_overflow_unwinds_without_aborting`, `end_to_end::a_binary_that_exits_at_once_settles_with_counts_above_zero`.
   - `-p skein-http-world`: the two that move, and `transcripts::every_transcript_decodes_to_its_expectation_whatever_the_cuts`.
   - `-p skein-conformance-sim`: `conformance::a_calm_world_pairs_each_race_as_the_ring_does`, `conformance::a_file_is_made_written_at_offsets_read_back_and_stated`.
   - `-p skein-fail-stop-tests`: `fail_stop::a_panic_on_a_second_thread_aborts_the_process_after_its_report`.
   - `-p skein-protocol-world`: `worlds::one_end_closing_while_the_other_sends_leaves_both_settled`, `worlds::a_seed_replays_to_the_same_run`.
2. **`fail_stop.rs`, three runs.** The hook takes one path whatever `RUST_BACKTRACE` says, and every run passes through all of `assert_report`, so each run can carry one value of each thing:

   ```rust
   #[test]
   fn a_panic_on_a_second_thread_aborts_the_process_after_its_report() {
       let output = panic_report(Some("full"), "str", None);
       // the "hook reported first" assertion, then
       assert_report(&output, "<unnamed>", "the fixture's thread panics");
   }

   #[test]
   fn owned_and_non_string_payloads_follow_the_default_report() {
       for (backtrace, payload, message) in
           [(None, "string", "the fixture's owned message"), (Some("1"), "any", "Box<dyn Any>")]
       {
           let output = panic_report(backtrace, payload, Some("fixture-worker"));
           assert_report(&output, "fixture-worker", message);
       }
   }
   ```

   Bite: make the hook print a backtrace hint only where `RUST_BACKTRACE` is unset, or skip its backtrace when it's `"1"`, in `skein-fail-stop`, and see a row fail. Restore.
3. **Move the two prefix sweeps to `skein/tests/http/tests/suite/fuzzy/`.** They need `transcripts()`, `exchange`, `settings` and `serve`: make those `pub(super)` or `pub(crate)` where they are, or copy them if they're short. `SEEDS` goes with its test where it must. The file's head says what's added. They inject the cut directly, so what they inject falls: say so in a comment, in place of a tally (`testing-strategy.md`, 3). Moved tests need no bite.
4. **`CALM` falls from 16 to 4.** A calm world draws only ports and the order of a cancel and its target, so four seeds must show both orders. Read how a seed draws that order. If these four don't show both, take the smallest count that does, and say which. `CHAOS`'s 200 in the fuzzy suite holds the breadth. The doc comment on `CALM` stays true. Bite: in `skein/testing/skein-conformance/src/scenarios.rs`, invert the expected pairing of a race, and see `a_calm_world_pairs_each_race_as_the_ring_does` fail. Restore.
5. **`worlds.rs`'s seeds.** A loop with an aggregate assertion falls to the smallest range that holds it, doubled for margin. A loop with none falls to four. Each lives in a named constant:
   - `an_llm_s_stream_goes_from_the_server_s_top_to_the_client_s_top`, 12: holds `ids > 0`.
   - `a_connection_carries_two_or_three_calls_through_both_stacks`, 8: shows both two and three calls.
   - `one_end_closing_while_the_other_sends_leaves_both_settled`, 30: `reset > 3` stays.
   - `the_wire_resetting_at_any_moment_leaves_both_settled`, 20: four, if `fuzzy::worlds::every_scenario_holds_whatever_the_caps_and_the_wire` draws a reset. Read `Settings` to see. If it doesn't, keep 20, and say so.
   - `a_seed_replays_to_the_same_run`, four seeds of two runs: two seeds.
   - The three holding tests at 3, 3 and 6 stay unless the profile puts them over the stop rule, and then each keeps the seeds its `writer_held`, `upload_held` or `upload_stopped` needs.

   Bite: in `skein/tests/protocol/src/world.rs`'s referee, drop the check of hearing a closure or of settling, and see the smaller loop fail. Restore.
6. **`UNWINDS` falls from 20 to 3,** with the comment saying so: a second unwind shows what the first left behind, and a third that it doesn't build up. Bite: a counter in `legacy_stderr_overflow` that panics with another message from its second call, and see `unwind 1` fail. Remove it.
7. **A wait that has arrived arms nothing.** `Run::wait` and `Run::either` (`skein/testing/skein-conformance/src/run.rs`) call `wait_begins` before anything, and `Real::wait_begins` (`skein/tests/conformance/real/src/lib.rs`) arms a `Deadline`, reading the scheduler, auxv, uptime and the process's threads, for each operation. The survey's reading, to check by the profile: reap once, return if it has arrived, and arm only then.

   ```rust
   pub(crate) fn wait(&mut self, process: B::Process, token: Token) -> Complete {
       self.reap(process);
       if let Some(complete) = self.arrived.remove(&token) {
           return complete;
       }
       self.backend.wait_begins("the conformance operation's completion", PATIENCE);
       // as today
   }
   ```

   `within` already begins with a reap, so a simulated world takes the same steps.
8. Fix each row's next causes, largest first, re-measuring after each.
9. Write the figures.
10. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p skein-conformance-real -p skein-conformance-sim -p skein-world-tests -p skein-http-world -p skein-fail-stop-tests -p skein-protocol-world --all-targets -- -D warnings` with any other package you changed. Then `heavy cargo nextest run` on them, and `--profile fuzzy` on `skein-http-world` and `skein-conformance-sim`. Commit and push.

## Report

For each row, what its tests paid for and the before and after, each bite, what the moves add to fuzzy, the causes found and not fixed, and which options you took. Your implementation and check minutes apart.
