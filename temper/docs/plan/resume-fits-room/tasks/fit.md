+++
id = "fit"
part = "fit"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-worker-wire/src/fit.rs", "jig/crates/jig-protocol-worker-wire/src/tests/fit.rs", "jig/crates/jig-protocol-worker-wire/src/sizes.rs"]
effort = "high"
minutes = 60
+++

## Goal

`jig-protocol-worker-wire` fits a resumed start's committed calls to a
room, and measures the smallest resumed start, as
`jig/docs/design/protocol/channel.md`, 9 (*A resumed start fits its
room first*) and 11 have them: every call kept by its name and tool,
the newest host answers whole while the frame fits, the rest cut to
fixed words naming their bytes, a delivery's answer never cut. This is
the request's item 2, the fix's core; `fit-room` calls it from the
engine's link.

Acceptance:

- `fit::Fitted`, `fit::cut_words`, `fit::fit` and
  `sizes::smallest_resumed` exist as
  `temper/docs/plan/resume-fits-room/notes/contracts.md`, section 2 has
  them, and do what its numbered steps say.
- The table of Steps, 2 passes, each row bitten (Steps, 3).
- Every existing test of `jig-protocol-worker-wire` passes, and
  `jig-protocol-engine-workers` and `jig-protocol-worker-link` build
  their tests unchanged.
- The **Not yet** entry `11: jig-protocol-worker-wire neither fits a
  resumed start's committed calls to a room nor measures the smallest
  resumed start` in `jig/docs/design/protocol/channel.status.md` is
  removed, its lines alone (`notes/common.md`, *Status lines*).

## Inputs

- `jig/docs/design/protocol/channel.md`, 9 (*A resumed start fits its
  room first*), 11 (the `jig-protocol-worker-wire` sizes' bullet), 12
  (*A resumed start fitted to its room*: the step table).
- `jig/docs/design/protocol/channel.md`, 6.1 (*Calls committed since a
  resumed run's last turn*; a delivery's evidence must decode).
- `temper/docs/plan/resume-fits-room/notes/contracts.md`, sections 1
  and 2.
- `temper/docs/plan/resume-fits-room/notes/common.md`.

## Owns and may change

- Owns: `src/fit.rs` (new), `src/tests/fit.rs` (new), and
  `smallest_resumed` in `src/sizes.rs` with its doc line.
- Neighbour files, as named:
  - `jig/crates/jig-protocol-worker-wire/src/lib.rs`: `pub mod fit;`
    after `pub mod sizes;`; and its first doc line may become
    `//! Worker channel records and their sizes, and a resumed start's
    committed calls fitted to a room, without domain or payload policy.`
    The fit treats each answer's body as opaque bytes.
  - `jig/crates/jig-protocol-worker-wire/src/tests.rs`: `mod fit;` after
    `mod lists;`, the file's last module.
- Pre-approved:
  - **Measuring.** The generated `Assignment::measure()` gives a body's
    exact bytes, allocation-free. Default: measure the record once, then
    count each cut by its body's length less its words', which the
    encoding makes exact (each body carries a four-byte length), and
    measure the returned record again in the tests, never in `fit`.
  - **Rebuilding.** The generated records are rebuilt through their
    `into_parts` and `new` with the caller's `limits`. If a record has
    no `into_parts`, use its accessors and say so; never hand-edit
    `src/generated/`.
  - **Tests' limits.** Default: the crate's test fixtures' sizes through
    `sizes::limits`. A row whose start must pass a room uses a small
    room of its own, its answers sized to it, a handful of answers at
    most.
- Not yours: every other crate, `src/generated/`, the schema and the
  goldens, any `docs/` but the status entry.

## Steps

1. **The surface.** `src/fit.rs`, step code (`notes/common.md`; no
   `std`, every `match` whole, no indexing, checked arithmetic, no
   `as`), with the module doc citing `protocol/channel.md`, 9 and 11:
   `Fitted`, `cut_words` and `fit` as contracts section 2 has them,
   its four steps in their order. `smallest_resumed` in `sizes.rs`, by
   the encoding's arithmetic (header 8, fixed fields 57, each call 14
   plus its name, its tool and `cut_words(answer)`'s length), `None`
   past `u32`.
2. **The table,** `src/tests/fit.rs`: one test,
   `a_resumed_start_is_fitted_to_its_room`, its rows:

   | row | the assignment, the room | expected |
   |---|---|---|
   | fits | three committed host answers, room past its frame | `Whole`, returned equal to the input |
   | past by its calls | five host answers of one size, the room holding the rest of the start and two of them whole | `Cut { cut }`, the two newest whole, every older cut to `cut_words(len)`, frame at most the room |
   | met exactly | as above, the room exactly the frame with the newest two whole | the newest two whole |
   | one byte short | that room less one | the newest one whole |
   | a delivery | as "past by its calls", a delivery among the oldest | the delivery whole, every other older host answer cut |
   | short answers | as "past by its calls", an oldest host answer no longer than its words | that answer whole, its neighbours cut |
   | errors | a cut answer whose error flag is set | still set, its body the words |
   | still past | a charter past the room alone | `Past { bytes }`, `bytes` the returned record's frame, every host answer that can be cut, cut |

   Every row: each call's name, tool and place unchanged; nothing of the
   assignment but cut bodies changed (compare its parts); the returned
   frame, measured, at most the room but in "still past".
   Beside it, `the_smallest_resumed_start_is_measured`: for small's
   stock fields (32, 64, 64, 69,632) and standard's (128, 64, 64,
   135,168), an assignment built as contracts section 2 defines it,
   measured plus eight, equals `smallest_resumed`, and equals 6,817 and
   27,201; `smallest_resumed(u32::MAX, 64, 64, 1)` is `None`. And
   `cut_words(135168)` is exactly
   `[this call ran; its answer of 135168 bytes was left out of this start]`.
3. **Bite.** Break the rule three times by hand, each alone: cut from
   the newest instead of the oldest; cut a delivery; drop the `+ 8` of
   the header. See the rows fail for that reason, restore, and name in
   the report each break and the rows that failed.
4. **Runs.** `heavy cargo nextest run -p jig-protocol-worker-wire --lib
   -E 'test(/fit|smallest/)'`, then `heavy cargo nextest run -p
   jig-protocol-worker-wire --lib`; `heavy cargo check -p
   jig-protocol-engine-workers -p jig-protocol-worker-link --tests`.
   Check each run's count.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo
   clippy -p jig-protocol-worker-wire --all-targets -- -D warnings`;
   remove the status entry; commit saying why (a resumed start's
   committed calls must be cut to the link's room for the run to resume
   at all); the citations check; push.

## Report

As the implementer's text has it, and: the measured smallest resumed
start for each stock profile; each row's `Fitted` and its frame's bytes
against its room; the breaks of Steps, 3 and the failures seen; each
neighbour file edited and how.
