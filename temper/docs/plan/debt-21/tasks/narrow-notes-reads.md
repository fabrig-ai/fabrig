+++
id = "narrow-notes-reads"
part = "narrowing"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-notes/src/read.rs"]
effort = "high"
minutes = 60
+++

# jig-domain-notes: a pending read's kind and stage are paired by its type

## Goal

The item's named example (`jig/docs/development/debt/D017-unreachable-re-narrowing.md`). `ReadPending` holds a `Kind` and a `Stage` whose pairs are known only by reading `drive`: a name recall is never driven, and a scoped read is never at the name's stage. The type says so.

Acceptance:

- `ReadPending` holds `Named { name, .. }` or `Scoped { kind, stage, .. }`, and `drive` takes the scoped one. `Stage` loses `Ready` and `Name`, if nothing else holds them.
- The name `ReadPending`, and the signatures of `read::loaded` and `read::failed`, stay, so that `domain.rs` doesn't change.
- `rg -c 'unreachable!' jig/crates/jig-domain-notes/src/read.rs` reads 5: `read.rs:140, 176, 277, 289, 327` are gone. `:262, 263` and the `Rows` sites (`:266, 298, 312`) stay.
- `heavy cargo nextest run -p jig-domain-notes --lib` and `heavy cargo nextest run -p jig-notes-world --test suite` pass with no assertion changed.
- Net production lines: the cut's estimate is about +10.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/narrowing.md`, binding
- `jig/docs/development/debt/D017-unreachable-re-narrowing.md`
- `jig/docs/design/domain/engine.md`, 10

## Owns and may change

- **Owns:** `read.rs`.
- **`domain.rs`, `boundary.rs` and `drop_project.rs` stay out:** other plans are changing them, and the public `Rows` waits for that. If `read.rs` can't change without them, stop and ask.

## Steps

1. Read `read.rs` whole. `ReadPending`, `Stage`, `drive`; the five sites.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p jig-domain-notes --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the count; any site that stayed, and why.
