# README: status

2026-11-12. Where the tree's code lags or departs from [README.md](README.md).

- **Not yet:** 3: the seam's `Duties.lend` and `unlend` and, in front,
  `Report::Lent`, and the root given `Event::Lends` once in front
  (`box.md`, *Lending its terminal*) [in flight: change 1942].

Section 3 and `pages/README.md` keep the selected launchpad row while composing a
slash command. Host tests cover typed `/open` for a chat, a goal and Home,
its opener fallback, and selection clearing on ordinary words, Esc and
navigation. No known gap remains in these rules.

Section 3 and `pages/README.md`'s no-project command composer has no known
implementation gap. The host's focused key-and-frame regression covers
completion, command execution, Escape, ordinary words, quit keys,
revision stability, navigation and separation from the sign-in field.
`temper help tui` names `/quit`, `/exit` working too, and Ctrl+C as closing keys and `q` as typing.

The document, sections 2 to 3 and `pages/README.md` and `theme.md`'s parts from the user's requests of 2026-11-08 and
the look request `tui-chat-title-below` have no known implementation gap:
the binary reads the engine's link file at start whatever the address,
for `Start.boxed`; `place` words the engine's place; the band is the
answered background moved by `BAND_STEP`, with the person's words dim
after an accent "❯" where none answered, and the palette has no `sel`;
up to three left-off chats take Up, Down, Enter and Esc while the
launchpad's box is empty, before skein's key reader; `/effort` is offered
on the launchpad wherever `/model` is; `tui-bullets` draws a Markdown
bullet "•"; and a chat's title, cut to 20 characters, its number and the
place lie in the rule above the box, the top line, `tui-chat-top` and
`tui-star` gone. View, theme, place, host and golden rows cover each, in
both schemes and with and without a band. `launchpad-chat` shows the rule
with its address as the place, under the person's opening question
(`journeys.md`, section 9). The real-terminal look
of each, "box" among them, is `../README.md`, section 5's after-landing check, owned by
temper/tui.

Section 3 and `pages/README.md` and `theme.md` and `README.md`, section 4 and `box.md`'s parts from the user's requests of 2026-11-09
have no known implementation gap but the two entries below. Typed
before the sign-in lands: a key goes to the link's field only while
`sign_in_drawn` holds, else to the box, the welcome and "Connecting"
among them; the link's changes keep the box's words; the host's going to
the launchpad from the start's address carries the no-project box's
words into Home's composer; and while `landing` holds the box opens no
list and Enter keeps its words, sends nothing and gives the notice
"Still signing in". The notice: `Notice`'s `Unknown`, `SigningIn` and
`Theme` stand alone in the quiet line's right half (`tui-signal`) over
the box's state, "playground", "paused", the count and "n to walk", and
first, at the row's start, where the row is too narrow; the sign-in
screen draws none; the host keeps its end, `limits::NOTICE` from the
report that gave it, among `Host::deadline`'s times, and a due at or
past it lowers it; an unknown command and `/chats N` with no Nth chat
give it and write no note. `/quit`: `QUIT` and `command_name` name
`Command::Exit` in the list, completion and `/help`'s note, in front
too; `/exit` sent whole is the same command, and its beginnings begin
none. `/theme`: `Own::Theme` wherever the commands are offered, its
list headed "Theme. Auto follows your terminal.", Auto with what it
reads (light or dark as the terminal says, or "dark: your terminal
doesn't say"), the chosen theme active, Esc and typing lowering it;
taking one gives the root `LookKept` in its scheme, saves the one
record `theme` after the root's requests and gives its notice; the
host takes `theme` out of `Start.kept`, reads an unreadable one as
Auto, and gives the root only the answers to the root's writes.
`Start.scheme` is none where the terminal says none, and the
recording's `screen` reads and writes `none`. The box's ink:
`tui-field` names a focused value's paint of `Paint::NONE`, so the
accent lies on "❯" alone, the sign-in's link field keeping the focus's
paint, and the goldens and the journeys' frames draw the box's words
in the terminal's ink. The state rows (`link_changes_keep_the_command_composers_words`,
`the_theme_list_steps`), the foot's `notice_states`,
`the_theme_list_reads_the_terminal_and_selected_choice` and
`a_theme_notice_says_the_choice_and_terminals_word`, the commands'
prefix rows and `theme_is_own_on_every_screen`, the goldens
`foot-notice-80x24-dark`, `foot-notice-40x12-dark`,
`foot-theme-80x24-dark`, `foot-theme-answered-80x24-dark` and
`welcome-typed-80x24-dark`, and the host's
`keys_typed_during_the_file_sign_in_stay_in_the_box`,
`a_notice_stands_six_seconds_and_a_newer_replaces_it`,
`the_theme_list_takes_keeps_and_notices_a_choice`,
`a_kept_theme_starts_the_next_host_in_it` and
`a_theme_write_answer_never_reaches_the_root` hold them. The theme kept
for the next run is held at the host, started with a kept `theme`
record; no run's test reads one back through the native shell's files,
which jig's `jig-client-shell-native` writes record by record. No real
terminal has shown any of it: the check, emacs vterm among them, is
`../README.md`, section 5's after-landing work, owned by temper/tui.

Section 3 and `pages/README.md` and `theme.md` and `../testing.md` have no known implementation gap in the
launchpad as drawn and views printed (`../../draft/surfaces.md`, 11.1,
11.4 and 11.5, less T2's and T3's parts), by the plan `tui-whole-t1`:
the TUI's events `Own`, `Open`, `Walk` and `EndWalk`; the header's
brief sentence, byline and " · /brief", else Home's headline; *Where
you left off*'s rows, each its project, what it is, when and its state,
the person's newest goal among them and "/chats N" on a chat's; Needs
you's rows marked "?", "$", "✓" or "•" by the view's
`shared::need_prompts`, and "n walks them"; Start's `/change` and
`/ask`; the box's "Ask for a change, or ask temper.  / for commands"
and the quiet line's count with "n to walk"; the walk by `n` or
`/needs`, a question's choices, "Tell it something else" and "Next", a
sign-off's "Review on FORGE ↗" and another kind's "Open it on the
web", ending in "Nothing else needs you."; the TUI's own commands in
the list and in `/help`'s note, `/change WORDS` starting the project's
chat, `/ask WORDS`, `/brief`'s note, `/chats N`, and `/open` handing
the opener the page's address alone, else noting it; and a reply's
`view` blocks printed by `view::printed`, a stop a line after its
actor's mark, code's stops and a branch's bar four columns in, a way's
name and line, a proposal's "Before" and "After", a card's terms, a
`not_drawn`'s words, and asking or failed. The view's `shared` tables,
the TUI's state, launchpad, foot, commands, printed and limits rows, the
host's rows, the goldens of the launchpad, the walk and printed views at
80 by 24, dark and light, and 110 by 32, dark, and the joint world's
`a_need_the_engine_asks_is_walked_from_the_launchpad_and_answered` and
`a_change_asked_from_the_launchpad_starts_a_chat` hold them. The walk
of each screen in a real terminal against its mockup follows landing
(`../README.md`, section 5), owned by temper/tui; where the design departs from the
mockups is `../../draft/surfaces.status.md`.

- **Not yet:** 3: the host gives the root no `Event::Launchpad` after `Start` (*The launchpad's reads*) [in flight: change 785].
