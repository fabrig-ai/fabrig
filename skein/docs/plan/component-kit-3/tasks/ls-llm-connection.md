+++
id = "ls-llm-connection"
part = "lib-sites"
kind = "migration"
needs = ["lib-contract"]
files = ["skein/crates/skein-llm-connection/**"]
effort = "medium"
minutes = 30
+++

## Goal

skein-llm-connection's private `earlier` and its `enum Lifecycle` move onto lib's `earliest` and `Lifecycle`. (`skein/docs/design/draft/component-kit.md`, 3.4.) No behaviour
changes; the crate's tests pass unchanged, but for tests of a removed
private helper that lib's tests cover.

The copies:

- `src/deadlines.rs`: `fn earlier(left: Option<Time>, right: Option<Time>) -> Option<Time>` and its callers → `skein_lib::earliest`.
- `src/component.rs`: `enum Lifecycle { Live, Closing, Aborting, Closed }` → `skein_lib::Lifecycle`; the close and abort arms of `Request` use `close()` and `abort()` where they write the transition by hand, keeping the `unreachable!`/`assert!` for `Closed` as they are.
- Any other inline merge of two optional deadlines in the crate (`rg -n 'is_none\(\) \|\||\.min\(' src`) → `earliest`.

Acceptance: each copy above is gone or reads through lib as stated; the
counts of `notes/counts.md` it touches fall by what it removed; clippy and
the crate's tests pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`,
  `skein/docs/plan/component-kit-3/notes/lib-sites.md` (the table of lib's
  items, how to move, and the checks), `skein/docs/plan/component-kit-3/notes/counts.md`.
- lib's items as merged: `skein/crates/skein-lib/src/{time,queue,stack,lifecycle,slab,hex,decimal}.rs`.
- `skein/docs/design/llm-connection.md` for the owner's close and abort.

## Owns and may change

- Owns `skein/crates/skein-llm-connection/**`.
- Nothing outside them: no public item of the crate changes.
- Pre-approved:
  - Only the copies named here; a copy of the same helper found elsewhere in the owned crates moves too, and the report names it.

## Steps

1. Move each copy as `notes/lib-sites.md` says; remove what is left dead.
2. `python3 -I -B scripts/old_forms.py --write` from `skein/`.
3. `cargo fmt`; `heavy cargo clippy -p skein-llm-connection --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-llm-connection`. Commit and push.

## Report

As `notes/lib-sites.md` says.
