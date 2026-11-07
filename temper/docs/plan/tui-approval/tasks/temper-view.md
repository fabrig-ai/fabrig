+++
id = "temper-view"
part = "temper-approval"
kind = "feature"
needs = ["temper-domain"]
files = ["temper/crates/temper-client-view/**", "temper/crates/temper-client-shell-web/temper.css", "temper/crates/temper-client-shell-web/tests/suite/main.rs", "temper/tools/gallery/src/samples.rs", "temper/tools/gallery/src/worlds.rs", "temper/tools/gallery/src/tests.rs", "temper/tools/gallery/names.txt"]
effort = "high"
minutes = 180
+++

## Goal

The browser draws the approval page: its card in each state of `temper/docs/design/client/README.md`, 3.5, *The approval page*, with every word of its table, its acts bound to the domain's `approve::Act`, under the banner alone (README 5.1, *at the approval place*), in the sign-in card's look (`temper/docs/design/client/browser.md`, 2.5, *The approval page's card*); and the gallery prints each state (`browser.md`, 7.2, the `approve-*` rows).

Acceptance:

- `temper-client-view` has a module `approve`, called from `root.rs`'s `build` right after the `Authorize` place's branch, drawing `root.approval()` in each `approve::State` with README 3.5's headings and words exactly, and its acts: the field "Code" (its input `Act::Words`, Enter `Act::Check`), "Check", "Approve", "Refuse", "Type a code" and "Try again", each a node with its role and name.
- While the place is open, the frame is the banner alone: no rail, tab bar, accounts' or playground's banner, and no paragraph of the pages not about the work, whatever the link's state.
- A view table test holds every state's heading, words and acts, and each act's decoded `Action`; a test holds the frame's parts absent.
- `temper.css` holds the card's rules under `/* ==== tui: approval ==== */`, right after the sign-in's rules; a rule test in `temper-client-shell-web`'s suite holds the code's face.
- The gallery holds the eleven `approve-*` states of `browser.md`, 7.2, each reached through the root's public events, each with its `HOLDS` row; `names.txt` lists them, sorted.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *The approval page* (the states table and its bullets); 5.1, *at the approval place* and *signed out at the approval*; 5.12, *ago* and *The tick*.
- `temper/docs/design/client/browser.md`, 2.5, *The approval page's card*; 7.2 (the `approve-*` rows); 7.3.
- `temper/docs/plan/tui-approval/notes/common.md`.
- The precedent: `temper/crates/temper-client-view/src/sign_in.rs`, `authorize` and `front` (the card: `look::MAIN`, `look::SIGN_IN`, a `Slot` with `look::CARD`, the brand paragraph, the level-1 heading), and its `authorize_tests`; `frame.rs`'s `rail_project` and `banner`; `choice.rs`'s `banner`; `root.rs`'s `build`.
- The domain's half, from `temper-domain`: `temper_client_domain::approve::{State, Act, Page}` (`code()`, `words()`, `state()`), `Domain::approval()`, `Action::Approve`, `Place::Approve { code }`, `address::print_approve`.

## Owns and may change

- Owned: `temper/crates/temper-client-view/**`: a new `src/approve.rs` and its `mod approve;` line in `lib.rs`; `root.rs` (the branch); `frame.rs` (`rail_project` none while `root.approval()` is some; the `Bindings`' private `Act` gains `Approve(temper_client_domain::approve::Act)` with an `insert_approve` beside `insert`, decoded to `Action::Approve`: `Words` from an `Input`'s value, `Check` from the field's `Submit`, the others from a `Press`); `choice.rs` (`banner` draws nothing while `root.approval()` is some); the playground banner's two calls in `root.rs` skipped while it is some; `words.rs` (the page's `Word`s and their English phrases, browser shell); `look.rs` (the tokens below); new `src/tests/approve.rs` with its `mod` line in `src/tests.rs`.
- In `temper-client-shell-web`, which `temper-domain` owned before you: `temper.css` (the block below, nothing else) and `tests/suite/main.rs` (one rule test). No other file of that crate.
- The gallery, `temper/tools/gallery`: `src/samples.rs` (eleven `Sample`s, `N` up by eleven, `domain`'s dispatch `if sample.name.starts_with("approve-") { return crate::worlds::approve_world(sample.name); }`, and `mockup` answering `None` for them: no mockup draws the page), `src/worlds.rs` (`pub(super) fn approve_world(name: &str) -> Domain`), `src/tests.rs` (eleven `HOLDS` rows, the array's length with `samples::N`), `names.txt` (eleven names, sorted; on a catch-up, regenerate it from the samples' sorted names rather than hand-merge).
- Words, binding (README 3.5's table): headings "Sign in a terminal?", "Approved", "Terminal signed in", "Sign-in refused", "Code expired", "Code already used", "Unknown code", "Sign in first", "temper couldn't answer just now" (with temper's typographic apostrophe, as `AuthorizeBusy`'s "couldn’t"); the card's sentences as the table gives them, CODE `ApprovalCode::shown()`, AGO 5.12's *ago* (`crate::words::Times`, so the view's tick wakes the page each minute); the field's name "Code"; the buttons' names. Apostrophes are the phrasebook's usual one (`’`), as its other words.
- Tokens, appended after the last used in `look.rs` (today `TERMINAL_DETAILS`, 602), each with its name in `NAMES`: `APPROVE` (`temper-approve`, on the main beside `SIGN_IN`), `APPROVE_CODE` (`temper-approve-code`, the code's paragraph), `APPROVE_ASKED` (`temper-approve-asked`, the time and the warning), `APPROVE_ACTS` (`temper-approve-acts`, the buttons' or the field's line), `APPROVE_PRIMARY` (`temper-approve-primary`, Approve). If another plan took those numbers on a catch-up, take the next free ones.
- Options:
  - The code's paragraph is one text run of the shown form; if the fake person needs a name to find it, give it the shown form as its name.
  - Deciding: "Approving…" or "Refusing…" as a `Status` in the buttons' place (default), or the buttons with skein-ui's disabled state if it has one; say which.
  - If `frame::banner` draws parts that mean nothing at the place (the switcher, the navigation), keep them as at the `Authorize` place (default); hiding them is a question.

## Steps

1. **Tests first, red** (the module drawing nothing until step 2):
   - `src/tests/approve.rs`, `each_state_draws_its_card`: rows `(how the state is reached, heading, words, acts)`, one per `approve::State` (the field, the field with its alert, reading, the ask, deciding, approved, signed in, refused, expired, used, unknown, signed out, busy); each built by the root's public events from `fixtures::root_in_project(None)` or the nearest fixture with a live link (`Event::Went { place: Place::Approve { code } }`, then `Event::Client { event: jig_client::Event::ApprovalAnswer { answer } }` or an `Event::Act`), the view built with `fixtures::browser_words()`; each row finds the level-1 heading, each sentence by `fixtures::main_text`, and each act by role and name. The ask's row holds "Asked 2 min ago" at a build two minutes after the answer's wall.
   - `each_act_decodes_to_its_action`: for the field's input, its submit, and each button, `crate::act` gives the `Action::Approve(..)` README 3.5 names; "Code" with `wdjb mjht` gives `Words` of those bytes.
   - `the_approval_place_draws_the_banner_alone`: at the place, with a project open and no usable account (so the rail and the accounts' banner would show on a page), the tree holds the banner and no rail, tab bar, accounts' or playground's banner; and signed out, the card's "Sign in first", not the signed-out paragraph.
   - `temper-client-shell-web`, `tests/suite/main.rs`, `the_approval_code_reads_at_a_glance`: `temper.css`'s `.temper-approve-code {` rule holds the mono face and 28 px, and the block sits after the sign-in's rules under its marker.
   - The gallery: the eleven `HOLDS` rows and `names.txt`, failing until the samples exist.
   Commit them red; the report says what each failed on.
2. `approve.rs`: `pub(crate) fn main(b: &mut Builder, root: &Domain, page: &approve::Page, words: &Words, bindings: &mut crate::frame::Bindings, times: &mut crate::words::Times)`, the card as `sign_in::authorize` opens it, then each state's heading, words and acts. The branch in `root.rs`; the frame's conditions; the bindings; the words; the tokens.
3. `temper.css`: after the sign-in's last rule (`@media (max-width: 600px) { .temper-main.temper-sign-in ...}`), the marker `/* ==== tui: approval ==== */` and the rules `browser.md`, 2.5, *The approval page's card* says: `.temper-approve-code` (`font: 28px var(--temper-mono)`, a letter-spacing of about 0.12em, the ink, a margin above and below), `.temper-approve-asked` (the muted ink, 13 px), `.temper-approve-acts` (a flex row, its gap 10 px, wrapping), `.temper-approve-primary` (the accent's background and white words, as the stylesheet's other primary buttons), the field (the mono face, its letter-spacing), the alert (the forms' red, as `.temper-main [role=alert].temper-flow-failed`), and the narrow rule inside the sign-in's `max-width: 600px` sizes. Use the stylesheet's own variables; no new colour.
4. The gallery: `approve_world(name)` builds `world(World::Bare, Address::Inbox { filter: Filter::ALL }, Then::Shown)`, steps `Event::Went { place: Place::Approve { code } }` (none for `approve-typing` and `approve-unread`, `WDJB-MJHT` otherwise), then, by name: `approve-typing` `Act::Words` of `wdjb mjht`; `approve-unread` `Words` of `WDJB-MJH` and `Check`; `approve-asking` an `ApprovalAnswer` `Read { code, state: Waiting { asked } }`, `asked` two minutes before `sample_now(World::Bare, name)`; `approve-approved`, `-signed-in`, `-refused`, `-expired`, `-unknown`, `-busy` a `Read` of that state; `approve-used` the ask, `Act::Approve`, then `Decided { decided: Used }`; `approve-signed-out` `Event::SignedOut`. Each `HOLDS` row: its heading and the part the state is about (the code in `approve-asking` and its Approve and Refuse buttons; the alert in `approve-unread`; "Signed in. You can close this tab." in `approve-signed-in`; "Type a code" in `approve-unknown`; "Try again" in `approve-busy`). `names.txt` takes the eleven names in sorted order, as `samples()` sorts them (`names_are_the_promised_ones`); its print test (7.3) runs the program into a scratch directory, so no run of yours is needed.
5. Break each rule once by hand and see its test fail, then restore: draw the rail at the place; bind Approve to Refuse's act; drop the ago's tick.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-gallery -p temper-client-shell-web --all-targets -- -D warnings`; focused tests, one at a time: `heavy cargo nextest run -p temper-client-view`, `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-gallery` (within its 7.3 cost: name its time), `heavy cargo nextest run -p temper-client-shell-web`.
7. Commit and push.

## Report

As the implementer's text says, and: the choices among the options; the tokens' numbers; each red test and what it failed on; each rule broken by hand and the failure seen; the gallery suite's time before and after.

## Answers

- 2026-11-07. *Should temper-domain repair its `CODE_TYPED_BYTES` dependency, or may temper-view edit the file?* Neither: the wire crate departed from the contract (`notes/contracts.md`, 6; `tasks/wire.md`), so the answer call renamed `jig_client_protocol_wire::CODE_INPUT_BYTES` to `CODE_TYPED_BYTES` in `jig/crates/jig-client-protocol-wire/src/{approval.rs,lib.rs}`; `temper-client-domain` is unchanged. Merge the trunk, check it compiles, and carry on from step 1 within your own files.
