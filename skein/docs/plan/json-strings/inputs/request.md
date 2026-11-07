# json-strings: skein-json's string paths stop costing several calls per byte

Every tree's worlds speak JSON through `skein-json`, and test builds stay unoptimized (`skein/docs/foundation/testing-strategy.md`, 8), so a function call per byte costs what it costs. Profiles of smith's tests, by instruction count:

| Test | `parse` | `string::decode` | `Encoder::put_string` | of which `escaped_len` |
|---|---|---|---|---|
| A median story of `smith-agent-process-world` | 28% | 15% | 21% | 18% |
| A 0.1 s story of `smith-local-process-world` | 21% | 11% | 16% | 14% |
| `stories::a_reasoning_turn_that_stops_after_a_read_is_told_and_nudged` | 31% | 29% | 27% | 24% |
| `config::tests::native_validated_dense_prompts_fit_the_seen_joint_receiving_ceiling` | | | 68% | 47% |

This is the cost in the code under test that 8.1's second question asks for: fix the cause. **No build setting changes** (8), and **no public interface or behaviour changes:** the same bytes written, the same refusals, the same errors at the same bytes.

## The causes

1. **`writer.rs`, `put_string`, calls `escaped_len` on both passes.** The writing pass throws the length away. The file's head says the measuring pass refuses what a caller's data can get wrong, and `refuse` is `unreachable!` on a writing pass, so the check there can't change a correct caller's output. It is a whole per-byte pass with no use.
2. **`escaped_len` costs several calls per byte:** the slice iterator, `Utf8::next`, `lead`, `escaped`, `saturating_add`.
3. **`write_escaped_content` calls `escaped` per byte** through `enumerate`.
4. **`string.rs`, `decode`, costs more per byte again:** the iterator, `next`, `plain`, `Utf8::next`, `lead`, a `Decoded` built and returned, `Buffer::room`, then `Buffer::put` looping to push one byte, and `begun` recomputed with checked arithmetic.

## What's wanted

- **The writing pass stops measuring.** Check every caller of `Encoder::write` first. If any can reach it unmeasured, or with text that its measuring pass didn't see, that decides how this is done; say what you found. A bug that the old check would have caught in a caller, text differing between passes at the same length, is no longer asserted: say so in `writer.rs`'s head and `json.md`, or keep it some cheap way.
- **A plain run costs far less than today's per-byte calls.** Runs of ASCII that need no escape and no UTF-8 state are the common case. For instance a 256-entry table of what each byte is, read in a plain loop with no call per byte, in `escaped_len`, `write_escaped_content` and `decode`, handing a whole run to `room` and `put` at once, and falling to the existing state machine at the first byte that isn't plain. That's a suggestion: the cheapest form under an unoptimized build wins. The library's searches run optimized in unoptimized builds (8), where they fit.
- **`decode`'s contract stays exact.** `json.md`, 7: an outcome doesn't depend on how the bytes were cut; a byte's validity is decided before the limit it would pass; a character's room is decided at its first byte; and a position says where what's at fault starts, so `begun` reads the same wherever an error stops a run. `Retention::Capped` and `Retention::Discard` count as they did. `next` stays per byte for borrowed spelling.
- **The subset holds.** `json.md`, 7, keeps `core::str::from_utf8` out of this code, and its checks run in the tokenizer's own code. Keep that. If "a byte at a time" there no longer says what happens, correct the line, and keep why.
- **A naive reference holds it.** `skein/tests/json`'s worlds already compare; add what's missing so that a run-based `decode` and `escaped_len` are checked against the per-byte state machine over random text cut anywhere, including non-ASCII, escapes, control bytes and limits. In the fuzzy suite if it's a sweep, and a row or two of step tests for the runs' edges. Keep it small: skein's fuzzy and focused suites have budgets too.

## Limits

- **A plan runs the focused and fuzzy suites alone (`AGENTS.md`, 5).** No acceptance figure is asked of this plan, and it's complete when it lands; the gate's records are read afterwards.
- Only `skein/crates/skein-json` and its tests, and `skein/docs/design/json.md`. `skein/testing/skein-heap` is another plan's. `skein_llm::wire::json` and each tree's decoders are out of scope, even if they look wasteful: name them in the report.
- `walk = false`. Effort high: every tree stands on this code. Two tasks, writer and decode, and the decode task lands alone so a red points at it.

## Sources

Too large for a source, so read by section: `skein/docs/design/json.md`, 4 and 7, and `skein/tests/json`.

- `skein/crates/skein-json/src/writer.rs`
- `skein/crates/skein-json/src/string.rs`
- `skein/crates/skein-json/src/utf8.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
