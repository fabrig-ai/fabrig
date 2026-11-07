+++
id = "writer-runs"
part = "writer"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-json/src/writer.rs",
  "skein/crates/skein-json/src/string.rs",
  "skein/crates/skein-json/src/tests/writer.rs",
  "skein/testing/skein-json-kit/src/generate.rs",
  "skein/tests/json/tests/suite/fuzzy/writer.rs",
  "skein/docs/design/json.status.md",
]
effort = "high"
minutes = 75
+++

# The writing pass stops measuring, and a plain run costs no call per byte

## Goal

Causes 1, 2 and 3 of the request. `Encoder`'s writing pass runs no `escaped_len`; `escaped_len` and `write_escaped_content` take a plain run with no call per byte. The same bytes come out, with the same refusals.

Acceptance:

- `put_string` and `string_parts` call `escaped_len` on the measuring pass only. The writing pass writes what it's given (`skein/docs/design/json.md`, 4).
- `escaped_len` finds each plain run with `string::after_plain` and adds its length once. `saturating_add` and `len_u64` per run are fine; nothing in a plain run is per byte.
- `write_escaped_content` finds each run of bytes written as they are with a loop of the same form, and calls `escaped` only for a byte that needs an escape.
- `writer.rs`'s head says what `skein/docs/design/json.md`, 4 says (see *Steps*).
- The tests below hold and each new one bites.
- The two **Not yet** lines below are gone from `skein/docs/design/json.status.md`.

## Inputs

- `skein/docs/plan/json-strings/inputs/request.md`: causes 1 to 3, and *The writing pass stops measuring*.
- `skein/docs/design/json.md`, 3.4, 4, 6 and 7.
- `skein/docs/plan/json-strings/notes/common.md`, `skein/docs/plan/json-strings/notes/contracts.md`. The latter holds the check of every caller of `Encoder::write`, already done: it needs no repeating, and it's why this task stops measuring outright.

## Owns and may change

Owns the files in the header. In `string.rs` you add `after_plain` and nothing else: `decode-runs` owns the rest of that file and starts after you. In `generate.rs` you add `runs` and nothing else.

`skein/docs/design/json.status.md`: remove these two lines, and change nothing else there (a departure, `notes/common.md`):

- `4, 7: the writing pass measures each string and checks its text. ...`
- `6: the step rows and the sweep of plain runs written.`

Readers: `put_json_spelling` keeps `escaped` as it is. `escaped` and `Escaped` stay, since it and the slow paths read them.

Options, each decided:

- **`Sink::Refused` in `put_string` and `string_parts`:** before_value's or key's early return already keeps a refused pass out. Keep the arm, as `match` lists every variant.
- **`Escaped::Plain` in `write_escaped_content`:** the run's loop has already taken every plain byte, so the arm is `unreachable!("a run ends at a byte to escape")`.
- **A borrow the sketch's match can't take:** `self.refuse` inside a `match &mut self.sink` may need the refusal returned out of the match first. Shape it as the compiler asks.
- **A lint the slice pattern trips** (`clippy::pattern_type_mismatch` or the like): write it as the lint wants (`*rest`, `ref after`) if that adds no call, otherwise a scoped `#[expect]`.

## Steps

**`writer.rs`.** The shape, the choices inside yours:

```rust
fn put_string(&mut self, text: &[u8]) {
    match &mut self.sink {
        Sink::Measuring { len, .. } => match escaped_len(text) {
            Some(escaped) => *len = len.saturating_add(escaped),
            None => self.refuse(Refusal::Text),
        },
        Sink::Refused(_) => {}
        Sink::Writing(writer) => write_escaped(writer, text),
    }
}

fn escaped_len(text: &[u8]) -> Option<u64> {
    let mut utf8 = Utf8::Between;
    let mut len: u64 = 2;
    let mut rest = text;
    loop {
        // Between characters, a plain run at once. Match on `utf8`: a derived `==` is a call.
        // `after_plain(rest)`, and `len` gains the difference of the two slices' lengths.
        let [byte, after @ ..] = rest else { break };
        // As today, for one byte: `utf8.next`, `escaped`, its width.
        rest = after;
    }
    match utf8 {
        Utf8::Between => Some(len),
        Utf8::Within { .. } => None,
    }
}

/// What follows the bytes at the head of `text` written as they are: all but
/// the escaped, non-ASCII included, since the writing pass keeps no state of UTF-8.
fn after_unescaped(text: &[u8]) -> &[u8]    // as `after_plain`: 0x20..=0x21 | 0x23..=0x5B | 0x5D..=0xFF

fn write_escaped_content(writer: &mut Writer, text: &[u8]) {
    // After `after_unescaped(rest)`: `put` the run if it isn't empty; the next byte, if any,
    // through `escaped` and `put`; go on after it.
}
```

`string_parts` moves its measuring into the measuring arm: on a writing pass the parts go straight to `write_escaped_content`. A part that's not UTF-8 and a character split between two parts stay refused when measured, as today.

`writer.rs`'s head, after "is a bug, asserted.", gains:

```text
//! The writing pass measures nothing and checks no string's text: it writes
//! what it is given. Text that differs between the passes at the same length
//! is a bug of the caller's code that nothing asserts; it is written as it
//! is, UTF-8 or not (json.md, 4).
```

**Step tests,** `skein/crates/skein-json/src/tests/writer.rs`:

- A reference of the test's own, the old `escaped_len` a byte at a time through `Utf8::next`: `per_byte_len(text) -> Option<u64>`, and `per_byte_spelling(text) -> Vec<u8>`, quoted and escaped a byte at a time with its own plain match of the byte.
- `every_byte_at_a_runs_edge_is_measured_and_written_as_a_byte_at_a_time`: for every byte value, `[b]`, `[a, b]`, `[b, a]`, `[a, a, b, a, a]`, `[0xC3, 0xA9, b]` and `[b, 0xC3, 0xA9]`: where `per_byte_len` says `None`, `Refusal::Text`; otherwise the document has that length and that spelling. One test, rows in a loop.
- `string_parts_measures_its_parts_as_one_string`: an empty part, and a run across two parts, measured and written; a part that's not UTF-8 and `é` split between two parts, refused.
- `a_writing_pass_with_invalid_utf8_is_a_bug` goes, since it no longer holds. Two take its place:
  - `a_writing_pass_with_a_bad_number_is_a_bug`, `#[should_panic(expected = "a document measured first is never refused")]`, which keeps `refuse`'s arm covered: `Encoder::write(1, &WRITER)`, then `json.number(b"x")`.
  - `a_writing_pass_writes_its_text_unchecked`: `Encoder::write(4, &WRITER)`, `json.string(b"\xC0\x80")`, and `finish` gives `"\xC0\x80"` quoted. It pins what `json.md`, 4 says.

**The sweep,** `skein/tests/json/tests/suite/fuzzy/writer.rs`:

`text_in_runs_is_written_as_a_naive_reading_has_it`: about 5,000 rounds from a seed. `generate::runs` with `pieces` in 0..6 and `longest` in 0..48; half the rounds with breaks that are valid text (`"`, `\`, `\n`, `\x08`, `\x00`, `\x1f`, `é`, `€`, `😀`), half with those and what isn't UTF-8 (`\x80`, `\xc3`, `\xc0\x80`, `\xed\xa0\x80`, `\xf5`). Through `skein_json_world::write`, one `Token::String`:

- refused with `Refusal::Text` exactly when `std::str::from_utf8` refuses;
- otherwise the document is the test's own naive spelling, a byte at a time. `write` already asserts the length measured is the length written.

It asserts both fell, more than 1,000 of each. If it takes over 2 s alone on your builder, lower the rounds; say what it took.

**Narrow tests, then the close:** `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-json -p skein-json-kit -p skein-json-world --all-targets -- -D warnings`; focused, `heavy cargo nextest run -p skein-json -p skein-json-world -p skein-llm -p skein-forge -p skein-oauth -p smith-protocol-llm`; fuzzy, `heavy cargo nextest run --profile fuzzy -p skein-json-world`. `python3 -I -B tools/citations/citations.py --base origin/main` before you push. Commit and push.

**A new test must bite:** widen `after_plain`'s range at one edge (`0x5D` to `0x5C`, taking the backslash), and separately put `escaped_len`'s run length one off; run the step test and the sweep and see each fail; restore. Narrowing the range is no break: the bytes it leaves out fall to the state machine, and only cost more.

## Report

As the implementer's text says, and: what the step test and the sweep broke on; the sweep's time alone; whether `after_plain`'s body kept its form; anything you saw in `skein_llm::wire::json` or a tree's decoders that looks wasteful, by name, unchanged.
