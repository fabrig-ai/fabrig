+++
id = "llm-mirrors"
part = "llm-dialects"
kind = "migration"
needs = []
files = ["skein/crates/skein-llm/src/types.rs", "skein/crates/skein-llm/src/translate.rs", "skein/crates/skein-llm/src/dialect.rs", "skein/crates/skein-llm/src/client.rs", "skein/crates/skein-llm/src/openai.rs", "skein/crates/skein-llm/src/openai/common.rs", "skein/crates/skein-llm/src/openai/response.rs", "skein/crates/skein-llm/src/anthropic/response.rs", "skein/crates/skein-fake-llm-protocol/src/documents.rs"]
effort = "high"
minutes = 30
+++

# skein: `Stop`, `Usage` and `Failure` are each defined once

## Goal

The first of three steps of `skein/docs/development/debt/D012-anthropic-on-openai.md`. The dialects' copies of `Stop`, `Usage` and `Failure` in `openai/common.rs` go, and both
dialects use the crate's own in `types.rs`, so `translate::{stop, usage, failure}` go too. No request, event or outcome changes.

Acceptance:

- `rg -n '(enum|struct) (Stop|Usage|Failure)\b' skein/crates/skein-llm/src -g '!*tests*'` lists only `types.rs`.
- `rg -n 'translate::(stop|usage|failure)\b|fn (stop|usage|failure)\(value: openai' skein/crates/skein-llm/src` finds nothing.
- `heavy cargo nextest run -p skein-llm -p skein-llm-world -p skein-fake-llm-protocol -p skein-llm-connection` passes, and every other reader compiles (step 3).
- Net production falls by about 75 lines (at most -40).

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `skein/docs/development/debt/D012-anthropic-on-openai.md`
- `skein/docs/design/llm.md`, 2 (the dialects share one vocabulary)

## Owns and may change

- **Owns:** the files in `files`.
- **Public paths stay.** `openai` re-exports the crate's three (`pub use crate::{Failure, Stop, Usage};`), and `anthropic.rs`'s re-export line keeps compiling through it for now: `llm-wire` retargets it. Outside readers spell `openai::Stop`, `anthropic::Stop`, `openai::Usage` (in `skein-fake-llm-protocol`, `skein/tests/llm`, `temper/tools/twin`), and they don't change.
- **The one test exception:** `translate.rs`'s inline test of these three mappings (about lines 1124 to 1144, its import about line 830) goes with them, since there's nothing left to map. No other test changes.
- **If `skein-fake-llm-protocol/src/documents.rs`'s conversion of `openai::Stop` to `skein_llm::Stop` (about lines 613 and 622) becomes an identity,** remove it; it's a copy's mapping too.
- **`types.rs`'s `Failure` is the wider:** it has `Cancelled` and `TimedOut { phase }`, which neither decoder produces, and `wildcard_enum_match_arm` is denied. `openai/response.rs` (about lines 603 to 613) and `anthropic/response.rs` (about lines 512 to 522) each gain an explicit arm that passes `detail` on. If `types.rs`'s derives lack one the copies have (`Hash`), add it: an addition.

## Steps

1. The copies and the three functions go; `dialect.rs` (about lines 148, 189, 192, 215, 218) and `client.rs` (about lines 1408, 1446) read the values directly.
2. The two arms.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-llm -p skein-fake-llm-protocol --all-targets -- -D warnings`; the tests above; `heavy cargo check --tests -p smith-protocol-llm -p smith-agent-service -p smith-agent-shell -p jig-test -p temper-fake -p temper-playground -p temper-twin`. The citations check. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items, the net lines, and what the fake's conversion became.
