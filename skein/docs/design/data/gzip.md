# gzip

Provisional, 2026-11-26. `skein-deflate`'s `gzip` module: one gzip
member (RFC 1952) around a raw DEFLATE stream (`deflate.md`), written at
either level and read back whole, bounded as the raw codec is. skein-llm's
client compresses a request's body with it at the fast level
(`../llm/wire.md`, section 1.8), skein's bundler writes a bundle's
compressed forms with it at the best (`../ui/web/bundle.md`, 2), and the
tests that read those forms back read them with it.

## 1. In one page

- **A module of `skein-deflate`,** under the crate's rules (`deflate.md`,
  1): `no_std`, the strict subset, pure functions, no allocation beyond
  what the caller hands it.
- **Encoding,** `bound`, `encode` and `encode_best` (2): the member's
  worst case for an input's length, and the input's member at the fast
  or the best level written into the front of the room the caller set
  aside.
- **Decoding,** `decode` (2): exactly one member, the whole input, its
  CRC-32 and length checked, into a list whose capacity is the most it
  may grow to.
- **Moved, not written again:** the header, the CRC-32 and the trailer
  are those skein-http's gzip encoder held. Its module goes and its
  callers name this one, so a request body's member is byte for byte
  what it was.
- **One owner for the member:** skein-http's encoder wrote one and the
  bundler wrote another with the outside `flate2`, which the user wanted
  gone (2026-11-26); this module is the workspace's only writer and
  reader of a gzip member, but for the fake LLM provider's inflater, which
  is test code of its own on purpose (`../testing/fakes/llm.md`, 3.1).

## 2. The member

```rust
pub fn bound(length: usize) -> Option<usize>;                      // deflate's bound and the member's 18 bytes; None past usize
pub fn encode(input: &[u8], out: &mut [u8]) -> Option<usize>;      // the member, `encode`'s stream inside; its length
pub fn encode_best(input: &[u8], out: &mut [u8]) -> Option<usize>; // the member, `encode_best`'s stream inside; its length
pub fn decode(member: &[u8], out: &mut List<u8>) -> Option<()>;    // `out` cleared, then the member's input inflated into it
```

- **The header,** ten bytes (RFC 1952, 2.3): `1f 8b`, DEFLATE (8), no
  flags, no time (0), the extra flags 0 at the fast level and 2 at the
  best (the slowest compression, RFC 1952, 2.3.1), and an unknown system
  (255). No time and no name, so one input gives one member at each
  level.
- **The trailer,** eight bytes: the input's CRC-32 (RFC 1952, 8; ISO
  3309) and its length modulo 2^32, each little-endian.
- **Never past the bound:** `bound(length)` is `deflate::bound(length)`
  and 18; each encoder with room of at least that writes the whole
  member and answers its length, and room short of it answers none.
- **Decoding takes exactly one member, the whole input:** `1f 8b`, 8 and
  no flags, any time, extra flags and system; then exactly one raw
  DEFLATE stream ending eight bytes before the input's end
  (`deflate.md`, 3); then the CRC-32 and the length of what it inflated.
  Anything else answers none, with `out` holding whatever was inflated
  before: a header short or other, any flag set, a stream with bytes
  after it or cut short, a trailer short or not matching, or output past
  `out`'s capacity.
- **Bounded by its caller,** as `decode` is (`deflate.md`, 3): a caller
  that knows the inflated length makes `out` of that capacity and checks
  the length it got.
- **No flags read:** skein's members carry none, and neither did
  `flate2`'s, so a name, a comment, an extra field or a header CRC is
  refused rather than skipped. Reading them waits for a caller that
  reads members skein did not write.

## 3. Tests

- **Step tests,** in the crate: the fast member's golden bytes for a
  fixed input, as skein-http's tests held, checked once against the
  system's `gzip`; the CRC-32's check value (`123456789` gives
  `CBF43926`) and its table built again; round trips through `decode`
  at both levels over empty, short and block-crossing inputs, each
  within its bound; the best member's header saying 2; each refusal of
  2.
- **Fuzzed:** generated inputs' members decode to them at both levels,
  within their bound; generated changes to a member are decoded to the
  input or refused, never past `out`'s capacity.
- **Its readers keep theirs:** `skein/tests/http`'s fuzzy sweep of the
  member stays in its world, naming this module; skein's bundler's tests
  unpack every form it writes byte for byte with `decode`
  (`../ui/web/bundle.md`, 2), and `tests/ui-web`'s the forms its server
  answers (`../ui/web/testing.md`).

## 4. Later

- **The ZIP's CRC-32s:** skein-forge's ZIP reader and the fake forge's
  ZIP writer keep CRC-32s of their own, the same function as this
  module's; one owner would be this one.
