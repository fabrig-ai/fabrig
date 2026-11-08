+++
id = "chat-rule"
part = "chat-rule"
kind = "feature"
needs = ["rule-title", "look"]
files = ["temper/crates/temper-client-tui/src/view/chat.rs", "temper/crates/temper-client-tui/src/view/foot.rs", "temper/crates/temper-client-tui/src/look.rs", "temper/crates/temper-client-tui/src/theme.rs", "temper/crates/temper-client-tui/src/words.rs", "temper/crates/temper-client-tui/src/tests/chat.rs", "temper/crates/temper-client-tui/src/tests/foot.rs", "temper/crates/temper-client-tui/src/tests/theme.rs", "temper/crates/temper-client-tui/src/tests/golden/chat-*.txt", "temper/crates/temper-client-tui/src/tests/golden/foot-*.txt", "temper/docs/design/client/tui/journeys/*/*.txt", "temper/docs/design/client/tui/journeys/*/*.html", "temper/tests/client/tests/suite/tui.rs"]
effort = "high"
minutes = 75
+++

## Goal

The look request `tui-chat-title-below` in the TUI's view: a chat's top
line goes, and its title moves into the rule above the box, aligned to
the end, two dashes from the edge: the project, " / ", the title cut to
20 characters at a word with "…", the chat's number, " on " and the
engine's place (`temper_client_tui::place`, task `look`). The
conversation starts on the screen's first row.

Acceptance (`temper/docs/design/client/README.md`, 6.14, *A chat*, *The
rule above the box*; 6.15):

- On a chat, the box region (`look::BOX`) holds first a level-1 heading
  whose parts each carry `look::RULE_TITLE` beside their paint: the
  project's name (`NAME`), " / " (`FAINT`), the cut title (`NAME`), " T"
  and the number and " on " (`HINT`), the place (`ADDRESS`); the theme's
  `tui-box` border draws its title (`title: true`). At 80 by 24, the
  request's chat ("What can you tell me about Hello?", T2, Hello, at
  `127.0.0.1:40691`, `Told::Boxed` given) lays its rule as
  `────────────────────────────────── Hello / What can you tell… T2 on box:40691 ──`
  and its first row is `❯ What can you tell me about Hello?`; not boxed,
  the place is `127.0.0.1:40691`. On the launchpad the box's rule is
  bare, and its framed header is as before.
- The cut, a function of this crate (`pub(crate)`), as a table: a title
  of 20 characters or fewer whole; `What can you tell me about Hello?` →
  `What can you tell…`; a title whose words fit only with a trailing
  space dropped; one long first word → its first 19 characters and `…`;
  a title with multibyte characters counted by Unicode scalar values.
  It cuts `shared::heading`'s words, never changing `shared`.
- Narrow: the parts' width plus six over `tui.columns()` drops " on " and
  the place first, then the project and " / "; rows at 80, at a width
  that keeps all but the place, at one that keeps the title and number
  alone, and at one so narrow that the rule is bare. Columns unknown (0)
  keep every part.
- While the commands' list or a question stands under the box, the rule
  keeps the title (a row with "/" typed on a chat).
- The chat's top line, the region `tui-chat-top`, the token `tui-star`
  and their looks are gone: `look::CHAT_TOP` and `look::STAR` removed,
  `look::NAMES` renumbered with them, the theme's regions and styles to
  match; no "✦" before the project anywhere.
- The chat and foot goldens are regenerated (`chat-*`, `foot-*`): each
  changed frame checked against 6.14 before it is written.
- The TUI journeys are reprinted (`heavy cargo run -p
  temper-client-journeys`): `launchpad-chat/04.txt` equals the request's
  frame (`inputs/chat-title-below-04.diff`) but its place, which is the
  recording's `127.0.0.1:PORT` (no `box` fact in the recording), and
  `heavy cargo nextest run -p temper-client-journeys` passes.
- Commits carry the trailer `Look-request: tui-chat-title-below`.

## Inputs

- The request's message and frame:
  `temper/docs/plan/tui-launchpad-2/inputs/request-commit-chat-title-below.txt`
  and `temper/docs/plan/tui-launchpad-2/inputs/chat-title-below-04.diff`.
- `temper/docs/design/client/README.md`, 6.14 (*A chat*, *The rule above
  the box*, *A narrow terminal*) and 6.15 (the regions' and tokens'
  tables).
- `skein/docs/design/ui-screen.md`, 5.2 (*A framed region*, *A region
  framed by rules*, *A title with no start*), which task `rule-title`
  built.
- `temper/docs/design/draft/look-requests.md`, section 3 (each commit
  carries the trailer).
- `temper/docs/plan/tui-launchpad-2/notes/common.md` and
  `notes/contracts.md`, sections 1 and 4.

## Owns and may change

- Owns the files in `files`. `foot::Foot` gains what the rule needs,
  read in `foot::read` from the root and the TUI's state, so that
  `view.rs` needs no change; `chat::Chat` loses the top line's fields.
- Task `left-off` changes `temper-client-tui` at the same time, in
  `view/launchpad.rs`, `view.rs` and `tests/launchpad.rs` and the
  launchpad goldens: none of those is yours. Build your test fixtures in
  your own test files; don't edit `tests/fixtures.rs` or
  `tests/goldens.rs`. If a launchpad golden fails on the box's border
  taking `title: true` (it should not: the launchpad's box has no
  heading), report it under `## Left` rather than editing it.
- Reader edits: any use of `look::STAR` or `look::CHAT_TOP` outside your
  files (there should be none), at its call site alone; name it. And the
  joint world's TUI story, `temper/tests/client/tests/suite/tui.rs`: its
  `heading`, `✦ Project 1 / {QUESTION} T{task}`, becomes the rule's
  words, `Project 1 / Which checks should… T{task} on ` followed by the
  world's address (its `Start` is not on a box), asserted on the text as
  today; nothing else in that file. If the world's window is too narrow
  for every part, assert the parts the narrowing keeps, and say so.
- Pre-approved: the " on " and "…" are the TUI's own phrases in
  `words.rs`; the number's " T2" is the view's `Shared::Chat` as today.

## Steps

1. `theme.rs` and `look.rs`: the box border's title; `CHAT_TOP`,
   `STAR` and their looks removed, `NAMES` renumbered.
2. `foot.rs`: the rule's parts, the narrowing, the heading drawn first in
   the box region on a chat; `chat.rs`: no top line; the cut.
3. Tests as above, as rows of the existing tables in `tests/chat.rs`,
   `tests/foot.rs` and `tests/theme.rs`; break each once (the cut's word
   rule, the narrowing's order, the border's title), see its row fail,
   restore.
4. Goldens, then the journeys' reprint and their test.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui --all-targets -- -D warnings`; `heavy
   cargo nextest run -p temper-client-tui -p temper-client-shell-tui -p
   temper-client-journeys`; `heavy cargo nextest run -p
   temper-client-world -E 'test(/tui/)'` (read the count).
6. Commit (with the trailer) and push.

## Report

As the implementer's text says; say how `04.txt` compares with the
request's frame.
