+++
id = "hand-lists"
part = "lists"
kind = "migration"
needs = []
files = ["hearth/crates/hearth-domain-render/src/parts/exports.rs", "hearth/crates/hearth-domain-render/src/schema.rs", "hearth/crates/hearth-domain-render/src/schema/references.rs"]
effort = "medium"
minutes = 15
+++

# Render's lists stop doubling by hand

## Goal

D001's first table lists five `.room() == 0` checks in `hearth-domain-render` that keep nothing: each doubles a `List` by hand and refuses nothing, which `List::uncapped` already does. They go before D001's table does, so that no check is left without a reason.

Acceptance:

- `git grep -n 'room()' -- hearth/crates/hearth-domain-render/src/parts/exports.rs hearth/crates/hearth-domain-render/src/schema.rs hearth/crates/hearth-domain-render/src/schema/references.rs` prints nothing.
- Every list those checks grew is `List::uncapped()`, and pushes as before. What the code gives is unchanged: every test of `hearth-domain-render`, and of `hearth-render-world`, passes as it stands.
- No ceiling moves. If one does, `notes/common.md` says what to do.

## Inputs

- `hearth/docs/plan/hearth-memory-6/notes/common.md`
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "The machines and the domains": the three rows that say "Nothing"
- `skein/crates/skein-lib/src/list.rs`: `uncapped` and `push`

## Owns and may change

- **Owns:** the three files under `files`.
- **Where the first capacity comes from:** a check doubles a list that was made with a capacity. Each such `List::with_capacity(..)` in these files, where only those checks grew it, becomes `List::uncapped()`. A list made elsewhere and passed in, whose maker is outside these files: change the maker's line too, and only that line, and name it in the report.
- **A list with a limit stays.** If a list in these files is refused past a limit, by a check that isn't one of the five, it's an edge. Leave it.

## Steps

1. Read each of the five checks: `exports.rs`' `push`; `schema.rs`' `push`, `fault` and `add_default`; `references.rs`' `condition`.
2. Replace the doubling with a push on an uncapped list. A helper that only doubled and pushed goes, its callers pushing directly, if nothing else is left in it.
3. No new test: this changes no behaviour, and the render world's tests and goldens hold it.
4. `heavy cargo clippy -p hearth-domain-render --all-targets -- -D warnings`, then the tests of `hearth-domain-render` and `hearth-render-world`, and `heavy cargo check -p hearth-domain-builder --tests`.
5. Commit and push.

## Report

What went, the lists changed, what ran and how many tests.
