+++
id = "test-shapes"
part = "test-shapes"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-protocol-program/src/backend/conversation_tests.rs",
  "smith/benchmarks/tests/suite/offline.rs",
  "smith/benchmarks/src/check.rs",
  "smith/benchmarks/src/suite.rs",
  "smith/tests/agent-process/tests/suite/stories.rs",
  "smith/tests/agent-process/src/fake.rs",
  "smith/tests/agent-process/src/lib.rs",
  "smith/tests/agent-process/src/world/peer.rs",
  "smith/tests/agent-process/tests/suite/program_children.rs",
  "smith/tests/protocol-llm/tests/suite/nesting.rs",
  "smith/tests/local-process/tests/suite/composition.rs",
  "smith/tests/agent/tests/suite/simulation.rs",
  "smith/tests/agent/tests/suite/fuzzy/simulation.rs",
]
effort = "medium"
minutes = 150
+++

# The tests whose own shape costs

## Goal

Ten rows of `inputs/request.md`, 2, read at this cut and decided below, in 8.1's order: first whether the test belongs in focused, by what it checks and never by its cost; then what is wasted in it or its fixture. Seven rows are worked, and rows 2 and 3 may still end as stays by their fallbacks; three stay with no edit. Row 10 is the only one that moves, because what it checks makes it a fuzzy test. No assertion is lost: where a check moves or merges, say where it went.

Acceptance:

1. Each row below is done as its entry says, or takes its entry's stated fallback, and your report says which.
2. The tests named below pass in their profiles. Every other test of the six packages still passes.
3. Rows 7, 8 and 9 have no edit. The report repeats why they stay.
4. The report has one line per row: what cost, what changed, or why it stays. It gives any timing you took as a reading only.

## Inputs

- `smith/docs/plan/smith-test-cost/notes/common.md`.
- `smith/docs/plan/smith-test-cost/inputs/request.md`, 2, and `Limits`.
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1 (what makes a test focused or fuzzy; "cheaper by its algorithm first").
- `smith/docs/design/testing.md`, 6.1, and its 8 for the placement question, which stays open.

## Owns and may change

- **Owns:** this spec's `files`. `scope-input` owns the rest of `smith-bench`'s tests and sources at the same time: don't touch `run.rs`, `isolate.rs`, `probe/`, `pipeline.rs`, `runner.rs` or `probe_command.rs`.
- **Readers, read at the cut:** nothing outside its own test names `dense_result`, `chat_story` or `directory` (`composition.rs`'s). `fake::reasoning`, `reasoning_configuration` and `stopped_reasoning_peer` serve only row 3's story. No design document or tool names any of these tests; only `inputs/request.md` does.
- **Throwaway timing and prints** stay untracked or uncommitted, and are gone before you push (`AGENTS.md`, 4).
- **Out, whatever a timing shows:** build settings, `skein-json`, `skein-heap`, and the design's 10,000 resamples.

## Steps

### Row 1, `smith-protocol-program`, `a_dense_result_at_the_line_bound_decodes`: `0` goes

It belongs. Framing is by bytes and doesn't depend on the element, and at the same bytes `[]` holds more tokens and more depth than `0`. Zeros are decoded by `a_result_with_the_reviews_ignored_array_decodes` and refused past the bound by `a_result_past_the_line_bound_is_refused`, which both stay. So the test takes `[]` alone, its comment saying that's the bound's worst case. Also build `dense_result`'s body in one call, since `count` elements in an unoptimized loop are the fixture's own cost:

```rust
let mut elements = [element, b",".as_slice()].concat().repeat(count);
elements.pop();
bytes.append(&mut elements);
```

The byte-at-a-time `intake` push stays: `skein_lib::List` has no bulk append, and skein is out of this plan.

### Row 2, `smith-bench`, `offline::recorded_catalogues_and_results`: time it, then share what repeats

The ten stories stay in one process and keep their assertions. Time each once with a throwaway. Read at the cut, not measured:

- `every_committed_suite_and_configuration_pin_is_checked_offline` is the likely cost: it digests every seed and strictly parses 2.4 MB of summaries and 240 KB of baselines, once each, and verifies parity, which is the 10,000 resamples.
- `check_guard` (`src/check.rs`) reads and scans a whole design file per guard, 29 guards across the catalogue. This one is a true repeat. If design reads are at least 5% of this test, read each design file once per `catalogue`, through a private helper, with `check_task`'s signature and every refusal's words unchanged.
- `validate_suite` reads a pin per suite and agent (`src/suite.rs`). If that's at least 5%, read each pin once per `check_benchmark_tree_with_bundles`.

If neither reaches 5% and no other repeat shows, this row stays with no edit, and the report gives the stories' readings. Std's pointer checks are the build's and stay.

### Row 3, `smith-agent-process-world`, `stories::a_reasoning_turn_that_stops_after_a_read_is_told_and_nudged`: find the 2,256 events first

It belongs. Count the client's events by kind with a throwaway, then take the first option that applies. Keep every assertion and the seed in all of them.

- `acknowledgement_window(1)` stays: it is the story's *told*, the host acknowledging each told turn before the next.
- `observe_facts(TraceCase::StoppedReasoning, 8)` stays as the selector. If its `8` output frames multiply the events, raise that number for this case: slot pressure is what `fact_drain` and its fuzzy sweep hold, and this story asserts none.
- If the bytes are the cause (two items of 43,688 bytes, streamed and replayed), scale the sizes by eight: items of 5,461 bytes, `reasoning_item` 8,192 in `reasoning_configuration`, and `fake::reasoning` back on `Options`' defaults if the requests fit. Nothing asserts these sizes and nothing cites them, read at the cut. Keep the relation they hold: each item fits `reasoning_item`, and together they exceed it. Add to the story that `queries[2]` carries both items, if the fake shows them. That is one assertion gained for the sizes given up.
- If scaling breaks the story, a bound depends on those sizes: restore them, and report the bound and why the row stays.

The report says what the events were.

### Row 4, `smith-protocol-llm-world`, `nesting::nesting_asked_for_is_answered_as_its_problem_on_observed_provider_requests`: one dialect

The refusal, its note and the typed `Problem::Nesting` are the domain's, and they don't differ by dialect. Dialects differ in the document and in the feedback's native flag. `render.rs::written_text_and_mechanical_feedback_keep_their_error_flags` holds an `Invalid` result's neutral flag, and `codec.rs::conveys` holds that flag's native form in both dialects, read at the cut. So the world runs once. Take `Dialect::Anthropic` (`smith_protocol_llm_world::fixture::Dialect`), whose flag is true, if its final request crosses 8,192 bytes; otherwise take `Dialect::OpenAi`. Report which, with its byte count.

`crossed_old_document_cap` becomes a direct assertion on `check_document_bound`. A comment says why one dialect suffices and names those two tests. If you find either doesn't hold what's said here, add the missing row to `render.rs` (`Problem::Nesting`) rather than keep the second world.

### Row 5, `smith-agent-process-world`, `program_children::a_resumed_program_main_can_start_fresh_children`: one chat, both children

`named` changes only the last assertion, a fresh child's `--effort`: `high` when it names its model, `medium` when it inherits main's. So one chat can hold both. `chat_story` loses its parameter, its first turn's children name their model, and the follow-up starts both kinds together:

```rust
Turn::new(vec![Step::Together(vec![inspect_child("CUE-README", true), inspect_child("CUE-MAIN", false)])])
```

Update the counts that follow from it (`before`, `calls`, and `chat_parked(&later, ..)`). Find each fresh child's call by its session and assert its effort, so that neither `high` nor `medium` can go unseen, whichever order they run. Every other assertion stays, including `--resume` on main, no session reused, and the `Seen::Acted` reads. If the fake can't script both in the resumed activation, keep the two runs and report why.

### Row 6, `smith-local-process-world`, `composition::a_plain_snapshot_at_and_past_its_bound_answers`: no trace it doesn't read

It belongs: at and past a bound each name a behaviour. Its placements stay, since that question is open. But `directory` sets `launch.traced` for all three of its stories, and only `a_named_read_only_root_refuses_the_agents_write` reads a record. `referee::seen` reads no trace. So `directory` stops setting it, and that one story sets it. If a story fails without its trace, the world needs it there: restore it for that story and say so.

### Rows 7, 8 and 9: no edit

- **Row 7, `mcp::a_relay_sized_tool_list_fits_the_memory_envelope`.** One world and one tool, and the 25,566-byte name is what it checks. The cost is that string through the codecs, which is `skein-json`'s string paths, another plan's. Read, not profiled.
- **Row 8, `sink_memory::an_seen_full_escaped_prompt_fits_sink_output_and_allocation_at_every_chunk`.** No sweep: one run at one chunk size, 65,536, each of its chunks metered, under a scenario's memory ceiling. That belongs in focused. The 12 MB of escaped output is what it asserts.
- **Row 9, `smith_adapter::every_probe_tasks_overrides_pass_the_observed_public_startup_derivation`.** All 15 probes hold their own `[smith]` block, and each row runs the derivation the name promises, as rows of one table test in one process, the shape testing-strategy.md, 8, asks for. No row duplicates another.

### Row 10, `smith-agent-world`, `simulation::host_cancellation_at_many_moments_closes_the_whole_tree`: the sweep moves, its three behaviours stay

Question 1: 40 moments that assert only one aggregate. That is a sweep, so it's a fuzzy test (testing-strategy.md, 8), and it moves because of what it checks. Its three classes do name behaviours, and those stay focused.

- Move it unchanged, under its name, to `tests/suite/fuzzy/simulation.rs`, keeping `settled`'s `judged().1 == 1` check inline and adding whatever private helper it needs there. A comment says why it's there.
- New in `simulation.rs`: `host_cancellation_early_inside_the_tree_and_after_the_answer_settles`, a table of three literal `(cancel_at, seed)` rows taken from the sweep's own moments and seeds (print its classes once):
  - **early:** `RunFailure::Cancelled` with no child opened;
  - **inside the tree:** `RunFailure::Cancelled` with a child opened;
  - **after the answer:** `Answer::Accepted`.

  Each row goes through `settled`, and the failure message names its row.
- Run the moved test by name: `heavy cargo nextest run -p smith-agent-world --profile fuzzy -E 'test(=fuzzy::simulation::host_cancellation_at_many_moments_closes_the_whole_tree)'`, adjusting the path to what nextest lists. Check the count isn't zero.

### Finish

The rules that must bite are in the implementer's text: row 10's three rows, and row 5's two efforts. Then `cargo fmt`, `python3 -I -B smith/scripts/old_forms.py`, clippy on the six packages, and their focused tests (`smith-protocol-program`, `smith-bench`, `smith-agent-process-world`, `smith-protocol-llm-world`, `smith-local-process-world`, `smith-agent-world`). Commit each row apart, saying why. Push.

## Report

Per the implementer's text, with the one line per row of acceptance 4, the dialect and bytes of row 4, the events of row 3, and each fallback you took.
