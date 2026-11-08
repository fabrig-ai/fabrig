+++
id = "tui-launchpad"
part = "launchpad"
kind = "feature"
needs = ["tui-surface", "view-needs", "tui-views"]
files = [
  "temper/crates/temper-client-tui/src/view/launchpad.rs",
  "temper/crates/temper-client-tui/src/view/foot.rs",
  "temper/crates/temper-client-tui/src/tests/launchpad.rs",
  "temper/crates/temper-client-tui/src/tests/foot.rs",
  "temper/crates/temper-client-tui/src/tests/limits.rs",
  "temper/crates/temper-client-tui/src/tests/golden/**",
  "temper/docs/design/client/tui/journeys/**",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 180
+++

# The launchpad as drawn: the brief, Where you left off, Needs you, Start and the quiet line

## Goal

Draw the launchpad after the whole story's mockup, on today's root
(`temper/docs/design/client/README.md`, 6.14, *The launchpad*): the
header's brief sentence with " · /brief"; *Where you left off* with each
row's project, what it is, when and its state, a chat's "/chats N", and
the person's newest goal among them; Needs you with each need's mark and
project and "n walks them"; Start; `/brief`'s note; the box's placeholder;
the quiet line's new halves. Every golden and journey print it moves is
regenerated.

Acceptance (6.14; `notes/contracts.md`, sections 3 and 4):

1. **The header** (`view/launchpad.rs`, `home`): with a brief, its
   `sentence()` alone, then the byline `Word::Byline` with, where its
   `lede()` is not empty, `Word::BriefMore` after it, one paragraph in
   `look::BYLINE`. With none, the headline as today; connecting, out of
   date, no project, not kept and the playground as today.
2. **Where you left off** (`left_off`): Home's live chats (as today) and
   the person's newest goal: of `root.pages().goals()`' rows, those whose
   requester is the person (`me`'s party) and not ended, the newest by
   `last_activity`; all merged newest first (the lower task first on a
   tie), at most three. `LeftOff` as the contract has it: `project` the
   row's project's name in `me`, `kind` `Chat { number }` (1 the newest
   chat, counting chats alone) or `Goal`, `at` the view's clock of its
   last activity (`TimeForm::Clock`), `state` `shared::chat_state`. Drawn
   a row each under `look::LEFT_OFF` (the picked row under `look::PICKED`
   as today): the project (`look::NAME`) padded with spaces to the
   longest project among this section's and Needs you's rows and three
   more; the title; `Word::KindChat` or `Word::KindGoal`, " · ", `at`,
   " · ", `state` in `look::HINT`; and on a chat's row `Word::ChatNumber`
   (`look::HINT`) at the row's end. The row's room is the window's
   columns (`tui`'s) less the region's padding and the item's mark;
   the words are cut at a word with "…" so that they, two spaces and the
   number fit; where that leaves the words fewer than 24 columns, the
   number is left out and the words cut to the room. A character is a
   column, as `chat::cut` counts.
3. **Needs you:** `Launchpad.needs` becomes `Box<[NeedRow]>` (the
   contract, section 4), the first five of `shared::need_prompts(root,
   words.view())` (read where `needs()` is read today): each row its mark
   (`Word::Question`, `MoneyNeed`, `SignOffNeed` or `OtherNeed` by
   `NeedMark`, every variant listed, in `look::SIGNAL`), a space, its
   project padded as above, then its `line.words`; the section's last row
   ends with `Word::WalksThem` (`look::HINT`), placed and left out by the
   same rule as a chat's number.
4. **Start,** after Needs you, while the launchpad has a project:
   `Word::StartHead` (`look::HEAD`), then "/change" and "/ask"
   (`Own::name`, `look::NAME`) padded to the longer and three more, each
   followed by its words in `look::HINT`: `Word::ChangeStart` with the
   project's name, `Word::AskTemper`.
5. **`/brief`'s note:** while `tui.brief_shown()` and the header holds a
   brief, after the root's notes in the sections: the sentence and the
   lede, a paragraph each, then the byline, each in `look::NOTE`.
6. **The foot** (`view/foot.rs`): the launchpad's box placeholder with a
   project is `Word::BoxStart`; the quiet line (`quiet`, `read`): at its
   left the project's name (`look::HINT`, the chat's project on a chat,
   the launchpad's on the launchpad), `Word::Separator`, the closed line,
   `Word::Separator`, the link's state as today; at its right, " · "
   between two (`look::FAINT`): the box's state and "playground" in
   front (6.17, moved from the left), "paused · a message resumes it" on
   a paused chat, the needs' count as today, and on the launchpad with a
   count `Word::ToWalk` (`look::HINT`). `Foot.hint` and the left half's
   "/ for commands" go. Narrow: "n to walk" and its separator first, then
   the closed line and its separator, then skein's cut, as 6.14 says.
7. **Tests,** a failing test first for each (`notes/common.md`):
   - `tests/launchpad.rs`' `the_launchpad_says_each_state`: rows a brief
     with a lede (" · /brief" in the byline), a brief with none (no
     " · /brief"), the header's sentence alone; left off with two chats and
     the person's goal between them in time (order, kinds, numbers 1 and
     2 on the chats alone); a goal another person asked for, and an ended
     goal (not drawn); Needs you with a question, money, a sign-off and a
     proposal (each mark; build the rows as data where the root's
     fixtures lack a kind, and say which), the last row's "n walks them";
     Start's two rows; `/brief`'s note; no project (no Start).
   - `tests/launchpad.rs`' `the_left_off_chats_are_ordered_and_bounded`:
     rows with the goal newest, oldest and fourth (left out).
   - a row table for the room rule (name it `rows_keep_their_hint_in_the_room`):
     a short row at 80 columns (the hint at the end), a long title at 80
     (cut with "…"), a row at 40 (no hint), a row at 110.
   - `tests/foot.rs`' `quiet_states`: rows launchpad with project, model
     and needs ("Hello · Claude Opus 5.5 · high · ● Live" and "1 needs
     you · n to walk"), a chat (no "n to walk"), a paused chat, in front
     (the box's state and "playground" at the right), no picker, no
     project, narrow at 60 and at 40 (the order of leaving out).
   - `the_launchpads_are_their_goldens`: rows `launchpad-80x24-dark`,
     `launchpad-80x24-light`, `launchpad-110x32-dark` (new),
     `launchpad-40x12-dark`, `launchpad-none-80x24-dark` and
     `launchpad-brief-80x24-dark` (new, `/brief`'s note), the 80-column
     ones written by hand from 6.14's illustration (Hello, a chat and a
     goal, a question, Start), the 110-column one with the four marks.
   - `tests/limits.rs`: the launchpad at its bounds (three rows, five
     needs with long words, Start, `/brief`'s note at 200 and 600 bytes)
     fits `limits()`; raise `limits.rs` by what it takes if it doesn't,
     and say so (the file then joins this task's).
8. **Goldens and journeys moved by the quiet line and placeholder:** every
   other golden of `temper-client-tui` that draws the quiet line or the
   launchpad's box (chats, `chat-views-*`, foots, front, welcome) is
   regenerated from its test's output and read line by line: only the
   quiet line, the placeholder and the launchpad's body change. The TUI's
   journey prints are reprinted (`heavy cargo run -p temper-client-journeys`,
   no argument) and read the same way; if the replay goes stale rather
   than its print, record it again (`heavy cargo run -p
   temper-client-journeys -- again launchpad-chat`, `notes/common.md`,
   *Runs*) and say so.
9. **Status:** delete the **Not yet** bullet naming `tui-launchpad` in
   `temper/docs/design/client/README.status.md`. No other design line.
10. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p
    temper-client-tui`; `cargo nextest run -p temper-client-shell-tui`;
    `cargo nextest run -p temper-client-journeys --lib -E
    'test(=tests::the_committed_journeys_are_their_print)'`.

## Inputs

- `temper/docs/plan/tui-whole-t1/notes/common.md`; `notes/contracts.md`,
  sections 3 and 4; `inputs/request.md`, *What it builds*.
- `temper/docs/design/client/README.md`, 6.14 (*The launchpad* as drawn,
  *Where it departs from its mockup*, *Its project*, *The header*, *Where
  you left off*, *Needs you*, *Start*, `/brief`'s note, *The box*, *The
  quiet line*, *A narrow terminal*), 6.15 (the tokens' rows), 6.17 (*The
  box's state*, the quiet line in front).
- The mockups `temper/docs/design/client/tui/whole-story/launchpad.html`,
  `pair/launchpad.html`, `team/launchpad.html`: words, parts and order.
- The code: `view/launchpad.rs` (`Launchpad`, `left_off`, `home`, `read`,
  `sections`, `draw`); `view/foot.rs` (`Foot`, `read`, `quiet`, `width`);
  `view/chat.rs` (`cut`); `temper-client-domain`'s `pages/goals.rs`
  (`Goals`, its rows), `pages/home.rs` (`facts`, `Facts::brief`);
  `temper_client_protocol_wire::v1::StatusBrief`; the view's
  `shared::need_prompts`, `NeedMark`, `chat_state`, `closed_line`; tests
  `tests/launchpad.rs` (`home_read`, `line`, `entries`), `tests/foot.rs`,
  `tests/goldens.rs`; `temper/docs/design/client/tui/journeys.md`, 6 and
  8.2.

## Owns and may change

Owns the files listed. `view/foot.rs`: `Foot`'s fields, `read`'s
placeholder and the quiet line alone; not `view/foot/lists.rs` or
`view/foot/notes.rs` (`tui-walk`'s), not `view.rs`, `words.rs`, `state.rs`
or `event.rs` (`tui-surface`'s, done). A word or state the drawing lacks
is a question.

Options, each decided:

- **The person's party:** from `me` as the domain names it; if `me`
  carries no party comparable to a goal's `Requester::Person`, show the
  newest live goal of any requester and say so.
- **Goals not read on the launchpad:** if `pages().goals()` is none on
  `Home { project }` while the project has goals, draw no goal and say
  so; don't add a read.
- **A need kind the fixtures can't make** (money, a sign-off): test its
  drawing from a data-built `Launchpad`, and its reading in `view-needs`'
  table, which already covers it.

## Steps

1. The failing rows and goldens, seen failing; then the header, the rows,
   Needs you, Start, the note, the foot; the narrowest test after each,
   e.g. `heavy cargo nextest run -p temper-client-tui -E
   'test(/launchpad::/)'`.
2. The new goldens by hand, then the regenerated ones read line by line;
   the journeys reprinted and read.
3. The status edit; the runs of acceptance 10; `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tui
   --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
4. Commit (why: the launchpad drawn as the whole story's mockup has it, on
   today's root); push.

## Report

As the implementer's text says, with each row and the rule broken by hand
for it, the options taken, each golden and journey page regenerated and
why it moved, and where the drawing departs from the mockup beyond 6.14's
*Where it departs from its mockup*.
