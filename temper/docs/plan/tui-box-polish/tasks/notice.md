+++
id = "notice"
part = "notice"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-tui/src/state.rs", "temper/crates/temper-client-tui/src/lib.rs", "temper/crates/temper-client-tui/src/limits.rs", "temper/crates/temper-client-tui/src/words.rs", "temper/crates/temper-client-tui/src/view/foot.rs", "temper/crates/temper-client-tui/src/view/foot/notes.rs", "temper/crates/temper-client-tui/src/tests/state.rs", "temper/crates/temper-client-tui/src/tests/foot.rs", "temper/crates/temper-client-tui/src/tests/launchpad.rs", "temper/crates/temper-client-tui/src/tests/welcome.rs", "temper/crates/temper-client-tui/src/tests/limits.rs", "temper/crates/temper-client-tui/src/tests/golden/foot-notice-*.txt", "temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/tests.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 90
+++

## Goal

One-line notices go to the right end of the TUI's quiet line, not into
the conversation: the TUI's state holds one `Notice`, the quiet line
draws it in place of its right half for 6 seconds, timed by the host's
dues, and an unknown command's words become that notice. This task
makes the notice and moves the unknown command's note into it; it also
names `Notice::SigningIn` and its words, which `typing` gives, and
leaves room for `theme-list`'s `Notice::Theme`.

Acceptance:

- `temper_client_tui::Notice`, `Told::Notice`, `Tui::notice` and
  `limits::NOTICE` as `temper/docs/plan/tui-box-polish/notes/contracts.md`
  has them; `Told::Unknown`, `Tui::unknown` and the `unknown` field gone.
- The quiet line (`view/foot.rs`, `quiet`) draws a standing notice alone
  on its right (`look::SIGNAL`): the box's state, "playground", "paused ·
  a message resumes it", the count and "n to walk" give way to it; in a
  narrow window, where the left half, two spaces, the notice and the
  padding are wider than the columns, the notice comes first in the row
  and then " · " and the left half, which skein cuts from its end
  (`README.md`, 6.14, *The notice*). On the launchpad, the welcome and a
  chat alike.
- Words, the TUI's own: `Notice::Unknown { words }` says "Unknown command
  WORDS · /help lists them" (WORDS the box's words, "/" first), and
  `Notice::SigningIn` "Still signing in".
- The host: an unknown command, and `/chats N` with no Nth chat, give
  `Notice::Unknown` (the box cleared and `terminal::Act::Dismiss`, as
  now) and no note in the conversation; the notice stands 6 seconds from
  that report, a newer one replacing it and its end; `Host::deadline`
  counts its end; a `Due` at or past the end lowers it; a send, a key or
  another screen leave it standing (`README.md`, 6.13, *The notice's
  time*).
- Tests below pass and bite; the README.status.md entry for 6.13, *The
  notice's time*, 6.14, *The notice*, and 6.17, *The box's state*, is
  gone.

## Inputs

- `temper/docs/design/client/README.md`, 6.13 (*The notice's time*;
  the events table's `Send`), 6.14 (*The quiet line*, *A narrow
  terminal*, *The notice*, *Words no command of the TUI's names*,
  *`/chats N`*) and 6.17 (*The box's state*).
- `temper/docs/plan/tui-box-polish/notes/contracts.md`
  (*temper-client-tui*, the notice; *temper-client-shell-tui*, `notice`).
- `temper/docs/plan/tui-box-polish/notes/common.md`.

## Owns and may change

- Owns, in `temper-client-tui`: `src/state.rs` (`Notice` beside `Told`;
  the `notice` field in the `unknown` field's place; `Told::Notice` in
  `Told::Unknown`'s place and its arm in `step`, moving the revision
  only when the notice changes; `notice()` in `unknown()`'s place),
  `src/lib.rs` (the `pub use state::{...}` line gains `Notice`; no other
  line), `src/limits.rs` (`NOTICE`, after `WALK`), `src/words.rs` (two
  words at the end: `UnknownNotice` and `SigningIn`), `src/view/foot.rs`
  (`Foot` gains the notice's words; `read` says them; `quiet` draws
  them), `src/view/foot/notes.rs` (the TUI's unknown note leaves; the
  root's own notes stay), and the tests and goldens in `files`.
- Owns, in `temper-client-shell-tui`: `src/host.rs` (a field for the
  notice's end, declared after `need: Option<Need>,` and initialised
  after `need: None,`; `fn notice`; the `Due` arm; `deadline`;
  `unknown`; the first line of `send` and of `went`, which tell
  `Told::Unknown { words: None }` today, removed) and `src/tests.rs`.
- Reader edits allowed: any file of `temper-client-tui` or
  `temper-client-shell-tui` that names `Told::Unknown` or `unknown()`,
  call sites only; `temper/tests/client/**` likewise, if any.
- If the box's words can be longer than the phrase's hole bound, cut
  them to it at a character's boundary and say so in the report.
- The `/log` note keeps its own rule (until the next send or screen):
  `Told::Unknown` cleared `log_shown` today; keep that clearing where
  `Told::Cleared` or `Told::Went` already does it, and say in the report
  if a row needed it elsewhere.
- Not yours: `temper-client-view` (`shared::note` stays the browser's),
  `src/view/foot/lists.rs`, `src/event.rs`, `docs/help/`.

## Steps

1. State and words: `Notice`, the field, `Told::Notice`, `notice()`;
   `UnknownNotice` with one hole for the words, `SigningIn` with none;
   `limits::NOTICE`. Rows in `tests/state.rs` replace the `unknown` rows:
   a notice told, the same told again (no revision move), another
   replacing it, `None` lowering it, `Cleared` and `Went` leaving it.
2. The view: `Foot` carries `notice: Option<Box<[u8]>>`, said from
   `tui.notice()`; `quiet` puts it alone on the right, or first in the
   left `Role::Status` child where the narrow rule says (the existing
   width helpers count it). Drop the unknown line from
   `notes::read_notes`. Rows in `tests/foot.rs`, `the_foot_says_each_state`:
   a notice over each right-half part (box state, playground, paused,
   count with "n to walk") and the narrow case; `note_states` loses its
   "Unknown command /font. /help lists them." line and its
   `Told::Unknown`. `tests/launchpad.rs` and `tests/welcome.rs` rows that
   told `Told::Unknown` and read the note now tell `Told::Notice` and
   read it on the quiet line.
3. Two goldens, rows of `the_foots_are_their_goldens`:
   `foot-notice-80x24-dark.txt`, in front with "forgejo-3 restarting"
   and one need standing, the notice of `/qiut` in their place; and
   `foot-notice-40x12-dark.txt`, the same notice before the line's own
   words.
4. The host: `fn notice(&mut self, notice: tui::Notice)` tells it and
   sets the end to `self.env.now + limits::NOTICE`; `unknown()` calls it
   with the box's words; the `Due` arm lowers a notice whose end is at
   or before `now`; `deadline()` adds the end to its `earliest` chain.
   Test, placed right after `no_project_keys` in `src/tests.rs`:
   `a_notice_stands_six_seconds_and_a_newer_replaces_it` (an unknown
   command on the launchpad: the notice, the deadline at its end, a send
   and `/new` leaving it, `Due` before the end leaving it, a second
   unknown at 3 s moving the end to 9 s, `Due` at 9 s lowering it, with
   no line of it in the frame's conversation), and a row for `/chats 9`.
   Existing rows that read `unknown()` (about lines 1535, 1828, 2143 to
   2155 and 2691) read `notice()`; a row that asserted the note cleared
   by a send or a screen now asserts the notice standing.
5. Runs: the narrowest first (`heavy cargo nextest run -p
   temper-client-tui --lib -E 'test(=tests::state::the_states_steps_are_its_table)'`
   and the like), then `heavy cargo nextest run -p temper-client-tui -p
   temper-client-shell-tui -p temper-client-world`; then `heavy cargo
   nextest run -p temper-client-journeys --lib -E
   'test(=tests::the_committed_journeys_are_their_print)'`, reprinting
   if it names a stale print (`notes/common.md`, *Journeys*).
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui -p temper-client-shell-tui --all-targets
   -- -D warnings`; remove the status entry; commit, push.

## Report

As the implementer's text asks; name the goldens added and changed, the
journeys reprinted if any, and each bite.
