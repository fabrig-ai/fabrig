# Supplement skein-01.s1: the terminal's room for the application's lines

Added by ruling 134, from the design answer of 2026-10-16T13:28Z (skein
`d047b45a`, `docs/design/ui-text.md`, 3, 4 and 7). temper 03.1 needs its
landing; jig 19.4 moves jig's skein onto skein-01.s2's, which holds
it. Read first: skein's
`AGENTS.md`, `docs/development/workflow.md`, `ui-text.md`, 3, 4 and 7;
this plan's README, "Rules of this plan".

Scope: the settings' lines and `Terminal::room` (`ui-text.md`, 3: "*its
lines*: the most of the application's own lines the terminal holds that
the output has not taken (section 4); 0 for an application that prints
none"; 4: "the settings' lines less those waiting, and none once it is
closing (section 7)").
Needs: nothing (skein's `main`).
Files: `crates/skein-ui-text/src/terminal.rs`: `Settings.lines: u32`;
the line queue `lines + BATCH + 1`, each entry marked the application's
or the terminal's own; `pub fn room(&self) -> u32`; `say` asserts room;
the read's condition (`:252`, `self.lines.room() >= BATCH`) counts the
terminal's own lines alone; `close` queues `closed` as its own.
`tests/ui-text/tests/terminal.rs`: `settings()` (`:17-18`) gains
`lines: 2`. No other `skein_ui_text::Settings` literal is in skein
(`git grep -n 'Terminal::new' -- '*.rs'` prints `:27` and `:141`, both
through `settings()`).
Test shape, in `tests/ui-text/tests/terminal.rs`:
- `the_applications_lines_keep_their_room_while_the_output_is_held`:
  `start(32, 1)`; `room()` 2; say `one`, `two`; `room()` 0; feed
  `x\ny\n` one byte per `feed`; drain and drive as
  `a_print_waits_for_the_last_to_be_written` (`:105`) does; the bytes are
  `== test == \n[1] Accept\ntest: one\ntest: two\n`, then `test:
  commands: n, n words, p, q\n` twice; `room()` 2; after `close()`, 0.
  At the base the second line's feed fails: two application lines stop
  the read (63 places free, `:252` wants 64).
- `untaken_commands_hold_the_input` (ruling 137): `start(32, 1)`; feed
  `1`, then `\n`, leaving the command untaken; then feed and drive 64
  `p` bytes, which skein-io's pipe reads ahead into its intake whatever
  the terminal demands (`crates/skein-io/src/pipe.rs:352-361`; the
  terminal's `intake: BATCH`, `terminal.rs:77`); `sim.peer_feed` of `p`
  returns 1 (the pipe's one byte), then of `\n` returns 0; once the
  command is popped and the terminal driven, the `\n` fits (returns 1).
Proof: skein's gate.
Don't: change `BATCH`, a refusal's text or `say`'s prefix; give `lines` a
default; name a downstream repository (`git -C <worktree> diff
origin/main...HEAD | grep -i -w -E '^\+.*(jig|temper|smith)'` prints
nothing). skein 01's "nothing under `crates/` changes" does not bind it.
Size: at most 80 lines.
Done when: `git -C ~/src/rust/skein log origin/main --all-match
--grep='^Plan: client-api$' --grep='^Step: skein-01.s1$' --format=%H -1`
prints its landing.
