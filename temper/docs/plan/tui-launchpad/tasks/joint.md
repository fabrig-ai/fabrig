+++
id = "joint"
part = "joint"
kind = "feature"
needs = ["host-launchpad", "tui-launchpad", "tui-chat"]
files = ["temper/tests/client/tests/suite/tui.rs", "temper/tests/client/src/tui.rs", "temper/tests/client/src/front.rs", "temper/tests/client/src/fixtures.rs", "temper/docs/design/client/README.status.md", "temper/docs/design/STATUS.md"]
effort = "high"
minutes = 150
+++

# tui-launchpad: the launchpad asks temper and draws the chat, in the joint world

## Goal

One story in the client's joint world (`temper/tests/client`) proves the TUI end to end on
temper's real engine root: signed in, the launchpad drawn for the person's project, a
question typed in its box and sent, the Ask temper chat it starts opened, its answer drawn
while it is written and once committed. Then the plan's status entries are closed.

Acceptance:

- A story in `tests/suite/tui.rs`, `a_tuis_launchpad_asks_temper_and_draws_the_chat`,
  through the shipped host (`src/tui.rs`'s `Tui`), on a world whose engine offers Ask
  temper's template 5 to the person's project and whose next chat run replies in pieces
  (`Joint::reply_in_pieces`, `release_reply`):
  1. signed in by the pasted link, as `a_tui_signed_in_by_its_pasted_link_draws_me` signs
     in; the frame's text holds the launchpad's border title `temper · Project 1`, the
     box's placeholder `Ask temper` and the quiet line's `/ for commands` and `● Live`;
  2. `/` typed: the commands' list holds `/model` (where the picker is), `/new`, `/chats`,
     `/theme`, `/help` and `/exit`, and no `/font`; Esc closes it and empties the box;
  3. a question typed and Enter: the engine assigns charter 11 with the question as the
     task's words (`front.assignments`, as `configuring_chat.rs`'s
     `the_page_starts_the_asking_chat` reads them), and the frame's text holds the chat's
     top line `✦ Project 1 / QUESTION T` (its number the task's) and `❯ QUESTION`;
  4. the reply's first piece: the frame holds `✦ ` and its words, and the running line
     `✦ working` or its words while the run is out; `release_reply`: the whole answer
     committed, and the quiet line back;
  5. `/exit` typed and Enter (or pressed): the host's duty `close`.
  Each step's frame read as the existing stories read theirs (`tui.text()`), the world
  run between steps as they run it.
- The world: `Joint::with_printed_link`'s, with Ask temper offered as
  `Joint::with_configuring` offers it: a constructor beside them in `src/front.rs`, and a
  `fixtures::` function beside `tui_world`, if no existing one serves; the TUI harness
  (`src/tui.rs`) gains what typing words takes (keys of each character) if it lacks it.
- `temper/docs/design/client/README.status.md`: delete the **Not yet** entry "10, *The TUI
  in the joint world*: ..."; delete the blank lines left between the plan's entries at the
  head of *Gaps and departures*, so that the list reads as before the plan; and in the
  summary's TUI sentence, "beyond the **Not yet** entries of the launchpad, its commands
  and a chat below" goes where no such entry is left (if one is, it stays, and the report
  names it). `temper/docs/design/STATUS.md`: the sentence "Its launchpad, the browser
  terminal's commands and a chat, from the look request `tui-launchpad`, are designed and
  not yet built (`client/README.status.md`)." says they are built, with their goldens and
  this joint story, where no entry is left. These are a stated departure from the
  implementer's text.

## Inputs

- `temper/docs/design/client/README.md`: 6.13, 6.14 (*The launchpad*, *The foot*, *The
  commands*, *A chat*), section 10 (*The TUI in the joint world*).
- `temper/docs/design/domain/testing.md`, 6.3 (the joint world).
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, sections 0, 4 and 5.

## Owns and may change

- Owns the files in `files`; the story goes beside its siblings in `tests/suite/tui.rs`,
  whose constants `tui-launchpad` set to the launchpad's words.
- The joint world is the focused suite's: run the story by name, then the TUI stories
  (`TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-world -E
  'test(/tui::/)'`). No process, browser or live tier.
- If the person's model or account in this world makes `/model` absent from the list,
  assert the list without it and say so; if Ask temper can't be offered with the
  printed-link world's sign-in, stop and say what blocks it.
- The answer's words are the story's own fixture values, never the request's frames'.

## Steps

1. The story; `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-world -E
   'test(=tui::a_tuis_launchpad_asks_temper_and_draws_the_chat)'`; see it fail first
   where the world or harness lacks a piece, then pass.
2. The status entries and the summaries.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-world --all-targets -- -D warnings`; the TUI stories as above;
   `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-tui -p
   temper-client-shell-tui`.
4. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; the story's steps and what each frame held; any entry of
the plan left in the status file.
