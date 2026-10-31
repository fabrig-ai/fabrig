# The lib-site family: private copies onto lib's primitives

For the lib-sites tasks. Read `common.md` first.

## The shared items (landed by kit-lib and kit-slab)

- `skein_lib::earliest(a: Option<Time>, b: Option<Time>) -> Option<Time>`.
- `Queue::clear` and `Stack::clear`.
- `skein_lib::Lifecycle { Live, Closing, Aborting, Closed }`.
- `skein_lib::hex::digit(nibble) -> u8` (lowercase ASCII) and
  `skein_lib::hex::value(byte) -> Option<u8>`.
- `Decimal::parse(digits: &[u8]) -> Option<u64>` beside `Decimal::of`.
- `Slab::iter`, the occupied entities in slot order (kit-slab).

## What every site task does the same way

- `fn earliest` / `fn earlier` over `Option<Time>` (either shape: two
  options in, or `&mut Option<Time>` and a time) and inline copies of the
  same merge become `skein_lib::earliest`. The `&mut` shape becomes
  `*next = earliest(*next, Some(at))`.
- `for _ in 0..q.len() { q.pop(); }` (or `drop(q.pop())`, `let _ =`)
  becomes `q.clear()`.
- A private `enum Lifecycle { Live, Closing, Aborting, Closed }` becomes
  `skein_lib::Lifecycle`; derives it lacked are lib's.
- `fn hex` that maps one nibble to its digit, or one digit to its value,
  becomes `hex::digit` / `hex::value`; `fn decimal` that parses ASCII
  digits becomes `Decimal::parse`; one that writes a number's digits
  becomes `Decimal::of(n).as_bytes()`.
- Lower the counts with `--write`.

## Pre-approved defaults

- **Upper-case hex:** a site that writes upper-case digits (a wire that
  requires it) keeps its function and is named in the report.
- **A parse that differs:** a `decimal` that accepts a sign, leading
  zeros differently, an empty input or a width other than `u64` keeps
  its own rule only if a test or wire pins the difference; otherwise it
  moves onto `Decimal::parse` and narrows its result with a checked
  conversion. Name each kept one with what pins it.
- **Not a copy:** a function with one of these names that does something
  else (reads a document's tokens, generates random digits, encodes a
  whole buffer, is a test's independent reference) keeps its name and its
  work; where it encodes or decodes one digit inside, it calls lib's
  primitive. The count's target allows exactly these (`decisions.md`).
- **A pop loop that uses what it pops** is not a copy: leave it.
