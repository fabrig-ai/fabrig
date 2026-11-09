# red-87-replay: the plan

An after-landing red (`inputs/request.md`): the browser story
`browser_playground::story_a_walks_on_the_built_playground` downloads
story A's session log from the built playground and replays it natively;
a few runs a day the replay fails with `Differs { line: 524, .., why:
Said }`, so what the browser's world said by some line is not what the
native world says over the same log.

## What the cut found

Read, not run (a cut builds nothing):

- **The replay's loop matches the slices.** `skein-playground`'s
  `replay.rs` steps while busy, moves the idle clock to the next due
  instant or the next stamp, enters each input at its stamp and folds
  what the visitor reported and the world printed after each step; the
  slices enter a slice's inputs before stepping and post after, and the
  page's `log::Log` folds between stamps. The visitor's inputs only
  queue (`Visitor::enter`), so nothing is said between an input's entry
  and the next step. The log's text carries every field of the bridge's
  input (`cache` and `keep` included).
- **Nothing ambient in the world:** no `HashMap`, `HashSet` or
  `RandomState`, no `SystemTime`, `Instant::now` or `getrandom`, no
  transcendental float, no code gated on `target_arch` in the world's
  crates. The world's printer bounds unread output at 64 KiB, but only
  the service prints there, and only sign-in links.
- **The build's word size reaches the world.** The browser's world is
  the wasm32 build, the replay the native 64-bit one, and:
  - `temper-playground`'s `documents/turn_bounds.rs` writes the story's
    deployment limits from `size_of::<Box<[u8]>>()` (8 on wasm32, 16
    natively) and `size_of::<temper_engine_domain::Record>()`:
    `hub.transcript_bytes` is about twice the resume on wasm32 and four
    times natively; `root.loads.bytes` and `reply_bytes` differ by the
    record's size. The two worlds run different documents.
  - The engine counts held bytes with `size_of` of its slots (for
    instance `temper-engine-domain`'s `loads.rs`, which cuts a load's
    page at `size_of::<Record>()` plus each record's owned bytes against
    `root.loads.bytes`, and `store/people_bytes.rs`), and the
    playground's documents are tight (temper's `playground.md`, 8: "each
    just big enough"). So near a bound the two builds decide
    differently, which content near the bound, and so the person's
    timing, makes happen now and then.

  That is the lead the second part starts from; it is not proven, which
  is why the first part makes the next difference say what it was.

## Parts and tasks

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `differs-says` | `differs-says` | `skein-playground` (`replay.rs`), `skein-ui-web` (`log.rs`, `log/text.rs`, `tests/log.rs`), `skein-playground-world` (`suite/replay.rs`); `skein/docs/design/playground.status.md`'s entry | | 60 |

Slice 1 is `differs-says`; `walk = false`.

## Design written by this cut

- `skein/docs/design/playground.md`, 6.2, *What a difference says*:
  the line's input, both digests and the replay's last 8 lines, in the
  log's text, each cut at 200 bytes. Its status, `playground.status.md`:
  a **Not yet** 6.2, which `differs-says` removes.

## Critical path

`differs-says`, released with the first push; the second part, written
next, needs it.

## Overlaps with drafts beside this one

None of the drafts beside this one names `skein-playground`,
`skein-ui-web`'s log, `skein/tests/playground` or skein's
`playground.md`.
