+++
id = "decode-runs"
part = "decode"
kind = "feature"
needs = ["writer-runs"]
files = [
  "skein/crates/skein-json/src/string.rs",
  "skein/crates/skein-json/src/utf8.rs",
  "skein/crates/skein-json/src/tokenizer.rs",
  "skein/crates/skein-json/src/tests/text.rs",
  "skein/tests/json/tests/suite/fuzzy/strings.rs",
  "skein/tests/json/tests/suite/main.rs",
  "skein/docs/design/json.status.md",
]
effort = "high"
minutes = 90
+++

# `decode` takes a plain run at once, and its contract stays exact

## Goal

Cause 4 of the request. Between characters, `string::decode` finds a plain run with `string::after_plain` and hands the whole of it to the limit and the text buffer, falling to the state machine at the first byte that isn't plain. Nothing a consumer can see changes: the same tokens, `Long`s, `Skipped`s, errors and positions, however the bytes were cut.

Acceptance:

- `decode` makes no call per plain byte, and `Buffer::put` no call per byte of what it's given.
- `Buffer.bytes` is an `Intake` (`skein/docs/design/json.md`, 7), and `tokenizer_bound` is the same figure as before: `Intake::price` and `List::<u8>::price` agree.
- `next` stays per byte, and `put_json_spelling` is untouched.
- Every test of `skein/tests/json/tests/suite/memory.rs` holds, its ceilings unedited.
- The tests below hold and each new one bites.
- The two **Not yet** lines below are gone from `skein/docs/design/json.status.md`.

## Inputs

- `skein/docs/plan/json-strings/inputs/request.md`: cause 4, *`decode`'s contract stays exact*, *The subset holds* and *A naive reference holds it*.
- `skein/docs/design/json.md`, 3.4, 3.6's table, 6 and 7.
- `skein/docs/plan/json-strings/notes/common.md`, `skein/docs/plan/json-strings/notes/contracts.md`.
- `skein/crates/skein-lib/src/intake.rs`, `append`, `clear` and `meet`.

## Owns and may change

Owns the files in the header. In `tokenizer.rs`, only the text buffer's lines: `tokenizer_bound`'s price, `Tokenizer::new`'s buffer, `number_more`'s push and `number_token` and `string_token`'s `to_boxed`. A `use` the compiler asks for goes with them.

`skein/tests/json/tests/suite/main.rs` gains `mod strings;` under `mod fuzzy`.

`skein/docs/design/json.status.md`: remove these two lines, and change nothing else there (a departure, `notes/common.md`):

- `3.4, 7: string::decode takes a plain run a byte at a time, ...`
- `6: the step rows and the sweep of plain runs read.`

Options, each decided:

- **Why `Intake`, not `List`:** `List` has no append of a slice, and adding one is outside this plan's limits (the request). `Intake::append` takes the run in one call, and it grows as `List` does, by `grown`. The field's comment says why it's an `Intake` (`json.md`, 7).
- **A `memory.rs` ceiling falls:** don't raise it. Report `Question:` with the peak measured and what took it there.
- **`Buffer::room` and `Buffer::put`** become `pub(crate)`, for the step test's reference below. That's all they gain.
- **The limit's overflow:** `room`'s `Error::TooLong` on a `u64` overflow stays as it is, for a run as for a byte.

## Steps

**`string.rs`.** The shape, the choices inside yours:

```rust
pub(crate) struct Buffer {
    /// The text kept: an `Intake` for its `append`, which takes a run in one call (json.md, 7).
    pub(crate) bytes: Intake,
    ...
}

impl Buffer {
    /// As today, but `self.bytes.append(bytes).expect("room was reserved for the character")`.
    pub(crate) fn put(&mut self, bytes: &[u8])

    /// The text kept, taken out; the buffer keeps its storage.
    pub(crate) fn take(&mut self) -> Box<[u8]>     // `meet(Read::Fill(len))`

    /// A run of plain bytes (json.md, 3.4), the first at offset `first`. It
    /// leaves what those bytes one at a time would.
    fn run(&mut self, run: &[u8], first: u32) -> Result<(), Error>
}

pub(crate) fn decode(text: Text, piece: &[u8], first: u32, out: &mut Buffer) -> Result<Piece, Error> {
    // `rest`, what's left of the piece, and `at`, its first byte's offset, `checked_add`ed once a run.
    // Each round:
    //   match `text` (a derived `==` is a call): between characters, `after_plain(rest)`;
    //     a non-empty run goes to `out.run`.
    //   `let [byte, after @ ..] = rest else { return Ok(Piece::More(text)) };`
    //   between characters, `out.begun = at`;
    //   as today, one byte through `next`, `out.room`, `out.put`;
    //   a closing quote is last: `assert!(after.is_empty(), "a scan ends at its first quote")`.
}
```

**What `Buffer::run` must leave.** The contract, each part of which a row below holds:

- **`begun`:** for a run that fits, its last byte's offset, as the per-byte loop leaves it. For a run that passes `Strict`'s limit, the offset of the first byte past it, whatever its place in the run. An empty run changes nothing: `run` isn't called for one, and an empty piece leaves `begun` as it was.
- **`Strict`:** the bytes that fit are kept and counted. The first that doesn't takes `length` to `limit + 1` and fails `StringTooLong`, as one byte would. The simplest form splits the run where the room ends, hands the part that fits to `room` and `put`, then sets `begun` and calls `room(1)` for the byte past it.
- **`Capped`:** a whole run through `room` then `put` leaves what per-byte leaves: `room` clears the text once `length` passes the cap, and `put` retains nothing past it. `length` is the whole count.
- **`Discard`:** counted, not kept.

**`tokenizer.rs`.** `List::with_capacity` becomes `Intake::with_capacity`; `List::<u8>::price` becomes `Intake::price`; `number_more` calls `append(&[byte])`; `number_token` and `string_token` call `Buffer::take`, and `string_token`'s `clear` goes with it, as `take` empties the buffer. `Intake::meet`'s `Read::Fill(0)` gives an empty box, which `Capped` past its cap and an empty string rely on: check it by the rows.

**`utf8.rs`'s head and `string.rs`'s head.** `utf8.rs`'s first line becomes what's true, and its why stays:

```text
//! UTF-8's state machine, a byte at a time (RFC 3629, section 4), for every
//! byte but a plain run's (json.md, 3.4).
```

`string.rs`'s head says "every byte checked as UTF-8": it stays true, since a plain run is ASCII. Add one line, that a plain run is taken whole, citing `json.md`, 3.4.

**Step tests,** `skein/crates/skein-json/src/tests/text.rs`:

A reference of the test's own, the old `decode` a byte at a time through `string::next`, `Buffer::room` and `Buffer::put`:

```rust
/// What a string's text leaves, its pieces decoded one after another, stopping at the first error:
/// each outcome, then the text kept, the length, and `begun`.
fn by_runs(pieces: &[&[u8]], first: u32, retention: Retention) -> Left
fn per_byte(pieces: &[&[u8]], first: u32, retention: Retention) -> Left
```

- `a_run_reads_as_its_bytes_one_at_a_time_however_it_is_cut`: rows of text and retention, each cut in two at every place and also into single bytes, `first` non-zero in some, and `by_runs` equal to `per_byte` on each:
  - a run closed by its quote; an empty string; an empty piece; a run left open at the piece's end;
  - a run, then each of: `\n` spelled as an escape, `é`, a surrogate pair, `é`, a control byte, `\x80`, `\xC0\x80`, a bad escape;
  - `é` cut between two pieces, then a run;
  - `Strict` with the limit at a run's first byte (`0`), its middle and its last, and full at a run's end with `é` next, whose room is decided at its first byte;
  - `Capped` at those same places, and `Discard`.
- `every_byte_at_a_runs_edge_reads_as_it_does_alone`: every byte value `b`, in `aa` `b` `aa"` and `b` `aa"`, under each retention, whole and cut either side of `b`, `by_runs` equal to `per_byte`.

**The sweep,** `skein/tests/json/tests/suite/fuzzy/strings.rs`. The module's head says why it goes through the tokenizer: `decode` is the crate's own, and the fuzzy profile selects only the worlds' binaries, so the step test above holds it against `next` directly, and this sweep holds it against the reference, a naive reader a byte at a time.

- `strings_in_runs_read_as_the_reference_reads_them_wherever_they_are_cut`: about 3,000 rounds from a seed. A body from `generate::runs` with `pieces` in 0..6 and `longest` in 0..48; half the rounds with breaks a string may hold (`\n`, `\\`, `\"` spelled as escapes, `é`, `😀`, `é`, `€`, `😀`), half with those and what fails (a raw `\n`, `\x00`, `\x1f`, `\x80`, `\xc3`, `\xc0\x80`, `\xed\xa0\x80`, `\xf5`, `\x`, `\ud83d`, `\udc00`, `\u12`). Placed as `["body"]` or `{"body":1}`. `Limits { depth: 4, number: 16, length: 1 << 12, .. }`, with `string` half the time 4096 and half drawn in 0..=the body's length, and `chunk` in 1..=32; `Settings::calm`; `world::check`, which holds every token, outcome and position against the reference.

  A tally (`skein_protocol_kit::seen::Seen`) asserts that fell: `Done`, `Failed(StringTooLong)`, `Failed(Utf8)`, `Failed(Control)`, `Failed(Escape)`, `Failed(Surrogate)`; and "a limit passed inside a run": `StringTooLong` where `reference::reading`'s `failed_at` is a plain byte with a plain byte before it.
- `text_caps_and_skips_count_runs_as_the_reference_does`: about 1,000 rounds, valid breaks only, `limits.string` roomy, `world::check_demands`, which draws `Text` caps below 32 and `Skip`s. A run's length reaches past those caps. It asserts `Token`, `Long` and `Skipped` fell.

If either takes over 2 s alone on your builder, lower its rounds; say what it took.

**Narrow tests, then the close:** `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-json -p skein-json-world --all-targets -- -D warnings`; focused, `heavy cargo nextest run -p skein-json -p skein-json-world -p skein-llm -p skein-forge -p skein-oauth -p smith-protocol-llm`; fuzzy, `heavy cargo nextest run --profile fuzzy -p skein-json-world`. `python3 -I -B tools/citations/citations.py --base origin/main` before you push. Commit and push. This branch merges on its own, after `writer-runs`, so that a red points at it: don't carry anything of another task's on it.

**A new test must bite.** Break each, run the rows and the sweep, see them fail for its reason, restore:

- `begun` at a run's end left at its first byte;
- the byte past `Strict`'s limit one off;
- `after_plain` taking `0x80`;
- `put` retaining past `Capped`'s cap.

## Report

As the implementer's text says, and: what each bite broke and what failed; each sweep's time alone; anything `memory.rs` showed about `Intake`; the calls you made in `Buffer::run`; anything you saw in `skein_llm::wire::json` or a tree's decoders that looks wasteful, by name, unchanged.
