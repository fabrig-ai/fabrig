+++
id = "llm-wire"
part = "llm-dialects"
kind = "migration"
needs = ["llm-mirrors"]
files = ["skein/crates/skein-llm/src/wire.rs", "skein/crates/skein-llm/src/wire/**", "skein/crates/skein-llm/src/lib.rs", "skein/crates/skein-llm/src/anthropic.rs", "skein/crates/skein-llm/src/anthropic/**", "skein/crates/skein-llm/src/openai.rs", "skein/crates/skein-llm/src/openai/**", "skein/crates/skein-llm/src/dialect.rs"]
effort = "high"
minutes = 50
+++

# skein: what both dialects share sits in `wire`, and Anthropic names nothing of `openai`

## Goal

The second of three steps of `skein/docs/development/debt/D012-anthropic-on-openai.md`. A private module `wire` takes what both dialects use, by verbatim moves, and Anthropic's modules
read it there. No request changes: the same bytes go out, and no test changes.

Acceptance:

- The item's **Measure** reads 0: `rg -c 'openai::' skein/crates/skein-llm/src/anthropic.rs skein/crates/skein-llm/src/anthropic -g '!*tests*' | awk -F: '{s+=$2} END{print s}'`. The narrower words too: `rg -n '\bopenai\b' skein/crates/skein-llm/src/anthropic.rs skein/crates/skein-llm/src/anthropic -g '!*tests*'` finds nothing, since a `use crate::openai;` and a bare `json::` that resolves through it are the same coupling.
- `rg -n 'openai|anthropic' skein/crates/skein-llm/src/wire.rs skein/crates/skein-llm/src/wire` finds no module path (`Provider::OpenAi` and `Provider::Anthropic`, which `classify` dispatches on, aren't paths of a dialect's module).
- `heavy cargo nextest run -p skein-llm -p skein-llm-world -p skein-fake-llm-protocol -p skein-llm-connection` passes with no test file changed, `anthropic/tests.rs` and `request_tests.rs` included: they stay green through the re-exports.
- `anthropic::decoded_bound`'s value is the same for every limit (`dialect`'s function is the same sum).
- Net production rises by at most 60 lines (estimate +40: a module's lines and re-exports). The part ends below where it started.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `skein/docs/development/debt/D012-anthropic-on-openai.md`
- `skein/docs/design/llm.md`, 2 (the shared vocabulary) and 4 (the bound's words): written, and binding

## Owns and may change

- **Owns:** the files in `files`. `mod wire` is private: no new public path.
- **The moves, each verbatim:**

  | `wire/` file | What moves | From |
  |---|---|---|
  | `limits.rs` | `Limits`, `encoder_bound` | `openai/common.rs` |
  | `failure.rs` | `DecodeError`, `ProviderError`, `RateLimit`, `classify` and both its tables, `decode_error`, `read_error` | `openai/common.rs`, `openai/response.rs` |
  | `json.rs` | all of `openai/json.rs`, `long_text`, `optional_text` | `openai/json.rs`, `openai/response.rs`, `openai/request.rs` |
  | `detail.rs` | `append`, `clipped`, `measured`, `limit_detail` | `openai/common.rs` |
  | `reasoning.rs` | `replay_item`, `write_replay`, `append`, `charge` | `openai/response/reasoning.rs` |

  `openai/common.rs` keeps `decoded_bound`. `reasoning::{following_index, slot}` stay in `openai`, since both read `Opened`. Names here are by reading: if one more item must follow to compile, it follows, and your report says so.
- **Public paths stay, by re-export.** `openai` keeps `pub use` of `Limits`, `DecodeError`, `Json`, `ProviderError`, `RateLimit`, `classify`, `encoder_bound`, and `pub(crate) use` of `json` and the other internal paths, so `translate.rs`, `client.rs` and `dialect.rs` barely change. `anthropic.rs` re-exports the same names from `crate::wire` and `crate::`, never through `openai`. `lib.rs`'s `DocumentError` and `Json` read `wire`. `wire::Limits` can't sit at the crate's root: `skein_llm::Limits` is another type.
- **`anthropic::decoded_bound` keeps its value, and moves.** Today it adds Anthropic's decoder and marks to `openai::decoded_bound`. Its body moves to `dialect.rs`, which knows both dialects, and `anthropic.rs` re-exports it under its name. **This cut decided it** (`temper/docs/plan/debt-19/plan.md`): a value a public function returns doesn't change in a refactor.
- **Design files:** none; `skein/docs/design/llm.md` is already written. Don't touch `llm.status.md`: `llm-output` closes its entry.
- **Don't move fixtures:** `.config/fixture-readers.toml`, `skein/tests/llm/tests/suite/archives.rs` and `derive.rs` read them by path.
- **Comments you move keep their words;** the old-form count `actual` may only fall.

## Steps

1. `wire.rs` and its files, by `git mv` where a whole file moves (`openai/json.rs`), one family a commit, each compiling.
2. Anthropic's `use` lines.
3. `decoded_bound`.
4. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-llm --all-targets -- -D warnings`; the tests above; `heavy cargo check --tests -p smith-protocol-llm -p smith-agent-service -p jig-test -p temper-fake -p temper-playground -p temper-twin`. The citations check. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items, the net lines, and any item that followed beyond the table.
