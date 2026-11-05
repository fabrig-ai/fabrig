+++
id = "llm-output"
part = "llm-dialects"
kind = "migration"
needs = ["llm-wire"]
files = ["skein/crates/skein-llm/src/wire.rs", "skein/crates/skein-llm/src/wire/output.rs", "skein/crates/skein-llm/src/openai/response.rs", "skein/crates/skein-llm/src/anthropic/response.rs", "skein/docs/design/llm.status.md", "skein/docs/development/debt/D012-anthropic-on-openai.md"]
effort = "medium"
minutes = 25
+++

# skein: one `Output`, over each dialect's `Part`

## Goal

The last step of `skein/docs/development/debt/D012-anthropic-on-openai.md`: **Paid when** holds, and the item goes. The two `Output` enums are the same text but for the `Part` each wraps, so one generic
enum holds them, and each dialect names it over its own `Part`. Public paths, test files and sizes stay.

Acceptance:

- `rg -n 'enum Output' skein/crates/skein-llm/src/openai skein/crates/skein-llm/src/anthropic` finds nothing, and `rg -n 'enum Output' skein/crates/skein-llm/src/wire` finds one.
- `heavy cargo nextest run -p skein-llm -p skein-llm-world -p skein-fake-llm-protocol` passes with no test file changed. `heavy cargo check -p temper-twin --tests`: `round_trip.rs` matches `Output` and `Part` exhaustively and doesn't change.
- The item is deleted, and `skein/docs/design/llm.status.md`'s **Not yet** entry for section 2 goes.
- The part's whole net production is below zero (estimate -45): your report gives it.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `skein/docs/design/llm.md`, 2: why this is the crate's one generic item: binding
- `skein/docs/foundation/programming-model.md`, 10.3

## Owns and may change

- **Owns:** the files in `files`.
- **The shape:**

  ```rust
  // wire/output.rs
  /// One decoded stream event over a dialect's own `Part`.
  #[derive(Clone, PartialEq, Eq, Hash, Debug)]
  pub enum Output<P> {
      Part(P),
      TextDelta { index: u32, content_index: u32, text: Box<[u8]> },
      ToolStart { index: u32, id: Box<[u8]>, name: Box<[u8]> },
      ArgumentsDelta { index: u32, delta: Box<[u8]> },
      ReasoningDelta { index: u32, summary_index: u32, text: Box<[u8]> },
      Completed { stop: Stop, usage: Usage },
      Failed { failure: Failure, detail: Box<[u8]> },
      Progress,
  }

  // openai/response.rs and anthropic/response.rs
  pub type Output = crate::wire::Output<Part>;
  ```

  No `impl` on it, no bound and no generic function over it. Comments on its use cite `skein/docs/design/llm.md`, 2, so a reader sees the departure's reason.
- **`dialect::Output` stays,** and so do `responses_outputs` and `anthropic_outputs`: its `Part` is a translated block or an error and its failure carries a report, so it isn't a copy, and a function over both dialects would be a second generic item. Their `Completed` and `Failed` arms carry the values straight, since `llm-mirrors`.
- **If a type alias can't spell a variant somewhere** (a `use Output::*`, say), change that line alone and say so.
- **`Part` doesn't change,** in either dialect, and neither does what each decoder emits.
- **If a memory ceiling or `decoder_bound`'s value moves,** stop and ask: an alias over the same layout shouldn't move them.

## Steps

1. `wire/output.rs`; the two aliases. Delete the item and the status entry.
2. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-llm --all-targets -- -D warnings`; the tests above. The citations check. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items, the part's net production by `git diff --shortstat` against the plan's base, and any line the alias forced.
