# smith-test-cost: the plan

smith's focused suite goes back toward its 30 CPU seconds by removing what its own tests and fixtures waste (`inputs/request.md`), with no budget, timeout or build setting changed. No figure is asked of it: the gate's records are read after landing.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| scope-input | `scope-input` | | the directory the bench's scope gate lists becomes the request's `shared_temporary`: `/tmp` for a real run, a test's own in the tests |
| test-shapes | `test-shapes` | | the request's ten rows: seven are worked, three stay with their reasons |
| local-setup | `local-setup` | `test-shapes` | `make_local` parses its launch's document once, with no product change |

## Critical path

`test-shapes`, then `local-setup`. `scope-input` runs beside them and was released first, as the request asks. The request puts part 3 last, so `local-setup` waits for `test-shapes` although it builds on none of its code.

## Decisions

### Part 1

- **The directory lives on `run::Request`, not `Machine` or an environment variable.** A request is what `smith-bench attempt` and `probe` carry across their child processes, and an archived `prepared.json` then says what the attempt watched. `Machine` holds tool paths, and an environment variable would let a machine's environment move a real run's gate unseen.
- **The field has a serde default of `/tmp`.** A prepared document written before it reads as a real run. `benchmarks.status.md`, 6, records that an earlier request kept stored forms as they were, and this adds one without breaking any.
- **Only `probe` takes `--shared-temporary`.** `probe_command::a_replay_arm_runs_through_the_probe_command_and_prints_one_line` plans and dispatches inside one child, so it needs a way to name its directory on the command line. `suite`, `calibrate` and `fix` refuse the flag, so a suite or a calibration can't be moved, and a run that uses it says so in its evidence and its `prepared.json`.
- **The test fixture's default names a directory that doesn't exist.** A test that forgets gets an unavailable listing, never the machine's `/tmp`. `fault::tests::a_refused_attempt_keeps_the_seed_step` does reach an attempt (the request's "may be one", now read) and needs no edit.
- **`runner::a_file_left_in_tmp_by_an_attempt_is_a_write_outside` stops re-running itself.** Nothing `run::attempt` reaches reads the environment that re-run set, read at the cut; the spec keeps it if the implementer finds otherwise.

### Part 2, by row

Everything here was read from code at the cut. Nothing was run or profiled, and each spec entry carries a fallback where the reading could be wrong.

| Row | Decision | Why |
|---|---|---|
| `a_dense_result_at_the_line_bound_decodes` | `0` goes, and the fixture builds in one call | at the same bytes `[]` holds more tokens and depth, and two neighbours already hold zeros |
| `recorded_catalogues_and_results` | time the ten stories, then share what repeats | one true repeat was read, a design file per guard (29 guards); the likely cost is the digests, the 2.4 MB of summaries and the resamples, which are the check itself and stay |
| `a_reasoning_turn_that_stops_after_a_read_is_told_and_nudged` | count the events, then take the first option that applies | `acknowledgement_window(1)` is the story's *told* and stays; the frames or the 87 KB of reasoning are the suspects, and nothing asserts or cites those sizes |
| `nesting_asked_for_...` | one dialect | the refusal is the domain's; `render.rs` and `codec.rs::conveys` hold the flag in both dialects |
| `a_resumed_program_main_can_start_fresh_children` | one chat holding both children | `named` changes only the last assertion |
| `a_plain_snapshot_at_and_past_its_bound_answers` | no trace | it reads no record, and `directory` sets `traced` for all its stories; both bounds and both placements stay |
| `a_relay_sized_tool_list_fits_the_memory_envelope` | stays | the name's size is what it checks, and its cost is `skein-json`'s string paths, another plan's (read, not profiled) |
| `an_seen_full_escaped_prompt_fits_...` | stays | no sweep, despite the question: one chunk size, each chunk of one run metered |
| `every_probe_tasks_overrides_pass_...` | stays | 15 probes with 15 `[smith]` blocks, already rows of one table test |
| `host_cancellation_at_many_moments_...` | the sweep moves to fuzzy, three named moments stay focused | 40 moments assert one aggregate, which is what makes it a fuzzy test; its three classes do name behaviours |

The move is the only one in the plan. It adds about 0.14 CPU s to fuzzy's 27, by the request's figure.

### Part 3

Kept, not dropped, but smaller than the request's reading suggested.

- **Taken:** `make_local` parses the same document twice on traced and interactive launches. That is the fixture's waste, and it's removed with no product change.
- **Not taken:** `header::name`. It already writes in bulk, its scan is over one or two endpoints, and its three passes are the fixed point `trace_header` needs. Reworking that isn't real waste, so no product crate changes.
- **Not taken:** a parse shared across a story's worlds. About 18 of the 64 stories loop over both placements and would save one parse of two, but `agent::Config` isn't `Clone` and the document varies by launch, so a cache is more fixture than that saving.
- **No design change:** parts 2 and 3 change tests and fixtures only, so only part 1 wrote design.

## Overlaps with drafts beside this one

None in the code. `model-catalogue-2/smith-limits` touched `smith/benchmarks/**` and `smith/tests/**` and has merged, and the `limits-*` drafts in flight are temper's.

`scope-input` and `test-shapes` share the `smith-bench` crate while running side by side, against the one-writer rule. They touch different files, and the specs say which, so holding one for the other would only have cost a slot.
