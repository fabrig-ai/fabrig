# smith-test-cost: smith's focused suite back under its 30 CPU seconds

The focused suite's limit is 30 CPU s per tree (`skein/docs/foundation/testing-strategy.md`, 8). smith's has read 30.7 to 32.1 over the last 19 full gates, 31.3 at the median, on 3,639 tests, and has grown about 0.4 a day since 11-01. Its machines' load doesn't explain it: the gates with the longest wall read 31.3, the shortest 31.1. Fuzzy, at 27, is fine and out of scope.

This plan fixes what smith's own tests and fixtures waste, in 8.1's order. **No test leaves its tier to escape the budget, no budget or timeout changes, and no build setting changes** (8, "A slow test is understood, never hidden"). Nothing asserted today may be lost.

Two costs are outside this plan, and other plans have them: **`skein/crates/skein-json`'s string paths, and `skein/testing/skein-heap`'s counting allocator.** Don't touch either, and don't count on them.

## 1. The bench's scan of the machine's `/tmp`: fix this first

`smith/benchmarks/src/isolate.rs`, `temporary_entries`, lists every entry directly under the real `/tmp`, stats it and inserts it in a `BTreeMap<PathBuf, _>`. `run.rs` calls it twice per attempt, for the `nothing-outside` check. A profile of `pipeline::an_existing_scratch_root_is_refused_and_never_deleted` puts 72% of that test's instructions there, nearly all of it comparing paths. A builder's `/tmp` held about 20,000 entries, left by other trees' browser tests. **So these tests' cost depends on what else the machine has run,** which is the hidden I/O that 8 forbids.

These reach it: `pipeline::login_shells_keep_machine_path_in_setup_agent_and_grading` (0.36 CPU s), `pipeline::grading_and_archives_keep_agent_commits_and_untracked_files_outside_the_timer` (0.30), `pipeline::an_existing_scratch_root_is_refused_and_never_deleted` (0.25), `runner::a_file_left_in_tmp_by_an_attempt_is_a_write_outside` (0.31) and `probe_command::a_replay_arm_runs_through_the_probe_command_and_prints_one_line` (0.67, through its child). Find any others that reach `run::attempt`; `fault::tests::a_refused_attempt_keeps_the_seed_step` (0.22) may be one, unread.

**What's wanted:** the directory the check watches is an input to the attempt (in `run::Request` or wherever fits), `/tmp` for a real run, and a test's own scratch directory in the tests. The product's behaviour doesn't change. `a_file_left_in_tmp_by_an_attempt_is_a_write_outside` still proves what it says, against the directory it names, and something still holds that a real run watches `/tmp`, without scanning it. Say so in `smith/docs/design/benchmarks.md`.

## 2. Tests whose own shape costs

For each row: first, does it belong in focused? What it checks decides that, never its cost. `testing-strategy.md`, 8, and `smith/docs/design/testing.md`, 6.1, are the rule: a sweep over many moments or sizes that names no behaviour is a fuzzy test, and moves if that's what it is. Then, if it belongs: find what's wasted in the test or its fixture and fix that. Leave a row alone if nothing is wasted, and say why. Don't thin a test merely to make it cheaper.

CPU s are from one full gate. What's in the third column is read from code or a profile unless it says otherwise.

| Test | CPU s | Known so far |
|---|---|---|
| `smith-protocol-program`, `backend::conversation_tests::a_dense_result_at_the_line_bound_decodes` | 0.59 | Decodes a document at the line bound twice, of `0` and of `[]`. Its comment says `[]` holds more tokens per byte, so it's the bound's worst case; `a_result_past_the_line_bound_is_refused` and `a_result_with_the_reviews_ignored_array_decodes` (0.19) already take zeros. If `0` proves nothing `[]` doesn't, it goes |
| `smith-bench::suite`, `offline::recorded_catalogues_and_results` | 0.50 | Ten stories in one process, on purpose. Profile: std's pointer checks about 20%, `stats::draw_solved` 5% (the design's 10,000 resamples, which stay), SHA-256 5%. Which story costs, and is a fixture read or digested repeatedly? |
| `smith-agent-process-world::suite`, `stories::a_reasoning_turn_that_stops_after_a_read_is_told_and_nudged` | 0.36 | 18 times its world's median. The client handles about 2,256 events and the fake peer's streaming is 27%. Why so many? `observe_facts(TraceCase::StoppedReasoning, 8)` and `acknowledgement_window(1)` are the visible differences |
| `smith-protocol-llm-world::suite`, `nesting::nesting_asked_for_is_answered_as_its_problem_on_observed_provider_requests` | 0.33 | The whole world per `client_peer::configurations()`. Does the behaviour differ across all of them? |
| `smith-agent-process-world::suite`, `program_children::a_resumed_program_main_can_start_fresh_children` | 0.29 | Unread |
| `smith-local-process-world::suite`, `composition::a_plain_snapshot_at_and_past_its_bound_answers` | 0.22 | Unread |
| `smith-fake-claude::suite`, `mcp::a_relay_sized_tool_list_fits_the_memory_envelope` | 0.21 | Unread |
| `smith-agent-service::suite`, `sink_memory::tests::an_seen_full_escaped_prompt_fits_sink_output_and_allocation_at_every_chunk` | 0.20 | A sweep by chunk: do all chunk sizes name a behaviour? |
| `smith-bench::suite`, `smith_adapter::every_probe_tasks_overrides_pass_the_observed_public_startup_derivation` | 0.19 | Unread |
| `smith-agent-world::suite`, `simulation::host_cancellation_at_many_moments_closes_the_whole_tree` | 0.14 | Sweep-shaped by its name: question 1 first |

## 3. The local process world's setup

In a median story of `smith-local-process-world` (64 tests, 3.2 CPU s), `process::make_local` is 24% of instructions: `config::from_value` 12%, `profile::derive` 8%. Separately, `process::product_header` is 10%, of which `smith-agent-service`'s `profile::effective::header::name` is 9%: a box per limit, with a scan of endpoints per name. A fixture many tests share should be cheap to build (8). Look for what a shared cheap path can drop without losing what the stories check. If a story needs the whole path, it keeps it; a product change is in scope only where it removes real waste, and says so. This part is worth less than 1 and 2, so it goes last and may come out empty.

## Limits

- **A plan runs the focused and fuzzy suites alone (`AGENTS.md`, 5).** The bench's own command, tiers and walks aren't run. No acceptance figure is asked of this plan, and it's complete when it lands; the gate's records are read afterwards.
- Which scenarios run in both placements is an open question (`smith/docs/design/testing.md`, 8). Leave it open.
- `walk = false`. Effort medium. Three tasks at most, one per part, and part 3 is dropped from the cut if the reading in the cut shows nothing to take.
- Report per row: what cost, what changed, or why it stays.

## Sources

Too large for a source, so read by section: `smith/benchmarks/src/run.rs` (`Request`, and the two `temporary` calls), `smith/docs/design/testing.md` 6.1, `smith/docs/design/benchmarks.md`, `smith/tests/local-process/src/process.rs` and each row's test.

- `smith/benchmarks/src/isolate.rs`
- `smith/benchmarks/src/passive.rs`
- `smith/benchmarks/tests/suite/pipeline.rs`
- `smith/benchmarks/tests/suite/runner.rs`
- `smith/benchmarks/tests/suite/probe_command.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **A test's helper that the change makes unused goes in the same task.**
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
