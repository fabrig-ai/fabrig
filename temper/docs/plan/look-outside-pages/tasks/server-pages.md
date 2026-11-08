+++
id = "server-pages"
part = "server-pages"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-shell-web/files/pages.css", "temper/crates/temper-service/src/clients/google/pages.rs", "temper/crates/temper-service/src/clients/google/tests.rs", "temper/crates/temper-service/src/clients/google.rs", "temper/crates/temper-service/src/clients/bundle.rs", "temper/crates/temper-service/src/setup/routes.rs", "temper/crates/temper-service/src/tests/setup_lifecycle.rs", "temper/crates/temper-service/src/tests/sign_in.rs"]
effort = "high"
minutes = 75
+++

# look-outside-pages: the pages temper's server writes, in the welcome's look

## Goal

A Google sign-in's endings and the GitHub App's setup pages are bare HTML today. Draw them
as the request's `outside-pages/09.html`: temper's mark in a header, then the column, its
first act filled and "Back to …" quiet. The endings link a new stylesheet of the bundle's,
`files/pages.css`; the setup pages, whose policy fetches nothing, carry its rules inline
with their hashes moved. `no-bundle.html` stays plain.

Acceptance:

- `temper/crates/temper-client-shell-web/files/pages.css` exists, the request's
  `pages.css` with its font URLs relative to itself (contracts, 2).
- Every ending of `clients/google/pages.rs` is the contracts' bytes (contracts, 2): with
  a bundle served, its `<link>` names `/bundle/DIGEST/files/pages.css`; with none, no
  `<link>`. Escaping holds as today.
- The setup's plain pages and manifest page are the contracts' markup; their inline style
  holds `pages.css`'s rules but its `@font-face` ones; `PAGE_STYLE_HASH` and
  `MANIFEST_STYLE_HASH` match (the unit tests that pin them pass).
- A focused unit test pins that each rule of `pages.css` outside its `@font-face` rules
  stands in the setup pages' inline style, so the two cannot drift.
- `heavy cargo nextest run -p temper-service` passes; `temper-client-shell-web`'s
  focused tests pass.

## Inputs

- `temper/docs/design/protocol/sign-in.md`, 4.3 (its new bullets: the look and the
  stylesheet).
- `temper/docs/design/client/forge.md`, 9.1: *The pages' look* and *Where GitHub's way
  can't go on*.
- `temper/docs/design/client/browser.md`, 2.5, *Outside a project* (what the column
  looks like).
- `temper/docs/plan/look-outside-pages/notes/contracts.md`, 2;
  `temper/docs/plan/look-outside-pages/notes/common.md`.
- The request's `outside-pages/09.html` and `outside-pages/pages.css` (common.md says how
  to read them).

## Owns and may change

- Owns: `temper/crates/temper-client-shell-web/files/pages.css` (new);
  `temper/crates/temper-service/src/clients/google/pages.rs`, `google/tests.rs`;
  `temper/crates/temper-service/src/setup/routes.rs` and its unit tests.
- May change, call sites only: `temper/crates/temper-service/src/clients/google.rs`
  (each `pages::…(…)` call takes the stylesheet's address); `clients/bundle.rs` (one
  helper on `Bundle` giving `/bundle/DIGEST/files/pages.css`, if you put it there).
- May change, assertions only: `temper/crates/temper-service/src/tests/setup_lifecycle.rs`
  (its markup parts: `Back to temper</a>` stays true), `src/tests/sign_in.rs` (an
  ending's heading bytes). Keep what each checks for everything else.
- Read, never change: `clients/no-bundle.html` and `src/tests/bundle.rs` (the plain
  no-bundle page stays as it is); `temper/tests/client-browser/tests/browser_all/browser_github.rs`
  (it finds the link "Back to temper", still true; compile it, don't run it).
- Options, pre-approved:
  - The endings' functions take the stylesheet's address as a first parameter,
    `sheet: Option<&[u8]>`, made where `Clients` holds the bundle (`self.bundle`). If
    another shape reads plainer (a field the pages read), take it and say so.
  - If the font rules' relative URLs don't resolve under `/bundle/DIGEST/files/` (check
    `jig_protocol_engine_clients_http::bundle::file`), make them resolve and say how.
  - If `pages.css` must also be bundled elsewhere for the gallery's or the bundle's tests
    to keep passing, do the least and say so.

## Steps

1. `files/pages.css` from the request's, its four `url(...)` rewritten to
   `url(fonts/ibm-plex-sans-latin-NNN-normal.woff2)`.
2. `google/pages.rs`: `page(sheet, heading, paragraph, then, choose, back)` writes the
   contracts' bytes; each ending passes `sheet` on. Escape the stylesheet's address as
   any other attribute. `google.rs`: one place makes `Option<Box<[u8]>>` from
   `self.bundle` (the digest from `Bundle::digest`), each call site passes it.
3. `setup/routes.rs`: `STYLE` becomes `pages.css`'s rules but the `@font-face` ones (one
   line is fine), plus what a `button.primary` needs to draw as `a.primary` does (font
   inherit, full width, cursor); `page()` and `form()` write the contracts' markup. Move
   both style hashes; the existing pinning tests recompute them.
4. Tests, in the existing modules:
   - `google/tests.rs`: `endings_escape_addresses_errors_and_return_attributes` keeps its
     rows, each called with a stylesheet; add one test `endings_draw_the_column_and_link_the_bundles_sheet`:
     for each ending, with `Some(b"/bundle/d1/files/pages.css")` the bytes hold the
     `<link>`, `<header><a href="/">temper</a></header>`, `class="primary"` on its first
     link and, where it has one, `<p class="quiet"><a href="…">Back to sign in</a></p>`;
     with `None`, no `<link`.
   - `setup/routes.rs`'s tests: the shell checks see `<header><a href="/">temper</a></header>`,
     `<h1>` holding the heading, `class="primary"`; add `the_setup_style_holds_the_pages_sheets_rules`
     (reads `../../../temper-client-shell-web/files/pages.css` by `include_str!`, drops its
     `@font-face` rules, and finds each remaining rule's selector and declarations in
     `STYLE`).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-service -p temper-client-shell-web --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-service`; `heavy cargo nextest run -p temper-client-shell-web`;
   `heavy cargo check -p temper-client-browser-world --tests` (compiled, not run).
6. Commit with the trailer (common.md), push.

## Report

As the implementer's text says, with `## Account` rows: 09.html's header, column,
filled act, quiet link; `pages.css` linked; the overlong address's ending among them (say
which function draws it); the setup pages' inline rules and hashes; `no-bundle.html` plain.
