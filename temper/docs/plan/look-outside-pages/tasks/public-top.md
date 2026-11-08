+++
id = "public-top"
part = "public-top"
kind = "feature"
needs = ["outside-looks"]
files = ["temper/crates/temper-client-view/src/public.rs", "temper/crates/temper-client-view/src/tests/public.rs"]
effort = "medium"
minutes = 40
+++

# look-outside-pages: the public pages under the top bar

## Goal

The public pages (`/public`, a board, a landing) draw the old banner. Give them the top
bar: temper's mark at the left, a link, and "Sign in" at its right where the deployment
has a way (the request's frame, contracts, 4). Their main stays as it is.

Acceptance:

- `build_public` draws `Banner [TOP]`: its brand link as today (the projects' list),
  styled `BRAND`, named and worded "temper"; then, where Google's way and an origin are
  given, `Slot [TOP_END] { Link "Sign in" }` with today's address. No `BANNER`.
- The main and its tokens are unchanged.
- `heavy cargo nextest run -p temper-client-view` passes; `heavy cargo check -p
  temper-client-browser-world --tests` compiles; the gallery's focused tests pass.

## Inputs

- `temper/docs/design/client/browser.md`, 8.2 (*Anyone, signed in or not*: the top bar's
  "Sign in") and 2.5, *Outside a project*.
- `temper/docs/plan/look-outside-pages/notes/contracts.md`, 1 and 4;
  `temper/docs/plan/look-outside-pages/notes/common.md`.

## Owns and may change

- Owns: `temper/crates/temper-client-view/src/public.rs`,
  `temper/crates/temper-client-view/src/tests/public.rs`.
- May change, assertions only: `temper/tests/client-browser/tests/browser_all/browser_public.rs`
  and `temper/tests/client-browser/src/**` if one reads the banner's class or token
  (compile, don't run); the gallery's tests if a public sample's frame is asserted.
- Read, never change: `temper.css` (the top bar's rules are there from `outside-looks`).

## Steps

1. `public.rs`: the banner's style `TOP`; `BRAND` on the brand link (as `Draw::link`
   builds it, add the style after it opens, or give `link` a style); the external "Sign
   in" inside `Slot [TOP_END]`.
2. `tests/public.rs`, `projects_are_named_and_the_banner_only_offers_a_known_sign_in`:
   assert the banner's `[TOP]`, the brand's `BRAND`, and "Sign in" inside the `TOP_END`
   slot where offered, none where not. The test must bite: drop `TOP_END` and see it fail.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-view --all-targets -- -D warnings`;
   the focused runs of Acceptance; commit with the trailer (common.md), push.

## Report

As the implementer's text says. `## Account` row: the public pages' top bar with "Sign in"
at its right.
