# json-strings: the shapes that cross the tasks

`writer-runs` creates both of these, and `decode-runs`, which needs it, builds on them.

## A plain byte, and the loop that finds a run

A plain byte is `skein/docs/design/json.md`, 3.4's: ASCII a string holds as it is. In `skein/crates/skein-json/src/string.rs`:

```rust
/// What follows the plain bytes at the head of `bytes` (json.md, 3.4): ASCII
/// a string holds as it is, which needs no escape and no state of UTF-8. A
/// slice pattern and a match make no call per byte (json.md, 7).
pub(crate) fn after_plain(bytes: &[u8]) -> &[u8] {
    let mut rest = bytes;
    while let [byte, after @ ..] = rest {
        match *byte {
            0x20..=0x21 | 0x23..=0x5B | 0x5D..=0x7F => rest = after,
            0x00..=0x1F | b'"' | b'\\' | 0x80..=0xFF => break,
        }
    }
    rest
}
```

It returns what follows rather than a count, so no loop counts per byte: a caller works out the run's length once, from the two slices' lengths.

The name and the signature bind. The body may take another form only if it makes no call per byte and is cheaper unoptimized; say so in the report.

## Text in runs, for the sweeps

In `skein/testing/skein-json-kit/src/generate.rs`:

```rust
/// Text in runs (json.md, 6): `pieces` times, a run of up to `longest` plain
/// bytes and then one of `breaks`; then a last run.
#[must_use]
pub fn runs(rng: &mut Rng, breaks: &[&[u8]], pieces: u32, longest: u32) -> Vec<u8>
```

The run's bytes are drawn from the edges of the plain range and a few inside it: `b" !#[]~\x7fa0Z/"`. Each sweep passes its own `breaks`, since what breaks a run differs between decoded text (the writer's) and a document's spelling (the tokenizer's).

## What the check of the writing pass's callers found

`skein/docs/design/json.md`, 4 and 7 rest on this; it was checked over the 149 calls of `Encoder::write` in the repository on 2026-11-06.

- **No caller reaches a writing pass unmeasured** but three tests, in `skein/crates/skein-json/src/tests/writer.rs`, which pass a literal length on purpose. Every other length comes from a measuring pass of the same data: directly; through a helper (`smith-protocol-program`'s `raw_length`, `skein-fake-forge-http`'s `git::write::finish`); through `skein-llm`'s `MeasuredRequest`, which owns the input it measured; or as the sum of parts measured by the same functions (`skein-fake-forge-http/src/git/store.rs`, its fixed envelope and commas aside).
- **No caller's text differs between its passes.** Each runs one function, or the same calls, over shared references. The one mutable state a pass is handed is temper's `Cut` budget and `Window`: `projects_json.rs` resets `room` and `left` before its writing pass, `offers.rs` makes a new one, and the rest hold `room: 0` on both.
- So the writing pass simply stops measuring, as cause 1 of the request says. A writing pass that writes more or less than was measured stays asserted, by `skein_lib::Writer`. What goes unasserted is text that differs at the same length (`skein/docs/design/json.md`, 4).
