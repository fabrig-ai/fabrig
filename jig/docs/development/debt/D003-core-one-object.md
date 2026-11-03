# D003: The core is one object of 83 fields spread over 50 files

- **Kind:** complexity
- **Where:** `jig-domain-core/src/lib.rs` (`Core`, `Core::new`), `jig-domain-core`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T8, A1 F8, F11, A2 F1)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 14: `Core::new` untouched)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (a reader can find a concept's state in one place)
- **Size:** L
- **Status:** open
- **Paid when:** `Core`'s state is grouped into owned parts by concept, each with its own `impl` in its own module, and `Core::new` builds them rather than every field.
- **Measure:** `awk '/^pub struct Core \{/,/^\}/' jig/crates/jig-domain-core/src/lib.rs | grep -cE '^\s+(pub(\(crate\))? )?[a-z_]+:'` (83 at base)

## What

The re-tree moved the core's modules into folders (`jig-domain-core/src/acts`, `jig-domain-core/src/reads`, `jig-domain-core/src/routing`,
`jig-domain-core/src/settings`, …), but the object did not split:

- `Core` has 83 fields, 23 of them `pub` (98 and 37 at the review);
- `impl Core` blocks sit in 50 files (`rg -c '^impl Core' jig/crates/jig-domain-core/src | wc -l`; 41 at the review);
- `Core::new` is 191 lines.

`Config.settings` is a typed `Settings` now (`lib.rs`, line 149), which paid that part of item 14.

## Why it matters

Every concept reads and writes the same fields, so a change to one concept's state can
reach code in any of 50 files, and a reader cannot see what a module owns.

## What pays it back

Owned sub-structs (runs, flights, settings, views, funding) with their methods, after
D002 removes the token maps that tie them together.
