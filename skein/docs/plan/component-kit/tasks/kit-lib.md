+++
id = "kit-lib"
part = "kit-contract"
kind = "interface"
repo = "fabrig"
needs = ["ratchets"]
files = ["skein/crates/skein-lib/src/lib.rs", "skein/crates/skein-lib/src/time.rs", "skein/crates/skein-lib/src/decimal.rs", "skein/crates/skein-lib/src/hex.rs", "skein/crates/skein-lib/src/lifecycle.rs", "skein/crates/skein-lib/src/queue.rs", "skein/crates/skein-lib/src/stack.rs", "skein/crates/skein-lib/src/requests.rs", "skein/crates/skein-lib/src/tests.rs", "skein/crates/skein-lib/src/tests/**", "skein/tests/lib/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 40
+++
# lib's missing primitives, landed alone

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.1 (lib): the items every lib-site task moves onto. No site
outside skein-lib moves here.

## What to build

- `pub fn earliest(a: Option<Time>, b: Option<Time>) -> Option<Time>` in
  `time.rs`, exported from the crate root: the earlier of two, `None`
  only when both are. `requests.rs`'s private `earliest` (line ~718)
  moves onto it.
- `Queue::clear(&mut self)` and `Stack::clear(&mut self)`: empty the
  container, keeping its capacity and cap.
- `pub enum Lifecycle { Live, Closing, Aborting, Closed }` in a new
  `lifecycle.rs`, exported, deriving `Clone, Copy, PartialEq, Eq, Debug`
  (the union of the two copies, `skein-llm-connection/src/component.rs:24`
  and `skein-oauth-accounts/src/component.rs:122`), with a one-line doc
  per variant in programming-model.md 5.2's words.
- `pub mod hex` (new `hex.rs`): `pub const fn digit(nibble: u8) -> u8`
  (lowercase ASCII digit of the low four bits) and `pub const fn
  value(byte: u8) -> Option<u8>` (the value of an ASCII hex digit, either
  case; `None` otherwise).
- `Decimal::parse(digits: &[u8]) -> Option<u64>` in `decimal.rs`: one or
  more ASCII digits and nothing else, `None` on empty input, another
  byte, or overflow. HTTP's two identical copies
  (`skein-http/src/client/head.rs:281`, `server/head.rs:380`) are the
  behaviour to match.

## Acceptance

- Step tests beside each item (lib's `src/tests/`), each one behaviour:
  `earliest` over the four cases; `clear` then `push` up to the cap;
  `hex::value` over every byte against `digit`; `Decimal::parse` round
  trips `Decimal::of` at 0, 1, `u64::MAX`, and refuses empty, a sign, a
  space and `u64::MAX + 1`.
- `heavy cargo nextest run -p skein-lib`, then `heavy cargo check --tests
  -p skein-http -p skein-io -p skein-shell` (lib's direct dependents
  build unchanged).
- No count rises; none falls yet.
