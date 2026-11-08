+++
id = "tui-foot"
part = "tui-foot"
kind = "feature"
needs = ["tui-surface", "view-lists", "screen-entries"]
files = ["temper/crates/temper-client-tui/src/view/foot.rs", "temper/crates/temper-client-tui/src/view/foot/**", "temper/crates/temper-client-tui/src/tests/foot.rs", "temper/crates/temper-client-tui/src/tests/golden/foot-*.txt", "temper/crates/temper-client-tui/src/tests/golden/first-*.txt", "temper/crates/temper-client-tui/src/words.rs", "temper/crates/temper-client-tui/src/look.rs", "temper/crates/temper-client-tui/src/view.rs", "temper/crates/temper-client-tui/src/limits.rs", "temper/crates/temper-client-tui/src/tests/event.rs", "temper/crates/temper-client-tui/src/tests/screens.rs", "temper/docs/design/client/tui/journeys/sign-in-approved/02.*", "temper/docs/design/client/README.status.md", "skein/crates/skein-ui-screen/src/render.rs", "skein/crates/skein-ui-screen/src/render/words.rs", "skein/crates/skein-ui-screen/src/tests/render.rs", "skein/crates/skein-ui-screen/src/tests/fixtures.rs"]
effort = "high"
minutes = 240
+++

# tui-launchpad: the foot, the commands, the lists and the notes

## Goal

The launchpad's and a chat's foot replace the first screen's bottom row: on a chat the
running line; the box between two rules, which keeps the focus and takes `q`; under it
the quiet line, "/ for commands · N needs you" and "MODEL · EFFORT · ● STATE", or in its
place a list: the commands as the box's words begin one, a command's list (`/model`,
`/effort`, `/chats`), a confirmation, or the chat's question; and the notes the
launchpad and a chat draw after their body.

Acceptance, as `temper/docs/design/client/README.md`, 6.14, *The foot*, *The commands*
and *The chat's question* (its list) say, every bullet:

- `src/view/foot.rs`: `Foot`, `read`, `draw` (returning the `Event` Esc makes), and
  `pub(crate) struct Notes`, `pub(crate) fn read_notes(root, tui, words) -> Notes` and
  `pub(crate) fn draw_notes(b, &Notes, words)`, which `tui-launchpad` and `tui-chat` call
  after their body.
- **The running line,** region `tui-working`: on `Chat { .. }`, while
  `shared::running(root, w)` is some, one list of one item (`tui-running`, its mark `✦`)
  with its words.
- **The box,** region `tui-box`: one `Role::TextArea` (`tui-field`), `State::SubmitsOnEnter`,
  bound `BOX`, named `AskTemper` off a chat and `Message` on one, its value the root's
  composer words (`pages().chats()`'s `words` off a chat, `pages().chat()`'s on one) with
  `Tui::box_written()`, its focus request `Tui::focus()`; its placeholder `AskTemper`,
  `Reply`, `Ended` on a chat that has ended (`task.result` or `chats::ended`), the
  terminal's "/ for commands" (`shared::say(Commands)`) with no project, and none while a
  list stands. Under it, in `tui-refused`, `shared::refusal(root, w)` when some.
- **The quiet line,** region `tui-bottom`, spread, two children: at its left
  `shared::say(Commands)`, or `Paused` on a chat the person paused (`chat::paused`), in
  `tui-hint`, then while the project's count is above 0 (`pages().needs().count(client,
  forge().approvals(), project)`, the project Home's or the chat's task's) " · " and
  `Needing` (its plural) or `NeedingCut` in `tui-signal`; at its right
  `shared::closed_line` and " · " in `tui-hint`, left out with no project or where the
  root's choice names no context (`pages().choice().context`), then `State` and the link's
  state as today's bottom row says it, in its token. With `Tui::columns()` above 0, the
  closed line and its " · " are left out where the two halves' widths
  (`skein_ui_screen::width` per character), two spaces and two columns of padding pass
  the columns.
- **A list** in the quiet line's place, region `tui-list`, the first that holds, as the
  browser's terminal orders them: `shared::confirmation` (head, `take` bound `Act { act:
  Confirm }`, `keep` `Act { act: Cancel }`, Esc `Act { act: Cancel }`); a command's list
  while `pages().listing()` is some (head `shared::list_head`, choices `shared::models`,
  `efforts` or `chats`, each `Act { act: choice.act }`, `selected` as `State::Selected`, Esc
  `Dismiss`); the commands while the box's words begin with "/", hold no white space and
  begin a command of `offered(root, words)` (no head, the menu named `Commands`, an item a
  command whose name begins with the words, its name and its description
  `shared::description`, `/exit`'s `Exit`, each bound `Command { command }`, Esc `Dismiss`);
  the chat's question while on a chat the box is empty and `shared::question(root, w)` is
  some and not `Tui::aside()`'s (head its question line, its choices named `Numbered`
  and the words, the `primary` first and selected then the others in order, each `Act {
  act: Needs(Choose { object, words }) }`, then `Else` numbered last, bound `Aside {
  object }`, Esc `Aside { object }`). Each an options menu (`Role::Menu`,
  `State::Options`) after the box in its form, its head a `Role::Text` in `tui-list-head`,
  its items `tui-option` with their words as the item's description.
- **The notes,** `Notes`: the root's (`pages().notes()`), `Help` as the TUI's own lines
  (each command of `offered` padded to the longest name and two more, then its words; then
  `KeyEnter`, `KeyTab`, `KeyPage`, `KeyEsc`, `KeyClose`, a line each), any other by
  `shared::note`; then the TUI's own, `Tui::unknown()`, as `shared::note(&Note::Unknown {
  words })`. Each a paragraph in `tui-note`.
- **"q to quit" goes:** `foot` binds no `QUIT`; delete `look::QUIT`, `Word::Quit` and its
  `limits.rs` term, `view.rs`'s `QUIT` decoding, `src/tests/event.rs`'s `q` row and
  `src/tests/screens.rs`'s `q to quit` assertions, the named lines alone.
- Rows, each a failing test first, in `src/tests/foot.rs`: a table test
  `the_foot_says_each_state`, a row a state of the bullets above (the count 0, 1, 3 and
  cut; paused; no project; each placeholder; offline and signed out; narrow; the running
  line on and off; each list's order, names, selected item and bindings; `/c` giving
  `/chats` on the launchpad and `/close` and `/chats` on a chat with no delegate; `/zz`
  giving no list; each list's Esc event; the question's order and its set-aside; the help
  note's lines; the TUI's unknown note), `read` rows where the root's fixtures reach the
  state (`src/tests/fixtures.rs`'s `live`, `deliver`, `event`, `client`), `draw` rows from
  data where they don't, the report naming which.
- **Goldens,** written by hand from the design and checked cell by cell, each a row of
  `goldens::check` in `src/tests/foot.rs`, the body the launchpad's stub: the four
  `first-*` goldens with the new foot (the box empty, its placeholder "Ask temper", the
  quiet line; at 40 by 12 the closed line gone); `foot-commands-80x24-dark.txt`, the request's
  02 foot (`/` typed, the six commands, `/model` active); `foot-question-80x24-dark.txt`, the
  request's 04 foot (the box empty with no placeholder, the question's head, "❯ 1. Answer
  Comma" active, "2. Answer Semicolon", "3. Tell it something else").
- **The journeys' prints:** `sign-in-approved`'s `02.txt` and `02.html` reprinted by
  `heavy cargo run -p temper-client-journeys` (generation): its first screen's foot. If the
  print stops on a stale replay (`host-launchpad` has landed), leave it under `## Left`
  for the task `again`: the committed-prints test
  (`temper-client-journeys`' `the_committed_journeys_are_their_print`) is then red until
  `again` lands, which is expected.
- `temper/docs/design/client/README.status.md`: delete the **Not yet** entry "6.14, *The
  foot* and *The commands*: no box, ..." (its bullet's lines alone), a stated departure
  from the implementer's text.

## Inputs

- `temper/docs/design/client/README.md`: 3.5, *A chat's terminal* (*The running line*, *The
  box*, *Commands*, *A command's list*, *The terminal's notes*, *A goal's question*); 6.13,
  *The TUI's events*; 6.14, *The foot*, *The commands*, *The chat's question*; 6.15.
- `skein/docs/design/ui-screen.md`, 5.3 (*an options menu*), 5.4, 6.2.
- The request's frames, for the look only: `temper/docs/plan/tui-launchpad/inputs/launchpad-01.txt`,
  `-02.txt` and `-04.txt`; their pages `git show
  9b3942a123:temper/docs/design/client/tui/journeys/launchpad/0N.html`.
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, sections 0 to 3.

## Owns and may change

- Owns `src/view/foot.rs` (and a `foot/` directory if it splits), `src/tests/foot.rs`, the
  goldens `foot-*` and `first-*`, and the named lines of `words.rs`, `look.rs`, `view.rs`,
  `limits.rs`, `tests/event.rs` and `tests/screens.rs`. `view/launchpad.rs`, `view/chat.rs`
  and their tests are the next tasks'.
- The frames differ where the design does: the active command carries the focus's mark
  and the accent over its `sel` band (02 draws a band alone); the top line and the
  header are the next tasks'. Name each golden line that differs from a frame in the
  report, and why.
- A key's act is decided by skein's key reader on the tree: Up, Down, Enter and Tab on
  the box with a list are skein's (`screen-lists`); this task binds the items. Escape's
  event is the one `draw` returns, which `view.rs` keeps for `escape`.
- If the root's choice gives no way to tell "no picker" but `context`, say so and use it.

## Steps

1. The rows and the goldens by hand; `TEMPER_BUNDLE=none heavy cargo nextest run -p
   temper-client-tui --lib -E 'test(/foot::/) | test(/goldens::/)'`; see them fail.
2. `foot.rs`: `read`, `draw`, the notes; "q to quit"'s lines gone.
3. The reprint; the status entry.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-tui --all-targets -- -D warnings`; `TEMPER_BUNDLE=none heavy cargo nextest
   run -p temper-client-tui -p temper-client-shell-tui -p temper-client-journeys`;
   `TEMPER_BUNDLE=none heavy cargo check -p temper -p temper-client-world --tests`.
5. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen; the golden lines that differ
from the frames, and why; the prints reprinted or left for `again`.

## Answers

2026-11-08. *A `label: false` submitting field lays " (enter sends)" after its value or
placeholder: a skein repair, or this task?* This task, widened by those files alone.
`skein/docs/design/ui-screen.md`, 5.4 already says it: with no label a field lays "its
value alone after its focus columns, its name still the tree's", so the renderer departs
from its design and no design or status line changes. In `skein-ui-screen`'s `control`
(`src/render.rs`), where the look says no label, lay no words after the value: no entry
word (`words::entry`), no state words (`words::states`, " (enter sends)" with them) and
no " (disabled)"; the node's states stay the tree's, as its name does. A labelled field
lays as it does, every existing skein golden byte-for-byte unchanged. First a failing row
in `src/tests/render.rs` beside `field_labels_follow_the_nearest_token`: a `TextArea` with
`State::SubmitsOnEnter` and no label, empty with a placeholder and then with a value, its
row the focus columns and the placeholder or value alone (extend `fixtures::field_look`
if it takes no states). Then resume from `0329779575`; step 4 adds `heavy cargo nextest
run -p skein-ui-screen -p skein-ui-screen-terminal` and `skein-ui-screen` to the clippy
run, and the report names the skein change.
