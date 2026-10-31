# The lib family: private copies onto lib's primitives

For the lib-sites tasks. Read `common.md` first.

## The shared items

Landed by `kit-lib` and `kit-slab` in `skein/crates/skein-lib/`; read
their `///` contracts on your base.

- `skein_lib::earliest(a: Option<Time>, b: Option<Time>) -> Option<Time>`:
  the earlier of two deadlines, either absent.
- `Queue::clear` and `Stack::clear`: drop every item, keeping storage.
- `skein_lib::Lifecycle { Live, Closing, Aborting, Closed }`
  (programming-model.md, 5.2), with `Clone, Copy, PartialEq, Eq, Hash,
  Debug`.
- `skein_lib::hex::digit(nibble: u8) -> u8`: a nibble's lowercase ASCII
  digit; a nibble above 15 is a bug. `skein_lib::hex::value(byte: u8) ->
  Option<u8>`: an ASCII hex digit's value, either case, or `None`.
- `Decimal::parse(digits: &[u8]) -> Option<u64>`: `1*DIGIT` as a `u64`;
  `None` for empty input, another byte, or a value past `u64::MAX`.
  `Decimal::of(n).as_bytes()` writes a number's digits.
- `Slab::first(&self) -> Option<Id<T>>` and `Slab::after(&self, id:
  Id<T>) -> Option<Id<T>>`: the live (unretired) entities in slot order,
  as a cursor that holds no borrow (`kit-slab`).

## What every task does in the files it owns

- `fn earliest` / `fn earlier` over `Option<Time>`, in either shape (two
  options in, or `&mut Option<Time>` and a time), and inline copies of
  the same merge become `skein_lib::earliest`; the `&mut` shape becomes
  `*next = earliest(*next, Some(at))`.
- `for _ in 0..q.len() { q.pop(); }` (or `drop(q.pop())`, `let _ =
  q.pop()`) becomes `q.clear()`.
- A private `enum Lifecycle { Live, Closing, Aborting, Closed }` becomes
  `skein_lib::Lifecycle`.
- A `fn hex` that maps a nibble to its digit, or a digit to its value,
  becomes `hex::digit` / `hex::value`; a `fn decimal` that parses ASCII
  digits becomes `Decimal::parse`; one that writes a number's digits
  becomes `Decimal::of(n).as_bytes()` at its callers.
- Lower the counts with `--write`.

## Pre-approved defaults

- **Upper-case hex:** a site that writes upper-case digits because its
  wire requires it keeps its function; name it.
- **A digit parse that accepts only lowercase**, where a test or the
  wire pins that upper case is refused: keep the refusal at the call site
  (`hex::value` and a check of the byte's case), or keep the function and
  name it. Where nothing pins it, take `hex::value`.
- **A decimal parse that differs** (a sign, leading zeros refused, empty
  input accepted, a width other than `u64`): keep its rule only if a test
  or the wire pins the difference, as a check beside `Decimal::parse` or
  a narrowing `try_from` of its result; otherwise take `Decimal::parse`
  as it is. Name each kept difference with what pins it.
- **Not a copy:** a function with one of these names that does something
  else (reads a document's tokens, generates random digits, encodes a
  whole buffer, is a test's independent reference) keeps its name and its
  work; where it maps one digit inside, it calls lib's primitive.
  `decisions.md`'s targets allow exactly the ones named there; another is
  a `Question:` only if you would keep it.
- **A pop loop that uses what it pops** is not a copy: leave it.
- **A crate whose lints refuse a lib item's use** (a `disallowed_methods`
  entry, say): stop with a `Question:`.
