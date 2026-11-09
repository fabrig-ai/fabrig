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
  - `temper-playground`'s `documents/turn_bounds.rs` derives limits
    from `size_of::<Box<[u8]>>()` (8 on wasm32, 16 natively) and
    `size_of::<temper_engine_domain::Record>()`; but it writes
    `documents::document()`, which tests use, not story A's deployment
    (below).
  - The engine counts held bytes with `size_of` of its slots (for
    instance `temper-engine-domain`'s `loads.rs`, which cuts a load's
    page at `size_of::<Record>()` plus each record's owned bytes against
    `root.loads.bytes`, and `store/people_bytes.rs`), and the
    playground's documents are tight (temper's `playground.md`, 8: "each
    just big enough"). So near a bound the two builds decide
    differently, which content near the bound, and so the person's
    timing, makes happen now and then.

  Story A's deployment is a kept file
  (`temper-playground/stories/a/deployment.json`), the same bytes on
  both builds, so `turn_bounds.rs` is not what differs at run time; its
  byte bounds are tight (`core.load_bytes` 8192, `core.run_bytes`
  16384, `documents.text_bytes` 128, `views.text_bytes` 1024,
  `hub.event_bytes` 1024, `people.amendment_bytes` 1101), and jig's and
  temper's accounting of several of them adds `size_of` of a slot.
  The wasm32 world counts the same content in fewer bytes, so where the
  native world reaches such a bound and the browser's doesn't, a page
  is cut at another row or an answer refused on one build alone: the
  same stamps, another digest, as red-87 shows.

  That is the cause the second part tests for first, natively and
  deterministically; the first part makes any difference left say what
  it was.

## Decided

- **Clear of the bounds, not word-size-free accounting.** Counting
  every slot at one size on every target would touch most of jig's and
  temper's step crates (`size_of` appears about two thousand times in
  them); the request asks for one or two tasks. So the rule is the
  application's: a story replayed natively reaches no byte bound on a
  64-bit build, where the count is the larger, so neither build decides
  at one (skein's `playground.md`, 6, *Another build's slots*;
  temper's `playground.md`, 8 and 9). The replay check stays.
- **The test doubles the overridden byte bounds** and replays the kept
  log: a story that reaches none says the same. The `small` profile's
  own bounds are roomy and stay; a bound reached by payload alone, or
  one startup refuses doubled, is left out by name. It is a second row
  of `story_a_kept_log_replays_natively`, one more replay, no new world.
- **Room is the fix, sized by the test:** a reached bound doubles until
  the doubled replay agrees, kept within the `memory` binary's figures;
  past them is the user's (`Question:`).
- **If the kept walk reaches no bound,** the task runs the browser story
  by name, alone, at most three times (the request's exception), reads
  the `Differs` that `differs-says` makes, and fixes what it shows at
  its cause; none failing is a `Question:` with the experiments.
- **Effort medium** for both tasks, as the request asks.

## Parts and tasks

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `differs-says` | `differs-says` | `skein-playground` (`replay.rs`), `skein-ui-web` (`log.rs`, `log/text.rs`, `tests/log.rs`), `skein-playground-world` (`suite/replay.rs`); `skein/docs/design/playground.status.md`'s entry | | 60 |
| `bounds-clear` | `bounds-clear` | `temper-playground` (`stories/a/deployment.json`), `temper-playground-world` (`suite/walk.rs`, `fixtures/story-a.log`); `temper/docs/design/playground.status.md`'s entries | `differs-says` | 120 |

Slice 1 is `differs-says`, slice 2 `bounds-clear`; `walk = false`.

## Design written by this cut

- `skein/docs/design/playground.md`, 6.2, *What a difference says*:
  the line's input, both digests and the replay's last 8 lines, in the
  log's text, each cut at 200 bytes. Its status, `playground.status.md`:
  a **Not yet** 6.2, which `differs-says` removes.
- `skein/docs/design/playground.md`, 6, *Another build's slots*: a
  native replay is another build, whose slots are larger; an
  application keeps its replayed stories below each bound that counts
  slots, on a 64-bit build. A rule for applications: skein's code has
  nothing to build for it.
- `temper/docs/design/playground.md`, 8, *Clear of the bounds that count
  slots*, and 9, *Clear of its byte bounds*; its status,
  `playground.status.md`: **Not yet** 8 and 9 by those names, which
  `bounds-clear` removes.

## Critical path

`differs-says` (60 min, released with the first push), then
`bounds-clear` (120 min), which needs its `Differs` to read what a
doubled replay or a browser run shows.

## Overlaps with drafts beside this one

None of the drafts beside this one names `skein-playground`,
`skein-ui-web`'s log, `skein/tests/playground`, skein's or temper's
`playground.md` and their status files, story A's deployment or
`temper/tests/playground`. Unavoidable, though no draft lists it: any
plan that changes what story A holds (the stock, a page's reads) moves
the kept log, which `bounds-clear` records again; such a merge records
it again by the same test's writer.
