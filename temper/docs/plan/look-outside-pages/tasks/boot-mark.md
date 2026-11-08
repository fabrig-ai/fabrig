+++
id = "boot-mark"
part = "boot"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-shell-web/loading.html", "temper/crates/temper-client-shell-web/tests/suite/placeholder.rs"]
effort = "medium"
minutes = 20
+++

# look-outside-pages: the boot draws the welcome's mark

## Goal

The boot's placeholder, `loading.html`, still draws the old mark: the star in `#b8664e`
with a dot. Draw the welcome's: the four-point star in the signal, `#c2410c`, no dot, the
wordmark as it is (the request's `outside-pages/10.html`).

Acceptance:

- `loading.html`'s mark is the contracts' (contracts, 2, *The boot's mark*): the star's
  `<g>` filled `#c2410c`, holding the star's path alone; no `<circle>`; nothing else of the
  file changes.
- The placeholder's focused tests pass, with a row pinning the new mark.

## Inputs

- `temper/docs/design/client/browser.md`, 2.5, *The first paint, the placeholder's*.
- `temper/docs/plan/look-outside-pages/notes/contracts.md`, 2;
  `temper/docs/plan/look-outside-pages/notes/common.md`.
- The request's `outside-pages/10.html` (common.md says how to read it).

## Owns and may change

- Owns: `temper/crates/temper-client-shell-web/loading.html`,
  `temper/crates/temper-client-shell-web/tests/suite/placeholder.rs`.
- Nothing else. `loading.css` keeps its rules (the mark's height and place are unchanged).

## Steps

1. In `loading.html`, `fill="#b8664e"` becomes `fill="#c2410c"` and the `<circle …/>` goes.
2. `tests/suite/placeholder.rs`, `placeholder_exposes_the_loaders_nodes_and_words`: add the
   row `"<g fill=\"#c2410c\" transform=\"scale(11.4)\">"`; after the loop, assert the
   markup holds no `<circle` and no `#b8664e`.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-shell-web --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-client-shell-web --test suite`.
4. Commit with the trailer (common.md), push.

## Report

As the implementer's text says, with `## Account` rows: 10.html's mark in the signal, the
dot gone.
