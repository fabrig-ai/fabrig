# DEFLATE

Provisional, 2026-11-26. `skein-deflate`: raw DEFLATE (RFC 1951), a
whole input to its compressed bytes and back, bounded, with no
allocation of its own beyond what its caller hands it. skein-kv's
payload logs compress their records with it (`kv.md`, section 6),
the crate's own gzip member wraps it (`gzip.md`), and skein-forge
reads a ZIP entry with it (`../forge/forgejo-operations.md`, 1). It is the codec temper's
`draft/store-shape.md`, section 11 left to its milestone S7 to pick:
DEFLATE, because skein already held an encoder and a decoder of it
under the strict subset, and its ratio on the JSON and text a store
keeps is about twice LZ4's.

## 1. In one page

- **A step crate,** `no_std`, under the strict subset
  (`../../foundation/programming-model.md`, 10), depending on
  `skein-lib` alone. Pure functions: no machine, no io, no state between
  calls.
- **Encoding,** `bound`, `encode` and `encode_best` (2): the worst case
  of the output for an input's length, and the input's raw DEFLATE
  written into the front of the room the caller set aside, at the fast
  level or the best. One input, one output at each level.
- **Decoding,** `decode` (3): exactly one raw DEFLATE stream, the whole
  input, into a list whose capacity is the most it may grow to.
- **Moved, not written again:** the encoder is the one skein-http's gzip
  encoder held (its blocks, its hash of 2^15 heads over four bytes, its
  32 KiB window, each block written the shortest of its own Huffman
  codes, the fixed codes or stored), and the decoder the one
  skein-forge's ZIP reader held privately. The gzip member's header,
  CRC-32 and trailer, which skein-http kept around `encode`, are the
  crate's `gzip` module (`gzip.md`); skein-forge keeps its ZIP framing
  and CRC-32 around `decode`.

## 2. Encoding

```rust
pub fn bound(length: usize) -> Option<usize>;                      // the worst case of either level's output; None past usize
pub fn encode(input: &[u8], out: &mut [u8]) -> Option<usize>;      // the fast level: the input's raw DEFLATE at `out`'s front, its length
pub fn encode_best(input: &[u8], out: &mut [u8]) -> Option<usize>; // the best level, the same way
```

- **Two levels, one way of writing blocks:** both parse the input into
  literals and matches, then write each block of at most 65,535 input
  bytes the shortest of its own Huffman codes, the fixed codes or
  stored. They differ in the parse alone.
- **The fast level,** `encode`, for what is compressed as often as it
  is written, a store's records and a request's body: each position's
  one latest candidate of the same four bytes, the match taken
  greedily. Compressing must cost far less than the bytes it saves.
- **The best level,** `encode_best`, for what is compressed once and
  sent many times, the browser bundle's compressed forms
  (`../ui/web/bundle.md`, 2): each position's earlier candidates
  followed back through the window along a chain, at most 4,096 of
  them, and a match taken only where the next position's is no longer
  (lazy matching), as zlib's best level does, so that its ratio is
  about zlib's best. Its bytes are not the fast level's; `decode`
  takes both.
- **Never past the bound:** a block is never longer than stored, so the
  output of an input of `length` bytes is at most `bound(length)` at
  either level, and each with room of at least that writes the whole
  stream and answers its length. Room short of it may answer none.
- **Deterministic:** the same input gives the same bytes at each level,
  so a golden output holds them.
- **An empty input** is one final fixed-Huffman block holding only its
  end of block, the two bytes `03 00`, at both levels, as skein-http's
  encoder gave and its gzip members keep.
- **What it costs:** at the fast level, a pass of the input, a table of
  2^15 heads, and a block's parse at a time, as skein-http's encoder
  did; its stack and heap are what that encoder measured. The best
  level adds the chain, an earlier position for each of the window's
  2^15, on the stack as the heads are, and at most 4,096 candidates
  compared at a position, so its time stays bounded by the input's
  length; it allocates nothing either.
- **Turned down:** one level for every caller, which would have the
  store and the requests pay the chain on every write, or the bundle
  the fast level's ratio; a level as a parameter of `encode`, which
  changes every caller for the one that wants the other; and the
  outside `flate2`, which the bundler used before the user wanted it
  gone (2026-11-26).

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

- **Step tests,** in the crate: round trips at both levels over empty,
  short, repetitive, random and block-crossing inputs, each output
  within its bound; a golden output for a fixed input at the fast
  level, as skein-http's tests held; the best level no longer than the
  fast on repetitive text, and matches reaching back across a block's
  start and to the window's far end; each refusal of 3 (trailing
  bytes, a truncated stream, a bad code, a distance too far, output
  past capacity).
- **Fuzzed:** generated inputs round-trip within their bound at both
  levels, and generated streams decode or are refused, never past their
  capacity.
- **Its callers keep theirs:** the gzip member's tests (`gzip.md`, 3)
  and skein-forge's ZIP tests pass over this code.
