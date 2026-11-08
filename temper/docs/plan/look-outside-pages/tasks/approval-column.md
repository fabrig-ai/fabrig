+++
id = "approval-column"
part = "approval-column"
kind = "feature"
needs = ["outside-looks"]
files = ["temper/crates/temper-client-view/src/approve.rs", "temper/crates/temper-client-view/src/tests/approve.rs", "temper/tools/gallery/src/samples.rs", "temper/tools/gallery/names.txt"]
effort = "high"
minutes = 90
+++

# look-outside-pages: a terminal's approval in the column, and Google's way back when signed out

## Goal

Draw every state of the browser's approval page (`/terminal`) in the column, not the
card (the request's `outside-pages/02.html` to `05.html`). Signed out on a deployment
that offers Google, the page says "Sign in, and this page comes back with your terminal's
code." and offers "Continue with Google", whose `then` is the approval's address with its
code: today it says to open a printed link such a deployment never prints, and the person
is stuck. Without Google, today's words stay. Codes, acts and `then` are the TUI's too:
none changes.

Acceptance:

- `approve::main` draws `Main [OUTSIDE, APPROVE]` as contracts, 3 has it, with no card,
  no brand paragraph, no `MAIN` or `SIGN_IN`:
  - asking for a code, and a typed code that doesn't read: the words, then `Form` "Code"
    { `TextField` "Code" [`APPROVE_FIELD`], `Button` "Check" [`OUTSIDE_PRIMARY`], the alert
    inside the form after the button };
  - the ask: the code [`APPROVE_CODE`], the asked time and the warning
    [`APPROVE_ASKED`], `Slot [OUTSIDE_ACTS] { "Approve" [OUTSIDE_PRIMARY], "Refuse"
    [OUTSIDE_SECONDARY] }`; deciding: `Slot [OUTSIDE_ACTS] { Status "Approving…" |
    "Refusing…" }`;
  - unknown: its words then a lone `Button` "Type a code" [`OUTSIDE_PRIMARY`]; busy: a
    lone "Try again" [`OUTSIDE_PRIMARY`]; the other states their words alone, as today.
- Signed out with the view's `Ways` offering Google and an origin: "Sign in first",
  `ApproveSignedOutGoogle`'s words, then a lone `Link` "Continue with Google"
  [`OUTSIDE_PRIMARY`] to `ORIGIN/sign-in/google?then=` the approval's printed address
  (`address::print_approve(page.code())`), percent-encoded as `sign_in::google` encodes
  (`…?then=/terminal%3Fcode%3DWDJB-MJHT`). Signed out without it: today's words, no link.
- `APPROVE_ACTS` and `APPROVE_PRIMARY` are drawn nowhere (their tokens leave in a later
  task).
- A gallery state `approve-signed-out-google` (`temper/docs/design/client/browser.md`, 7.2)
  draws the Google state; `names.txt` keeps its order.
- `heavy cargo nextest run -p temper-client-view` and `-p temper-client-gallery` pass;
  `-p temper-client-shell-web` passes.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *The approval page* (its states' table, the
  Google row, *Signed out*) and 5.1, *at the approval place*.
- `temper/docs/design/client/browser.md`, 2.5, *Outside a project*; 7.2 (the new row).
- `temper/docs/plan/look-outside-pages/notes/contracts.md`, 1, 3 and 5;
  `temper/docs/plan/look-outside-pages/notes/common.md`.
- The request's `outside-pages/02.html` to `05.html`.

## Owns and may change

- Owns: `temper/crates/temper-client-view/src/approve.rs`,
  `temper/crates/temper-client-view/src/tests/approve.rs`; the gallery's new sample in
  `temper/tools/gallery/src/samples.rs` and `names.txt`.
- May change, one line each: `root.rs`'s call of `approve::main` (it passes `&view.sign_in`);
  `sign_in.rs`, `google`'s visibility to `pub(crate)` so the approval reuses it (no
  other change there); `words.rs`, the new word `ApproveSignedOutGoogle` beside
  `ApproveSignedOutBody` (contracts, 5).
- May change, rows only: `temper/tools/gallery/src/tests.rs`,
  `each_state_holds_what_its_name_says`'s `approve-*` rows and one for the new sample;
  `temper/tools/gallery/src/lib.rs`, the least that gives one sample Google's way (today
  only `Then::GoogleSignIn` gets it, `lib.rs` near line 136): if a sample flag or a new
  `Then` arm reads plainer, take it and say so.
- Read, never change: `temper/crates/temper-client-domain` (the approval's states and
  codes stay as they are); `temper/tests/client/tests/suite/terminal_approval.rs` (it
  reads domain states only; run it). The `temper.css` rules are `outside-looks`'s and
  the old ones leave in `old-look-out`: touch no stylesheet.

## Steps

1. `approve.rs`: `main` takes `ways: &crate::sign_in::Ways`; opens the column with
   `outside::column(b, &title, &[look::APPROVE])`; each state as Acceptance has it; the
   signed-out state picks its words and link by `ways.google && ways.origin.is_some()`.
2. `root.rs`: pass `&view.sign_in`.
3. `words.rs`: `ApproveSignedOutGoogle`.
4. Tests: `tests/approve.rs`, `each_state_draws_its_card`: its table gains the
   signed-out-with-Google row (words, the link's name and address); add to it, or beside it
   as a small test, the column's shape: the main's styles `[OUTSIDE, APPROVE]`, the
   primary and secondary tokens on Approve and Refuse, `APPROVE_FIELD` on the field, the
   alert inside the form. Gallery: the new sample and its row. The test must bite: drop
   the `then`'s code and see it fail.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-view -p temper-client-gallery --all-targets -- -D warnings`; the focused
   runs of Acceptance and `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(/terminal_approval/)'`; commit with the trailer (common.md), push.

## Report

As the implementer's text says. `## Account` rows: 02.html (asking, the field and Check),
03.html (the ask, Approve filled, Refuse outlined), 04.html (signed in), 05.html (signed
out with Google, its `then`), and each state the pages don't draw (unread, reading,
deciding, approved, refused, expired, used, unknown, busy, signed out without Google).
