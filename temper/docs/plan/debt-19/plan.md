# debt-19: the nineteenth debt round

Seven independent items (`inputs/request.md`): skein D012, D014, D032; jig D008, D009; smith D011, D015. All seven are still open at this
plan's base; every **Measure** was re-read, and where it has moved the part says so.

## Parts and tasks so far

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| smith D015 | `smith-paths` | `paths-bench` | | medium | 30 | the benchmarks' shared test code moves behind `fixture`; their `#[path]`s go |
| smith D015 | `smith-paths` | `paths-rest` | `paths-bench` | medium | 60 | 40 more go; six stay, each with its reason; the item goes |
| smith D011 | `bench-fault` | `bench-fault` | `paths-bench` | high | 150 | one `Fault`; no string errors outside the edges; the item goes |

The skein and jig parts follow in later pushes.

## Critical path, as it stands

`paths-bench`, `bench-fault`, 180 minutes. `paths-bench`, `paths-rest`, 90.

## Decisions: D015

Re-measured: 57 attributes, as the item says.

- **Six stay, with their reason, and that's the item's second arm.** Five are the schema compiler's output (`generated/*.rs`), declared the same way in skein and jig, and one is
  `transcript-patches`' `patches/`, whose layout `skein/docs/design/codec.md`, 11 fixes. Hiding them behind `mod generated { .. }` would spell a convention differently in one tree for a count.
- **Two owners get a `fixture` feature:** `smith-bench` (`request`, `json`, `events`) and `smith-agent-shell` (`product_document`). Both are read by the crate's inline tests and by its own suite, so a world crate can't serve them. `smith-agent-process-world` takes the live probe, since it is that world's.
- **A module keeps its name,** so no test changes name: `grant_tests` moves to `grant_tests.rs` and `program_world` to `program_world.rs`, not the other way round.
- **`paths-bench` goes first and alone,** since `bench-fault` rewrites the same crate.
- **Production code:** no more than it started; only a feature's `mod` line and manifest lines are new, in test-only cfg.

## Decisions: D011

Re-measured: the item's `Measure` reads 205 now, and it under-counts: `Result<[^>]*, String>` misses `Result<Vec<u8>, String>`. The wider search reads 246. The task checks the wider one.

- **The kinds are chosen in `notes/contracts.md`, 1,** so the task has none to choose.
- **The kind reaches no stored file.** `End::HarnessError { what }` and `End::Refused { setup }` keep their strings, and `Display` is the detail alone. The item says the format changes only if a kind reaches the summary; a stored format would be a stop-and-ask, so this round doesn't. A kind in the summary is left for whoever wants it, and the item's reason for it stands in git.
- **`Setup` and `HarnessFault` go; `Refusal` stays:** its `file` and `key` are read by callers and tests, and it is what a document's reader returns, so it reaches a `Fault` by `From`. That's still one fault type in the harness.
- **No `From<String>`,** or the kinds would be decoration.
