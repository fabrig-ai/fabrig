+++
id = "typing"
part = "typing"
kind = "feature"
needs = ["notice"]
files = ["temper/crates/temper-client-tui/src/view.rs", "temper/crates/temper-client-tui/src/lib.rs", "temper/crates/temper-client-tui/src/view/foot/lists.rs", "temper/crates/temper-client-tui/src/state.rs", "temper/crates/temper-client-tui/src/tests/state.rs", "temper/crates/temper-client-tui/src/tests/welcome.rs", "temper/crates/temper-client-tui/src/tests/golden/welcome-typed-80x24-dark.txt", "temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/tests.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 70
+++

## Goal

What's typed in the box shows at once, from the first frame, and stays
through the sign-in: whenever the box is drawn (the welcome in front,
the launchpad's "Connecting" while the TUI signs in from a link, the
launchpad, a chat), keys go to the box, and the link's field takes keys
only while the sign-in screen is drawn. The sign-in never clears the
box, and the words typed before `me` was read are carried into Home's
composer. Enter before the sign-in lands keeps the words and gives the
notice "Still signing in"; nothing is sent or run until the person
presses Enter again once signed in (the user's request of 2026-11-09:
an "a" typed at 3.6 s on the welcome was lost).

Acceptance:

- `temper_client_tui::{sign_in_drawn, landing}` as
  `temper/docs/plan/tui-box-polish/notes/contracts.md` has them; the
  view's choice of the sign-in screen (`build`, about `view.rs:260`)
  reads `sign_in_drawn`.
- `Host::typed` gives the link's field a key only while `sign_in_drawn`;
  otherwise the box, as the events table's `Typed` row says
  (`README.md`, 6.13).
- `Tui`'s link changes (`set_link`), its sign-in and sign-out among
  them, leave the no-project box's words and their write count alone;
  the link's field is emptied as before.
- Going to the launchpad from the start's address (`Host::launchpad`)
  carries non-empty no-project words into Home's composer,
  `Act::ChatWords`, after the `went`.
- While `landing(root)`: the box's words open no list (`lists.rs`,
  `commands`), and `Host::send` gives `Notice::SigningIn` (the
  `notice` task's `fn notice`) and does nothing else, the words kept.
  Once landed, Enter acts on them as on any words.
- A golden of the welcome with words typed before the sign-in, a host
  test, and state rows, below, each biting; the README.status.md entry
  for 6.13's `Typed` and `Send` and 6.14, *The box* and *Typed before the
  sign-in lands*, gone.

## Inputs

- `temper/docs/design/client/README.md`, 6.13 (the events table's
  `Typed` and `Send`; *Its pages*), 6.14 (*The box*: *With a project its
  words are the root's* and *Enter*; *Typed before the sign-in lands*;
  *Signed in from the engine's link*; *The notice*) and 6.17 (*Signed in
  by the handed link*, *The welcome*).
- `temper/docs/plan/tui-box-polish/notes/contracts.md`, *temper-client-tui*
  (typing; the notice).
- `temper/docs/plan/tui-box-polish/notes/common.md`.

## Owns and may change

- Owns: `temper-client-tui`'s `src/view.rs` (the two functions; `build`
  reading `sign_in_drawn`), `src/lib.rs` (the `pub use view::{...}`
  statement alone), `src/view/foot/lists.rs` (the top of `commands`
  alone: `theme-list` edits `read`'s top and `command_items`),
  `src/state.rs` (`set_link`'s two `clear_command_words` calls go; the
  focus and the field as now); `temper-client-shell-tui`'s `src/host.rs`
  (`typed`, the top of `send`, `launchpad`); the tests and golden in
  `files`; the status entry.
- Do not touch `Host::event`'s `Typed` arm (`theme-list` edits it),
  `start`, `heard`, `route`, or the local sign-in's `found`,
  `need_local` and `again_local` (other plans change them).
- If a row of `tests/state.rs` or of the host's tests asserted the
  no-project words cleared by a link change, it now asserts them kept;
  say which in the report. If the box's words with a project must also
  survive a sign-out while the root's composer holds them, that is the
  root's and already so: check, don't change the root.
- If `landing` must also hold while the link is `Starting` with a party
  but `me` unread, it already does by its second clause; if the root
  keeps `me` across a sign-out so that `landing` would read false while
  signed out, the first clause holds it.

## Steps

1. `view.rs`: `sign_in_drawn(root, tui)` (the link signed out and
   `tui.local() != Local::Signing`) and `landing(root)` (the link signed
   out, or `root.pages().me(root.client())` none); `build` uses
   `sign_in_drawn`; export both.
2. `state.rs`: `set_link` keeps the words; state rows in
   `the_states_steps_are_its_table` (or a table beside it): command
   words typed, then `Link::Live`, then `Link::SignedOut` after linked,
   the words and `box_written` unchanged; the field still emptied.
3. `lists.rs`: `commands` returns none while `landing(root)`. A row in
   `tests/welcome.rs`: "/" typed on the welcome before the sign-in shows
   no list; after it lands, it does.
4. The golden `welcome-typed-80x24-dark.txt`, a row of
   `the_welcomes_are_their_goldens`: in front, the link signed out and
   the TUI signing in from the handed link, "fa" in the box.
5. The host: `typed`, `send`'s first lines, `launchpad`'s carry. Test,
   placed right after `no_project_lifecycle` in `src/tests.rs`:
   `keys_typed_during_the_file_sign_in_stay_in_the_box`, a table over
   in front (the welcome) and bare (`Connecting to HOST:PORT`), each:
   start signing in from a link (`local`), type "a": the frame shows
   "❯ a", the field is empty; Enter: the notice says "Still signing in",
   no request but the sign-in's, the words still "a"; the sign-in
   answered and `me` read with no project: the words still "a" and drawn;
   then with a project: the launchpad's box holds "a" (the root's
   `chats().words`), and Enter asks `Act::StartChat`'s exchange. Reuse
   the helpers of `the_engines_link_signs_in_and_falls_back`.
6. Runs: the narrowest first; then `heavy cargo nextest run -p
   temper-client-tui -p temper-client-shell-tui -p temper-client-world`;
   the committed journeys' print test (`notes/common.md`, *Journeys*),
   reprinting or recording again what it names.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui -p temper-client-shell-tui --all-targets
   -- -D warnings`; the status entry; commit, push.

## Report

As the implementer's text asks; the rows changed from cleared to kept,
the journeys touched, and the bites.
