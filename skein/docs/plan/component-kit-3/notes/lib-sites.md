# component-kit-3: moving a crate onto lib's primitives

Every `ls-*` task cites this note beside `notes/common.md`. It moves one
crate's private copies of lib's primitives onto lib's items
(`skein/docs/design/draft/component-kit.md`, 3.4). The items, as task
`lib-contract` landed them (read their docs in `skein/crates/skein-lib/src/`
before you start):

| Copy | lib's item |
|---|---|
| `earlier`/`earliest` over `Option<Time>`, or an inline `if next.is_none() \|\| Some(t) < next { next = Some(t) }` | `skein_lib::earliest(left, right)`; the `&mut` form is `*next = earliest(*next, Some(t))` |
| `for _ in 0..q.len() { drop(q.pop()); }` | `q.clear()` (`Queue::clear`, `Stack::clear`) |
| `enum Lifecycle { Live, Closing, Aborting, Closed }` | `skein_lib::Lifecycle`, with `close()` and `abort()` where the copy writes those transitions by hand |
| a walk over a slab by hand (a linked list through its entries, or a side list of ids) | `Slab::iter`, `Slab::iter_mut` |
| a nibble written as a hex digit | `skein_lib::hex::digit`, or `hex::pair` for a byte |
| a hex digit read | `skein_lib::hex::value` (either case), `hex::lower` (lowercase only) |
| `1*DIGIT` read as a `u64` | `skein_lib::Decimal::parse` |

## How

- The copy goes, with its tests if lib's tests cover the same contract,
  and every call site calls lib's item. A `pub(crate)` copy other modules of
  the crate call goes too: each caller calls lib.
- Keep observable behaviour (`notes/common.md`, "Drift"): an uppercase
  writer keeps uppercase (`hex::digit(n).to_ascii_uppercase()`), a
  lowercase-only reader uses `hex::lower`, and so on.
- A walk replaced by `Slab::iter` visits in slot order. If the crate's walk
  keeps another order (insertion, through `next` links) and a test or its
  design section depends on it, keep that walk, say so, and move only the
  walks whose order nothing observes.
- In step code, no closures: `for (id, entry) in slab.iter() { ... break; }`
  to find, then act on the id after the loop when the action needs `&mut self`.
- From `skein/`: `python3 -I -B scripts/old_forms.py --write` (if the
  `counts` task has merged; if the forms are not there yet, skip it and say
  so). Each count you touch falls by what you removed.

## Checks

A departure from `notes/common.md`'s check list: no public item changes,
so the crate's own tests are enough; its dependents need no run.
`cargo fmt`; `heavy cargo clippy -p PKG --all-targets -- -D warnings`;
`heavy cargo nextest run -p PKG` (the focused profile).

## Report

Each copy removed (file, function), what replaced it, any drift kept and
how, the counts before and after, and the crate's test count.
