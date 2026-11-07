# json-strings: the plan

skein-json's string paths stop costing several calls per byte, with no change of build setting, public interface or behaviour. The request is `inputs/request.md`; the design is `skein/docs/design/json.md`, 3.4, 4, 6 and 7, written with this plan.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| `writer` | `writer-runs` | | The writing pass runs no `escaped_len`. `escaped_len` and `write_escaped_content` take plain runs. Creates `string::after_plain` and `generate::runs`. |
| `decode` | `decode-runs` | `writer-runs` | `decode` takes plain runs. The text buffer becomes an `Intake`. |

Each part has its own slice, so that `decode-runs` merges alone and a red points at it, as the request asks.

**Critical path:** `writer-runs`, then `decode-runs`. Both are in `skein-json`, one writer at a time, and `decode-runs` builds on `after_plain` and `runs`.

## The calls made

- **The writing pass stops outright.** Every caller of `Encoder::write` was checked first: `notes/contracts.md` holds what was found. None reaches it unmeasured but three tests, and none gives it other text. `json.md`, 4 and `writer.rs`'s head say what goes unasserted.
- **No table.** The cheapest form under an unoptimized build and the subset is a slice pattern and a `match` on the byte: no call per byte. A 256-entry table needs a call or an `as` per byte to index.
- **The text buffer becomes an `Intake`.** `List` has no append of a slice, and adding one is outside the request's limits. `Intake::append` takes a run in one call.
- **Where the reference sits.** `decode` and `escaped_len` are the crate's own, and the fuzzy profile selects only the worlds' binaries. So step rows hold them directly against the per-byte state machine, and the sweeps hold them through the public paths against the world's reference and the standard library's check.

## Overlaps

None. No draft beside this one names `skein-json`, `skein/tests/json` or `skein-json-kit`. `skein/testing/skein-heap` is another plan's and is left alone.
