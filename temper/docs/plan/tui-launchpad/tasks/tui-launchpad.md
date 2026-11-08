+++
id = "tui-launchpad"
part = "tui-launchpad"
kind = "feature"
needs = ["tui-foot"]
files = ["temper/crates/temper-client-tui/src/view/launchpad.rs", "temper/crates/temper-client-tui/src/view/launchpad/**", "temper/crates/temper-client-tui/src/tests/launchpad.rs", "temper/crates/temper-client-tui/src/tests/golden/launchpad-*.txt", "temper/crates/temper-client-tui/src/tests/golden/first-*.txt", "temper/crates/temper-client-tui/src/tests/goldens.rs", "temper/crates/temper-client-tui/src/tests/screens.rs", "temper/crates/temper-client-tui/src/words.rs", "temper/crates/temper-client-tui/src/limits.rs", "temper/crates/temper-client-tui/src/view.rs", "temper/tests/client/tests/suite/tui.rs", "temper/docs/design/client/tui/journeys/sign-in-approved/02.*", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 180
+++

# tui-launchpad: the launchpad in place of the first screen

## Goal

Once signed in, the TUI draws the launchpad in place of the first screen's two lines: a
framed header with "temper · PROJECT" in its border and the address at its end, the status
agent's brief in words with "written by the status agent HH:MM", or Home's headline with
none; where the person left off; what needs them; the notes; then the foot `tui-foot`
drew. Every state the request names is drawn: no project, nothing needing the person, no
brief, before `me`, out of date, a narrow terminal.

Acceptance, as `temper/docs/design/client/README.md`, 6.14, *The launchpad* says, every
bullet:

- `src/view/launchpad.rs`: `Launchpad`, `read`, `draw`. Its project: the root's `Home {
  project }`, else the first project of `me` (the root is still at its start's address);
  Home's reads only while the root is at `Home` of it.
- **The header,** region `tui-top`, framed: a level-1 heading of `Role::Text` children,
  `Title` (`tui-title`), `Separator` (`tui-faint`), the project's name (`tui-name`) and
  `HOST:PORT` (`tui-address`); then, by state: the brief new to the person
  (`home::facts(root, project, wall).brief()`): one paragraph, its `sentence`, a space, its
  `lede`, then `Byline` and its `at` in the view's clock (`words.view().time(TimeForm::Clock,
  ..)`) in `tui-byline`; with none, `shared::home_headline(..)`'s words, whose tick is the
  view's next tick; Home's needs not answered (`Needs::Unknown`, where Home's headline
  says the project's name), nothing; before `me`, `Connecting`; out of date, the view's `OutOfDateMain`; the
  credential not kept, `NotKept` in `tui-refused`, as today. With no project, `Title` alone
  in the heading, then the view's `NoProjectYet` and `NoProject` with the address.
- **The sections,** region `tui-sections`, one node in `tui-spaced` whose children are
  the sections, then the notes (`foot::draw_notes`): *Where you left off*, a `Role::Term`
  `LeftOff` (`tui-head`) and a `Role::Description` of one paragraph, the chat's title, then
  `Separator` and `shared::chat_state(line, me, w)` in `tui-hint`: the newest row of
  `pages().chats()`'s `live(client)` by `last_activity` that has not ended
  (`chats::ended`), the section left out with none or the read unanswered; *Needs you*, a
  term `NeedsYou` and a description of a paragraph a need, at most five, of
  `pages().needs().needs(inbox, forge().approvals())` in its order: `Question` or
  `OtherNeed` (`tui-signal`), a space, then `shared::need_line(need, w).words`; left out
  while Home's needs count is 0 or unknown. Nothing to press: no node of the launchpad
  but the foot's is bound.
- **The tick:** `Launchpad` carries the wall at which its words next change (the
  headline's minute); `view.rs`'s `build` keeps it as the view's tick, the earlier of it
  and signing in's, one named line there.
- **The first screen goes:** `Connected` and `Projects` (`words.rs`, their `limits.rs`
  terms), `first`'s code, the four `first-*` goldens and their rows in `goldens.rs`, and
  `screens.rs`'s `the_first_screen_says_the_links_state_and_me` (its states move to this
  task's rows), the named lines alone.
- Rows, each a failing test first, in `src/tests/launchpad.rs`: a table test
  `the_launchpad_says_each_state`, a row a state: the brief new (sentence, lede, byline at
  its clock); no brief (Home's headline, "The last change landed 2 hours ago. Nothing
  needs you." or as the fixture's facts make it, and its next tick); Home's facts
  unanswered (the title alone); before `me`; out of date; not kept; *Where you left off*
  with an ended newer chat skipped, and left out with no live chat; *Needs you* with a
  question's `?`, a held task's `•`, six needs drawn as five, and left out at 0; no project
  (the words, no section, no address in the border); the project's name while the root
  is still at its start; the bindings (none but the foot's). `read` rows where the root's
  fixtures reach the state (`src/tests/fixtures.rs`; Home's watches answered by
  `f::client` with the wire's documents), `draw` rows from data where they don't, the
  report naming which.
- **Goldens,** written by hand from the design and the request's 01, checked cell by cell,
  rows of `goldens::check` in `src/tests/launchpad.rs`: `launchpad-80x24-dark.txt` and
  `launchpad-80x24-light.txt` (a brief, one chat left off waiting on the person, one
  question needing them, the foot's quiet line with "1 needs you" and a closed line);
  `launchpad-40x12-dark.txt` (the same: the address gone from the border, the closed line
  from the quiet line); `launchpad-none-80x24-dark.txt` (no project).
- **The joint world's oracle:** `temper/tests/client/tests/suite/tui.rs`'s `CONNECTED` and
  `PROJECTS`, which name the first screen, become the launchpad's signed-in proof, the
  border's `temper · Project 1` and the quiet line's `/ for commands`, each story still
  checking a live, signed-in screen; run `TEMPER_BUNDLE=none heavy cargo nextest run -p
  temper-client-world -E 'test(/tui::/)'`.
- **The journeys' prints:** `sign-in-approved`'s `02.txt` and `02.html` reprinted by `heavy
  cargo run -p temper-client-journeys` (generation): the launchpad with no project's facts
  read. If the print stops on a stale replay (`host-launchpad` has landed), leave it under
  `## Left` for the task `again`: the committed-prints test
  (`temper-client-journeys`' `the_committed_journeys_are_their_print`) is then red until
  `again` lands, which is expected.
- `temper/docs/design/client/README.status.md`: delete the **Not yet** entry "6.14, *The
  launchpad*: the TUI draws the first screen, ..." (its bullet's lines alone), a stated
  departure from the implementer's text.

## Inputs

- `temper/docs/design/client/README.md`: 3.5, *Home* (*Home on any project*, *the
  headline*, *Needs you*, *Chats*), *With no project yet*; 6.14, *The launchpad*; 6.15.
- `skein/docs/design/ui-screen.md`, 3 and 5.2 (a title's end part), 5.3.
- The request's frames, for the look only: `temper/docs/plan/tui-launchpad/inputs/launchpad-01.txt`;
  its page `git show 9b3942a123:temper/docs/design/client/tui/journeys/launchpad/01.html`.
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, sections 0 to 3.

## Owns and may change

- Owns `src/view/launchpad.rs` (and `launchpad/` if it splits), `src/tests/launchpad.rs`,
  the goldens `launchpad-*` and the named lines above. `tui-chat` runs beside this task in
  the same crate, in `view/chat.rs`, `tests/chat.rs` and its own goldens: touch none of
  them, nor `foot.rs`, `look.rs`, `theme.rs`, `event.rs` or `state.rs`, nor `view.rs` but
  the tick's line.
- The frame differs where the design does: "Where you left off" says the chat's state as
  Home's rows do ("Waiting for you"), not 01's "waiting on your words", and no ", draft";
  the blank rows are skein's `spaced` between the sections, none between the header and
  the first section. Name each golden line that differs from 01 in the report, and why.
- `Role::Term` and `Role::Description` are skein-ui's term and its words; if skein's screen
  lays them otherwise than a line and a description two columns in, say so and use what
  lays the design's look with the same roles' meaning.
- `## Left` for `again`: `sign-in-approved/walk`'s `see "Connected to"`, which no
  screen says once this lands.

## Steps

1. The rows and the goldens by hand; `TEMPER_BUNDLE=none heavy cargo nextest run -p
   temper-client-tui --lib -E 'test(/launchpad::/)'`; see them fail.
2. `launchpad.rs`; the first screen's lines gone.
3. The joint world's oracle; the reprint; the status entry.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-tui -p temper-client-world --all-targets -- -D warnings`;
   `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-tui -p
   temper-client-shell-tui -p temper-client-journeys`; the joint world's TUI stories as
   above; `TEMPER_BUNDLE=none heavy cargo check -p temper --tests`.
5. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen; the golden lines that differ
from 01, and why; the prints reprinted or left for `again`.

## Left

- For `again`: `temper/docs/design/client/tui/journeys/sign-in-approved/walk`'s `see
  "Connected to"`.
