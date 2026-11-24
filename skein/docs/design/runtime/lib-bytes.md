# lib: bytes

## 1. Bytes

- **Owned bytes** are a `Box<[u8]>` allocated at its final length.
  `bytes::copy_of` copies one, for data a domain keeps and also sends;
  `bytes::concat` copies parts, in order, into one box of their summed
  length; `bytes::zeroed` makes one for the side below to fill, a receive
  buffer.
  The byte search (`bytes::find`, `find_from`, `count`) runs in linear
  time and allocates nothing. `bytes::is_utf8` says whether bytes are UTF-8,
  `bytes::utf8_prefix` cuts valid UTF-8 at a character boundary at or
  below a byte count, and `bytes::utf8_replaced(bytes: &[u8]) -> Box<[u8]>`
  makes any bytes text of the same length: each byte that is not part of
  a UTF-8 character (`core::str::from_utf8`'s reading, a sequence cut
  short included) becomes `?`, and UTF-8 comes back unchanged. These are
  the one place skein and its users check text. Keeping the length keeps
  every bound the bytes already met, so a step that must send bytes it
  cannot vouch for as text (a repository's file, a job's log) sends them
  so rather than being refused whole.
- **`bytes::decimal(digits: &[u8]) -> Option<u64>`** reads `1*DIGIT` as a
  `u64`, or returns `None` for no digits, a byte other than an ASCII digit,
  or a number past `u64::MAX`; leading zeros are read as zeros.
- **`bytes::hex_value(byte: u8) -> Option<u8>`** is a `const fn` returning
  the value of an ASCII hex digit in either case, or `None` for any other
  byte.
- **`Reader`** reads a delivery without trusting any length in it: every
  read (`u8`, `u16`, `u32`, `u64`, `bytes`, `skip`) returns an `Option`,
  and a short one is a framing error for the caller, never a panic.
- **`Writer`** builds a message into a box whose length was computed
  first: `Writer::new(len)`, `put` the fields, `finish()`. A write past
  the end is refused whole (`Overflow`), writing nothing; finishing short
  is an assertion.
- **`Buffer`** builds bytes whose length is learnt as they are put, and
  is reused from one to the next: `Buffer::with_capacity(n)` allocates
  its n bytes once (`Buffer::price(n)`), `put` appends or refuses
  whole past the capacity (`Overflow`), writing nothing, and `set_u32`
  back-patches a big-endian `u32` over four bytes already put, a count
  or a length known only later (asserted in range). `len`, `room`,
  `capacity`, `as_slice`; `truncate` and `clear` for reuse; `to_boxed`
  copies the bytes out at their exact length, the buffer kept. Its
  `Debug` shows its length and capacity, never its bytes. A patch builds
  each record in one, a secret's bytes among them (../data/codec/patches.md).
- **`Tail`** keeps a stream's newest bytes in a ring: `Tail::with_capacity(n)`
  holds at most n, `push` and `put` drop the oldest once it is full, and a
  capacity of zero keeps nothing. `get` reads a byte in arrival order,
  `to_boxed` copies them out at their exact length, `into_boxed` hands them
  over in the ring's own buffer, and `clear` keeps the capacity.
- **`Decimal`** holds the decimal digits of a count, for a step that
  writes a number as text without formatting: measure it, then put it.
- **`bytes::ip_text(ip: IpAddr) -> Box<[u8]>`** writes an address as
  text without formatting, byte for byte as core's `IpAddr` prints it:
  an IPv4 address in dotted decimal, an IPv6 one in RFC 5952's text
  (lowercase hex, the longest run of two or more zero groups, the first
  of equals, as `::`, an IPv4-mapped one as `::ffff:a.b.c.d`). It
  leaves a mapped address as given: the caller reads one as its IPv4
  first where it means to.
- **`Secret`** holds a secret's owned bytes: a credential, a token, a
  key. Its `Debug` shows its length only, never its bytes, so a record
  that holds one prints, logs and traces without it (../data/codec/README.md, section 3).
  Its owner reads the bytes with `expose`, which makes each use visible in
  review. Clone, equality and `Hash` are over its bytes, as a record's
  derives need; it neither zeroes nor locks memory (io/files.md, 11).
- **A carrier prints lengths.** A record carrying a peer's bytes prints them
  as their length in `Debug`, never a byte of them, and a header prints its
  name and its value's length, so a credential passing through prints and
  traces without its bytes (http.md, 2; ../net/channel.md, 3). Only what ../oauth/README.md, 4
  names as no credential prints as it came.
- **Widening:** `length::usize_of` widens a `u32` count to an index and `length::len_u64` a length to a count, so no step writes `try_from` for either.
- **A length added up with checks:** `length::Measure` starts at `Measure::ZERO` and `add`s lengths: past a `usize` it stays `None`. `total` gives the sum, and `total_u32` the sum a `u32` holds. A writer measures a head or a frame with it before `Writer::new`.

### 1.1 Base64

`lib::base64` encodes bytes as base64 text and decodes it back (RFC
4648), for every crate that carries bytes in text: PKCE's challenge and a
token's claims (../oauth/README.md), PEM's certificates (shell-startup.md, 1.2), a
browser's screenshots (../testing/browser.md), a record in an event's data. skein's
crates use it and keep no copy of their own.

- **Two alphabets, each with its padding:** `Alphabet::Standard` (`+`,
  `/`), padded with `=` to whole groups of four; and `Alphabet::Url` (`-`,
  `_`), unpadded, as JWTs and PKCE use it (RFC 7515, RFC 7636).
- **Measured, then written.** `encoded_len` gives the text's length
  before anything is set aside, and `encode` writes into a `Writer` with
  that much room left (section 1): nothing allocates but the box its
  caller sized.
- **Decoding is bounded and canonical.** `decode` takes a maximum of
  decoded bytes and refuses past it before setting anything aside. It
  refuses, with `base64::Error`: a symbol outside the alphabet; padding
  where the alphabet takes none, or anywhere but the end; a length no
  encoding has; trailing bits that are not zero. So one value has one
  encoding.
- **In pieces.** A `Decoder` takes text in pieces, carrying what a piece
  leaves of a group, within the same maximum and the same refusals, so a
  reader that skips what surrounds the text (PEM's line breaks) or
  receives it in chunks (an event's data) decodes it without copying it
  whole first.

### 1.2 SHA-256

`lib::sha256` is SHA-256 (FIPS 180-4), for every crate that names bytes
by their content: io's content versions (io/files.md), PKCE's
challenge (../oauth/README.md), a browser bundle's digest directory (../ui/web/README.md),
replay's state digests (`lib-containers.md`, 1.1), and a service's own, such as hearth's,
whose domains name every input and output by it. skein's crates use it
and keep no copy of their own.

- **Streaming.** `Sha256::new()`, then `update(&[u8])` with each piece as
  it arrives, cut anywhere, then `finish(self) -> [u8; 32]`.
  `sha256::digest(&[u8]) -> [u8; 32]` is the one-shot form. The digest is
  its 32 bytes in the standard's order, as `sha256sum` prints them in
  hex.
- **No heap, no copy.** The state is eight words, one 64-byte block and
  the count of bytes hashed, so a step keeps one in its state and it
  allocates no heap. `update` compresses each whole block straight from
  the caller's slice, and copies only the part of a block a piece
  leaves: hashing costs the compression, not a copy.
- **`Clone`,** so a prefix common to several digests is hashed once and
  each goes on from its own copy. Its `Debug` shows the count of bytes
  hashed, not the state.
- **The standard's bound,** 2^64 − 1 bits, is asserted: no bounded step
  hashes that much.
- **Its users.** skein-io's `digest` module keeps its public
  API, `Digest([u64; 4])` and `digest(&[u8])`, as a thin layer over lib's
  (io/files.md): its words are the digest's 32 bytes read as four
  big-endian `u64`s. `skein-oauth`'s PKCE
  and `skein-ui-bundle`'s digests use lib's and keep no copy;
  `skein-ui-bundle`, a build tool in ordinary Rust (../ui/README.md, section 2),
  depends on lib for it.
- **HMAC.** `sha256::Hmac::new(key: &[u8])`, then `update(&[u8])` with
  each piece as it arrives, then `finish(self) -> [u8; 32]`, is
  HMAC-SHA256 (RFC 2104), for a service that checks a signed message as
  it streams. A key longer than a block is hashed first, as the standard
  says. Its state is two SHA-256 states, the inner seeded with the key's
  inner pad and the outer with its outer pad, with no heap. `Clone` as
  `Sha256`'s; its `Debug` shows the count of the message's bytes hashed
  alone (`Hmac { hashed: N }`), the pads not counted, never the key or
  its pads.
- **Comparing in constant time.** `sha256::equal(a: &[u8], b: &[u8]) ->
  bool` takes a time that depends on the lengths alone, never on where
  the bytes differ, for comparing a signature, a state or a token with
  the one expected. `skein-oauth`'s comparison of a returned state uses
  it and keeps no copy.

### 1.3 SHA-1

`lib::sha1` is SHA-1 (FIPS 180-4), for naming bytes as git names an
object in a SHA-1 repository: a blob's id is SHA-1 over `blob `, the
length in decimal, a NUL byte, then the bytes. It is never for a
security purpose: SHA-1 is broken for collisions, and nothing checks a
signature, a token or a state with it.

- **Streaming, as SHA-256's.** `Sha1::new()`, then `update(&[u8])` with
  each piece as it arrives, cut anywhere, then `finish(self) -> [u8; 20]`;
  `sha1::digest(&[u8]) -> [u8; 20]` is the one-shot form. The digest is
  its 20 bytes in the standard's order, as `sha1sum` and git print them in
  hex.
- **No heap, no copy.** The state is five words, one 64-byte block and
  the count of bytes hashed; `update` compresses whole blocks from the
  caller's slice and copies only the part of a block a piece leaves.
- **`Clone`,** with a `Debug` that shows the count of bytes hashed, not
  the state. The standard's bound, 2^64 − 1 bits, is asserted.
- **No HMAC and no comparison** of its own: a step that needs either
  takes SHA-256's (1.2).
- **Its users.** temper's engine, naming a file of a pull request by its
  blob id where the repository is SHA-1 (temper's
  `domain/forge/change/repairs.md`, a resolution's pull section).
