+++
id = "tui-local"
part = "tui-local"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-tui/src/note.rs", "temper/crates/temper-client-tui/src/state.rs", "temper/crates/temper-client-tui/src/view.rs", "temper/crates/temper-client-tui/src/lib.rs", "temper/crates/temper-client-tui/src/tests.rs", "temper/crates/temper-client-tui/src/tests/note.rs", "temper/crates/temper-client-tui/src/tests/state.rs", "temper/crates/temper-client-tui/src/tests/screens.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 120
+++

# tui-link-colours: the TUI's state and view for the sign-in from the engine's link

## Goal

`temper-client-tui` gains the TUI's note of its sign-in (`Note`, the directory's
`sign-in` line) and the state and view of a sign-in from the engine's link: while it
runs, the first screen's "Connecting to HOST:PORT"; when it ends without signing in, the
sign-in screen as with no file, drawing no refusal for the file's link. The host that
drives them is the task `host-local`'s; this task builds the surface it steps.

Acceptance:

- `src/note.rs`, `Note` as `temper/docs/plan/tui-link-colours/notes/contracts.md`,
  section 3, spells it, exported from `lib.rs`: `Note::print(address)` writes
  `HOST:PORT until NANOS\n` or `HOST:PORT signed-out\n`; `Note::read(bytes, address)`
  gives the note for that address, one line with or without its final `\n`, and none for
  another address, an unknown word, a number with a sign, a leading zero (but `0`) or
  past `u64`, a second line, a missing field, an extra space or more than `Note::MOST`
  bytes (`temper/docs/design/client/README.md`, 6.12, `sign-in`).
- `src/state.rs`: `Local` (`Idle`, `Signing`, `Ended`), `Told::Local { local }`,
  `Tui::local()`, as contracts.md, section 3. `Told::Local { local: Signing }` sets it;
  `Told::Local { local: Ended }` sets it and leaves the sign-in screen as it is with no
  file: `signing` back to `Idle`, `pasted` false, the field empty, so that no line of a
  link's sign-in is drawn; a `Told::Link` of a live, starting, offline or out-of-date
  link sets it back to `Idle`, as it sets `signing`; `Told::Asked` (a paste sent) sets it
  to `Idle` from `Ended`. Each change moves `revision`, none otherwise. `Told::Link` of a
  signed-out link after a sign-in keeps its effects (the "Signed out." line, `openings`
  moved).
- `src/view.rs`: while `tui.local()` is `Signing` and the link is signed out, the first
  screen is drawn, its "Connecting to HOST:PORT" line, the bottom row's "q to quit"
  button and the state "Connecting" in `tui-waiting` (README.md, 6.14, *Signed in from the
  engine's link*, *When*); otherwise the screens as now. `Ended` draws the sign-in screen
  with no line for the link's sign-in, whatever `not_signed_in` the root's link holds.
- Rows, each a failing test first:
  - `src/tests/note.rs` (new, `mod note;` in `src/tests.rs`): a table of lines and
    addresses to notes or none: both kinds printed and read back, each refusal above, the
    64-byte bound at 64 and at 65.
  - `src/tests/state.rs`, `the_states_steps_are_its_table`: rows for each `Told::Local`
    from each `Local`, a live link from `Signing` and from `Ended`, a paste's `Asked` from
    `Ended`, with the revision moved or not.
  - `src/tests/screens.rs`: in `the_first_screen_says_the_links_state_and_me` or beside it,
    `Signing` on a signed-out root draws "Connecting to 127.0.0.1:8367" (the fixture's
    address) and "● Connecting", and no "Sign in this terminal"; in
    `the_sign_in_screen_says_each_line`, `Ended` after a link's sign-in answered
    `LinkUnknown`, `LinkUsed` and `NotReady` (the table's roots) draws no line.
- No golden changes: no golden draws these states. If one does, regenerate it by its test
  with `SKEIN_CODEGEN_WRITE=1` and say so.
- `temper/docs/design/client/README.status.md`: delete the **Not yet** entry "6.12,
  `sign-in`, and 6.14, *Signed in from the engine's link*, in `temper-client-tui`: ...
  are not built." and nothing else (a stated departure from the implementer's text).

## Inputs

- `temper/docs/design/client/README.md`: 6.12, *What it holds*, `sign-in`; 6.14, the
  opening lines, *The TUI's state*, *Signed in from the engine's link* (all of it) and the
  first screen's *Before `me`*.
- `temper/docs/plan/tui-link-colours/notes/common.md`;
  `temper/docs/plan/tui-link-colours/notes/contracts.md`, section 3.

## Owns and may change

- Owns the files in `files` for this wave; `theme.rs`, `words.rs` and the goldens are the
  task `tui-colours`'s next, and stay as they are.
- `Told` gains a variant: its readers outside this crate are
  `temper/crates/temper-client-shell-tui/src/host.rs` (it constructs, never matches,
  `Told`) and the crate's own tests. A match the compiler names elsewhere gets the arm,
  and the report names it.
- `Note::MOST` may be a `u32` or `usize` associated constant; contracts.md's value, 64,
  holds.
- No new word: "Connecting to HOST:PORT" and "Connecting" are the view's already
  (`Word::Connecting`, `Shared::Connecting`).

## Steps

1. The rows; `heavy cargo nextest run -p temper-client-tui --lib`; see them fail.
2. `Note`, then `Local` and its steps, then the view.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-tui --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-tui -p temper-client-shell-tui`; `heavy cargo check -p temper
   -p temper-client-journeys -p temper-client-world --tests`.
4. Commit, push.

## Report

As the implementer's text says; each row's failure seen.
