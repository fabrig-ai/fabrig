+++
id = "list-near"
part = "sheet"
kind = "interface"
needs = []
files = ["skein/crates/skein-ui/src/binding.rs", "skein/crates/skein-ui/src/builder.rs", "skein/crates/skein-ui/src/tests/**", "skein/crates/skein-ui-web/src/page.rs", "skein/crates/skein-ui-web/src/page/web.rs", "skein/crates/skein-ui-web/src/tests/page.rs", "skein/crates/skein-ui-web/src/tests/page/**", "skein/crates/skein-ui-text/src/command.rs", "skein/crates/skein-ui-text/src/tests/**"]
effort = "high"
minutes = 60
+++

## Goal

A list its owner reads a page at a time can say its foot has come near: `Near { edge: Foot }` on a bound `Role::List` that continues. The sheet's picker reads on with it. A generic mechanism, skein's, with no word of a picker.

Acceptance:
- `binding::takes` accepts `Near { edge: Foot }` on `Role::List`, and refuses `Head` there; a `Role::List` bound takes nothing else;
- `skein-ui-web`'s `scrolled` reports a bound list's foot by the log's rules for `Near` (`ui-web.md`, 7): when it comes near, again as height moves while near, each quarter of a screen; never while `State::Busy`; only with `State::Continues`; it measures only a bound list;
- a list has no following, no head and no scroll of the page's own; a log, a scene and an unbound list behave exactly as before, their tests unchanged;
- the text shell offers it as its log-edge command does, with a list's foot alone;
- focused tests of `skein-ui`, `skein-ui-web`, `skein-ui-text` pass, and `temper-client-view` still builds.

## Inputs

- `skein/docs/design/ui.md`, section 6; `skein/docs/design/ui-web.md`, 7, *A bound list* and *Near*.
- `temper/docs/plan/look-round-13/notes/common.md`, *Runs*, *Deciding*; `notes/contracts.md`, *A bound list's `Near`*. This task names no look request's page; its commits still carry the trailer.

## Owns and may change

- Owns the `files` above.
- May change: `skein/crates/skein-ui-web/src/mapping.rs`, an attribute this needs; each `match` on `Role` the compiler asks for.
- Option: `Scrolled`'s listener already reports for any node with a listener in `page/web.rs`; if a list needs it registered, register only bound lists. If that would mean more than a small change there, a small helper beside the log's in `page.rs`, and say so.
- `skein/crates/skein-ui-web` was just changed by `ui-proposal-card/focus-reveal` on the trunk: build on it.

## Steps

1. Binding and builder, with a test on `takes`.
2. `scrolled` with a shared `near_foot` path, not a copy. Tests in `skein-ui-web/src/tests/page.rs`, the log's sketches for a list: near at a measure, again for a quarter, none busy, none without continues, none unbound, no follow after it.
3. Text shell's command, with a test.
4. Narrow tests, `cargo fmt`, old forms, clippy, commit with the trailer, push. The direct dependents across trees: `heavy cargo check -p temper-client-view --tests`.

## Report

As the implementer's text has it.
