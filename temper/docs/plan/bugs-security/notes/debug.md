# bugs-security: a Debug that hides secret bytes

Every task that replaces a derived `Debug` names this note, beside `notes/common.md`.

- **A canary in both forms.** A derived `Debug` prints a `Box<[u8]>`, a `[u8]` or a `List<u8>`
  as a list of numbers (`[99, 97, 110, ...]`), not as text, so a test that looks only for the
  canary's text proves nothing. It asserts the canary absent as text (`"canary-7f3a9c"`) and as
  its bytes' `Debug` (`alloc::format!("{:?}", CANARY)`, the list of numbers), as
  `skein/crates/skein-io/src/tests/process.rs`'s `spawn_debug_prints_env_names_without_values`
  does; and it asserts what replaces it (the length, or the count) present.
- **The test fails first** on the old code, and the report quotes the failing assertion.
- **The hand-written form keeps the derived one** for every field and variant without a secret
  (`notes/common.md`): `f.debug_struct` with the type's or variant's name, the same fields in
  the same order, each printed by its own `Debug`. The secret field keeps its place, printed as
  its length under the name the spec gives. A test that formats one variant without a secret
  holds it to a literal written from the old code's output, and passes on both.
- **Every `match` lists every variant** (`skein/AGENTS.md`; the same holds in every tree's step
  crates): no `_`, no `matches!`.
- **The impl's comment** says what it hides and why, in a line, citing the design section; no
  history.
