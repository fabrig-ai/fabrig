+++
id = "sign-in-column"
part = "sign-in-column"
kind = "feature"
needs = ["outside-looks"]
files = ["temper/crates/temper-client-view/src/sign_in.rs"]
effort = "high"
minutes = 60
+++

# look-outside-pages: the sign-in page and a forge's authorization in the column

## Goal

Draw the browser's sign-in page and every state of the `Authorize` place (a forge's
sign-in through temper) in the column, not the card (the request's
`outside-pages/01.html` and `06.html`): the heading, its words, "Continue with Google"
filled, "Read public projects" quiet. The card's own "temper" goes: the top bar carries
the mark.

Acceptance:

- `sign_in::main` and the signed-out `Authorize` front draw `Main [OUTSIDE]` (contracts,
  3): `Heading { level: 1 }` "Sign in"; at the authorization, "to continue to FORGE" as
  today; the forges' signed-out notes as today; then the lone `Link` "Continue with Google"
  [`OUTSIDE_PRIMARY`] with today's address, or, with no way, the `SignedOutMain`
  paragraph; then, with an origin, `Paragraph [OUTSIDE_QUIET] { Link "Read public
  projects" }`. No card, no brand paragraph, no `MAIN` or `SIGN_IN`.
- Every other `Authorize` state (waiting, pending, continuing, refused, busy) draws
  `Main [OUTSIDE]`, its heading and words as today; the unknown client's and unknown
  redirect's "Back to sign in" is `Paragraph [OUTSIDE_QUIET] { Link }`.
- Addresses, `then` and words are unchanged.
- `heavy cargo nextest run -p temper-client-view` and `-p temper-client-gallery` pass;
  `heavy cargo check -p temper-client-browser-world --tests` compiles.

## Inputs

- `temper/docs/design/client/README.md`, 5.1, *The pages not about the work* (*signed
  out, with a way of signing in*, *signed out at the authorization*).
- `temper/docs/design/protocol/sign-in.md`, 7 and 9.3.
- `temper/docs/design/client/browser.md`, 2.5, *Outside a project*.
- `temper/docs/plan/look-outside-pages/notes/contracts.md`, 1 and 3;
  `temper/docs/plan/look-outside-pages/notes/common.md`.
- The request's `outside-pages/01.html` and `06.html`.

## Owns and may change

- Owns: `temper/crates/temper-client-view/src/sign_in.rs` and its inline tests.
- May change, assertions only, keeping what each checks for everything else:
  `temper/crates/temper-client-view/src/tests/frame.rs`,
  `browser_google_sign_in_has_the_mark_heading_and_return_link` (no brand paragraph in the
  main: it checks the heading, the filled link and its address, the quiet public link) and
  `signing_out_explains_each_forges_independent_sign_in` if it reads the card;
  `temper/tools/gallery/src/tests.rs`, the `google-sign-in` row of
  `each_state_holds_what_its_name_says`.
- Read, never change: `forge_sign_in.rs` (its names stay), `tests/public.rs`,
  `tests/rail.rs` (they hold as they are: run them), `root.rs`. Leave `sign_in::google`'s
  body as it is: the task `approval-column` may make it `pub(crate)`; if that conflicts on
  a catch-up, keep both.
- Options, pre-approved: if a client story (`temper/tests/client`) reads the card's
  structure, change the assertion to the column and say so.

## Steps

1. `sign_in.rs`, `front` and `authorize`: `outside::column(b, &heading, &[])`, the parts
   in contracts, 3's order, the act tokens in place.
2. Tests as above. The test must bite: put `look::CARD` back on a sign-in main and see
   `browser_google_sign_in_has_the_mark_heading_and_return_link` fail.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-view --all-targets -- -D warnings`; the focused runs of Acceptance;
   commit with the trailer (common.md), push.

## Report

As the implementer's text says. `## Account` rows: 01.html (Sign in, Continue with Google
filled, Read public projects quiet), the forge's sign-in ("Forgejo at HOST", "to continue
to"), 06.html (Continuing to Forgejo…), and the states the pages don't draw (refused,
unknown client with Back to sign in, busy, signed out after Sign out with its notes, no
way of signing in).
