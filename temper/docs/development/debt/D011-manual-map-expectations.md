# D011: `clippy::manual_map` is excused site by site, 127 times

- **Kind:** lints
- **Where:** `temper-protocol-store`, `temper-service`, `temper-client-domain`, `temper-engine-domain-forge`, `temper-client-tools`, and 13 more crates, worlds and tools
- **Seen:** 2026-10-31 at `4a6683c72`, in the skein review of that day (`skein/docs/development/debt/reviews/2026-10-31-skein.md`), theme T7 ("`manual_map`'s one reason appears in 18 wordings"), for skein; temper's count is new here
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig review of that day (`jig/docs/development/debt/reviews/2026-10-31-jig.md`), A0 F9 and its order of work, item 3 ("Set `manual_map` once per step crate"), for jig
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (about 250 attribute lines fewer; one reason, said once per crate)
- **Size:** S
- **Status:** open
- **Paid when:** no item in temper carries `#[expect(clippy::manual_map, ...)]`; a step crate that needs it says so once at its root, as jig's do, with one wording (the measure is 0).
- **Measure:** `rg -U -c --type rust '#\[expect\(\s*clippy::manual_map' temper | awk -F: '{s+=$2} END {print s}'`: 126

## What

Step code takes no closures (`skein/docs/foundation/programming-model.md`,
10.3), so it writes `match` where clippy's `manual_map` asks for
`Option::map`. temper answers the lint item by item: 126 item-level
`#[expect(clippy::manual_map, reason = ...)]` in 68 files and one at a
crate root, 127 in all, with 52 wordings of the one reason ("step
translations use explicit matches without closures" 25 times, "the
strict subset uses explicit matches" 10, "the step subset uses explicit
matches" 10, and so on). The most are in `temper-protocol-store` and
`temper-service` (12 files each) and `temper-client-domain` (9).

jig paid the same debt by one crate-level `#![expect(clippy::manual_map,
reason = "step code matches optional fields without closures")]` per
step crate (`jig-domain-authority/src/lib.rs`, `jig-client/src/lib.rs`,
and others; 22 left in jig against 276 at the review). smith has none.

## Why it matters

Each new step copies an attribute and invents a wording; the
exceptions that do say something particular are lost among them.

## What pays it back

One task: a crate-root `#![expect]` with jig's wording in each temper
step crate that has a site, then the item-level ones deleted. If the
user prefers the closure-ban lints set once for the workspace, as the
skein review proposed (theme T7), that is a root `Cargo.toml` change and
pays skein's, jig's and smith's at once. The test-crate lint profiles of
`jig/docs/development/debt/D019-lint-config-copied.md` and
`skein/docs/development/debt/D036-lint-blocks-by-hand.md` are proposed as
one plan with this item, when the first of the three is paid.
