# DEFLATE

Provisional, 2026-11-15. `skein-deflate`: raw DEFLATE (RFC 1951), a
whole input to its compressed bytes and back, bounded, with no
allocation of its own beyond what its caller hands it. skein-kv's
payload logs compress their records with it (`kv.md`, section 6),
skein-http's gzip member wraps its output (`../net/http/client.md`,
section 1), and skein-forge reads a ZIP entry with it
(`../forge/forgejo.md`, 1.2). It is the codec temper's
`draft/store-shape.md`, section 11 left to its milestone S7 to pick:
DEFLATE, because skein already held an encoder and a decoder of it
under the strict subset, and its ratio on the JSON and text a store
keeps is about twice LZ4's.

## 1. In one page

- **A step crate,** `no_std`, under the strict subset
  (`../../foundation/programming-model.md`, 10), depending on
  `skein-lib` alone. Pure functions: no machine, no io, no state between
  calls.
- **Encoding,** `bound` and `encode` (2): the worst case of the output
  for an input's length, and the input's raw DEFLATE written into the
  front of the room the caller set aside. One input, one output.
- **Decoding,** `decode` (3): exactly one raw DEFLATE stream, the whole
  input, into a list whose capacity is the most it may grow to.
- **Moved, not written again:** the encoder is the one skein-http's gzip
  encoder held (its blocks, its hash of 2^15 heads over four bytes, its
  32 KiB window, each block written the shortest of its own Huffman
  codes, the fixed codes or stored), and the decoder the one
  skein-forge's ZIP reader held privately. skein-http keeps the gzip
  member's header, CRC-32 and trailer around `encode`; skein-forge keeps
  its ZIP framing and CRC-32 around `decode`.

## 2. Encoding

```rust
pub fn bound(length: usize) -> Option<usize>;            // the worst case of `encode`'s output; None past usize
pub fn encode(input: &[u8], out: &mut [u8]) -> Option<usize>; // the input's raw DEFLATE at `out`'s front, its length
```

- **Never past the bound:** a block is never longer than stored, so the
  output of an input of `length` bytes is at most `bound(length)`, and
  `encode` with room of at least that writes the whole stream and
  answers its length. Room short of it may answer none.
- **Deterministic:** the same input gives the same bytes, so a golden
  output holds them.
- **An empty input** is one final fixed-Huffman block holding only its
  end of block, the two bytes `03 00`, as skein-http's encoder gave
  and its gzip members keep.
- **What it costs:** a pass of the input, a table of 2^15 heads, and a
  block's parse at a time, as skein-http's encoder did; its stack and
  heap are what that encoder measured.

## 3. Decoding

```rust
pub fn decode(input: &[u8], out: &mut List<u8>) -> Option<()>; // `out` cleared, then the stream inflated into it
```

- **Exactly one stream, the whole input:** a final block ending in the
  input's last byte; trailing bytes, a malformed block or code, a
  distance before the output's start, or output past `out`'s capacity,
  each answer none, with `out` holding whatever was inflated before.
- **Bounded by its caller:** `out`'s capacity is the most it grows to,
  so a caller that knows the inflated length (a payload log's frame
  says it, `kv.md`, section 6) makes `out` of that capacity and checks
  the length it got.

## 4. Tests

- **Step tests,** in the crate: round trips over empty, short,
  repetitive, random and block-crossing inputs, each output within its
  bound; a golden output for a fixed input, as skein-http's tests held;
  each refusal of 3 (trailing bytes, a truncated stream, a bad code, a
  distance too far, output past capacity).
- **Fuzzed:** generated inputs round-trip within their bound, and
  generated streams decode or are refused, never past their capacity.
- **Its callers keep theirs:** skein-http's gzip tests and skein-forge's
  ZIP tests stay as they are and pass over the moved code.
