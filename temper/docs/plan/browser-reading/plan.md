# browser-reading: the reading story costs less, checking the same

The request: `inputs/request.md`. `browser-temper` is over its 85 CPU s budget, and `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place` takes 18 to 24 CPU s of it. The cost is its fixture (8 long replies of 9,600-word code lines, 839 CDP commands returning 5.65 MB) and the decoder that reads those replies.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `reading-window` | `tasks/reading-window.md` | | `small-window` names a 32,768-byte window, and the bundle's `long_text` and `turn_text` at 32,768. Reading's code lines are a quarter of today's, its counts and paragraphs as they are. The story passes by name, alone | 90 |
| `decode-index` | `tasks/decode-index.md` | | `skein-browser`'s `wire::decode` indexes a message's values and keys once in `parse`. `get`, `array`, `Array::next` and `raw` look up in it, under unit tests of the lookups. The story passes once by name, alone | 120 |

Both are released in the first push and run side by side. They share no file and no type.

**Critical path:** `decode-index` alone, about 120 implementer-minutes, most of them builds and the story's one run.

## What was decided

- **The window's floor is `long_text`, and today's window is at it.** `small-window`'s 131,072 equals the web shell's `LONG_TEXT`. `temper_client_domain::limits::valid` refuses a window under `long_text` (`temper/docs/design/client/README.md`, 5.15), since a line holds up to `long_text` (`thread_limits`). So "as small as the mechanism allows" means lowering the tier's bundle's `long_text` and `turn_text` with the window, under the same feature. The window mechanism stays the same.
- **32,768 bytes, a thirty-second of `document_bytes`.** It is the largest page `read_docs` answers (`temper-docs`' `PAGE_BYTES`), so a tier story's line holding one stays whole. Only the history story calls `read_docs`, in a turn of about 14 KB. No other tier story shows or types text near 32,768. A smaller window would fall below a page `read_docs` may answer. Below about 16 KB, the code lines would no longer carry most of a reply's bytes. The task's fallback is 65,536, if the story can't pass at 32,768.
- **Only the code lines scale; the counts and paragraphs stay.** A reply's bytes ride on one horizontally scrollable code line, one line tall at any length. Its height rides on its 64-byte paragraphs. The story's keys (6, 4, 6 and 4 presses) and the replies its steps name (`reply_end(2)`, `reply_end(3)`, `read_to_end` from 4, `messages - 2`) rest on those heights and on the window holding about two and a half replies. So each history keeps its share of the window: the long one 3.085 windows (today 3.084), the position one 0.837 (today 0.837). `LONG_CODE_WORDS` goes from 9,600 to 2,020 and `POSITION_CODE_WORDS` from 7,000 to 1,520. Fewer replies would change which reply each step reads, so the counts aren't what shrinks. The task may tune the code words within stated bounds if the window's room per line (`LINE_ROOM`) moves an eviction.
- **The design is written in this cut:** `temper/docs/design/client/browser.md`, 4.3 (*The tier's window*); a clause in `README.md`, 5.4 (the tier's bundle may name a smaller `long_text`, with its window); `skein/docs/design/browser.md`, 4 (*A message is parsed once*) and 8 (the index's memory). Each status file has a **Not yet** line, which its task removes.
- **The decoder's public surface doesn't move.** `Document`, `Value`, `Array` and `DecodeError` keep their names, signatures, derives and results. So none of its five direct dependents changes. `Value` becomes a document reference and an entry number. The index is 12 bytes an entry, counted while validating and allocated once. The memory suite's ceilings stay: the task shrinks the entry before it would raise one, and asks with measurements if nothing fits.
- **Each task runs the story by name, alone,** as the request allows: once for the decoder, against Chromium, and until it passes for the fixture. No task runs a tier, skein's `browser-skein` suites, or another story. No CPU figure is read or compared.
- **No notes and no contracts:** two independent tasks. The request's `For every task of this plan` stands in for `notes/common.md`.
- **No survey call:** the readers were few enough to read here. `reading.rs` is used by `Tier::reading` and the one story. `wire::decode` is read by `skein-browser`'s `browser.rs` and its tests, and `skein/tests/browser`'s tests. `small-window`'s window has one test, in `host_tests.rs`. No Markdown these tasks change is read by tests (`.config/gate/read-by-tests.txt`).

No stored format, protocol or public interface changes. The tier, its budget and the story's place stay. The story's file stays as it is.

## Overlaps with drafts beside this one

- `temper/docs/design/client/browser.status.md`: `ui-engine-split/engine-binary` (not merged) also edits it. This cut adds one **Not yet** line under the 7.3 entry, and `reading-window` removes it. Other sections' lines, so they merge cleanly.
- No draft beside this one names `reading.rs`, `browser_pages.rs`, `temper-client-shell-web/src/limits.rs` or `host_tests.rs`, `skein-browser`, `browser.md` in either tree, `temper/docs/design/client/README.md` (only its status file), or `skein/docs/design/browser.status.md`.
