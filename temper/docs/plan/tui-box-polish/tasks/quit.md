+++
id = "quit"
part = "quit"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-tui/src/event.rs", "temper/crates/temper-client-tui/src/lib.rs", "temper/crates/temper-client-tui/src/view/foot/lists.rs", "temper/crates/temper-client-tui/src/tests/commands.rs", "temper/crates/temper-client-tui/src/tests/foot.rs", "temper/crates/temper-client-tui/src/tests/golden/foot-commands-80x24-dark.txt", "temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/tests.rs", "temper/tests/client/tests/suite/tui.rs", "temper/docs/help/tui.md", "temper/docs/design/client/tui/journeys/**", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 50
+++

## Goal

In the TUI the command that closes temper is `/quit`: listed, completed
and in `/help`'s note under that name, with today's words and its
question while work runs in front. `/exit`, sent whole, still closes,
unlisted. The browser's terminal keeps `/exit`: the root's
`Command::Exit` and `temper-client-view`'s `shared` words are unchanged,
and the TUI names the command itself (the user's request of 2026-11-09:
they typed `/quit` and got "Unknown command").

Acceptance:

- `temper_client_tui::{QUIT, EXIT, command_name}` as
  `temper/docs/plan/tui-box-polish/notes/contracts.md` has them.
- The commands' list and `/help`'s note name `Command::Exit` "/quit",
  "Close temper" (in front "Close temper and stop the box"), in the
  places `/exit` held, front's `/log` before it; "/q" and "/qu" open the
  list with `/quit`; Tab completes it; Enter on "/quit" or "/q" closes,
  or in front asks while work runs, as `/exit` did.
- "/exit" sent whole is the same command; it is never listed, Tab never
  completes to it, and "/ex" or "/exi" begin no command (the unknown
  command's answer, whatever that is on the trunk: a note today, the
  notice once `notice` lands).
- The help page `temper/docs/help/tui.md` says `/quit` where it says
  `/exit` (both places), adding "(`/exit` works too)" once.
- The README.status.md entry for 6.14, *The commands* and *`/quit`*,
  6.16 and 6.17 is gone.

## Inputs

- `temper/docs/design/client/README.md`, 6.13 (the events table's
  `Send` and `Command`), 6.14 (*The commands*: *Offered*, *Their words*,
  *`/quit`*, *The list*, *Pressed*), 6.16 (*Closing*) and 6.17
  (*Quitting*, *Their words in front*).
- `temper/docs/plan/tui-box-polish/notes/contracts.md`, *temper-client-tui*.
- `temper/docs/plan/tui-box-polish/notes/common.md`.

## Owns and may change

- Owns: `temper-client-tui`'s `src/event.rs` (`QUIT` and `EXIT` after
  `LOG_COMMAND`; `command_name` after `Own`'s impl), `src/lib.rs` (the
  `pub use event::{...}` line alone), `src/view/foot/lists.rs`
  (`command_items` names each root command by `command_name`; the
  comment above it says `/quit`); `temper-client-shell-tui`'s
  `src/host.rs` (`send`'s matching of the root's commands by
  `tui::command_name`, and `EXIT` whole as `Command::Exit`); and the
  tests, golden, help page and journeys in `files`.
- Test edits: in `tests/commands.rs`, every "/exit" in the tables
  becomes "/quit"; in `tests/foot.rs`, `note_states`' "/exit    Close
  temper" and "/exit    Close temper and stop the box" lines become
  "/quit    ..." (no other line of it: `notice` edits that function's
  unknown line); in the host's `src/tests.rs`, the rows inside
  `no_project_composer_accepts_commands` and
  `in_front_quitting_asks_while_work_runs` that type or expect `/exit`
  take `/quit`, and each keeps one row sending "/exit" whole that
  still closes (or asks); in `temper/tests/client/tests/suite/tui.rs`,
  the commands' list expects "/quit" and the closing step types "/quit",
  and a second closing by "/exit" is not needed.
- New host rows go inside `no_project_composer_accepts_commands`: "/q"
  plus Enter closes, "/ex" plus Enter does not close and draws no list.
- If another reader names `Command::Exit`'s name for display in the
  TUI (grep `name()` in both TUI crates), it takes `command_name` too.
- Not yours: `temper-client-view`, `temper-client-domain`,
  `src/view/foot/notes.rs`, `src/state.rs`, `src/words.rs`.

## Steps

1. `event.rs`: the two constants and `command_name`; export them.
2. `lists.rs` and `host.rs` as above; check `/help`'s note reads
   `command_items` so it follows.
3. Tests as above; `foot-commands-80x24-dark.txt`'s "/exit     Close
   temper" line becomes "/quit     Close temper" (its paint row is
   unchanged).
4. Runs: `heavy cargo nextest run -p temper-client-tui --lib -E
   'test(/^tests::commands::/)'`, `heavy cargo nextest run -p
   temper-client-shell-tui --lib -E
   'test(=host::tests::no_project_composer_accepts_commands)'` (check the
   path), then `heavy cargo nextest run -p temper-client-tui -p
   temper-client-shell-tui -p temper-client-world -p temper-docs`.
5. Journeys: `heavy cargo nextest run -p temper-client-journeys --lib -E
   'test(=tests::the_committed_journeys_are_their_print)'`;
   `launchpad-chat`'s frame 02 draws the commands' list, so reprint
   (`notes/common.md`, *Journeys*) and check that only "/exit" lines
   changed.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui -p temper-client-shell-tui -p
   temper-client-world --all-targets -- -D warnings`; remove the status
   entry; commit, push.

## Report

As the implementer's text asks; name the journeys reprinted and the
bites (the alias removed by hand, the list's name reverted).
