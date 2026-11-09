+++
id = "team-prints"
part = "proof"
kind = "feature"
needs = ["team-reads", "team-page"]
files = [
  "temper/tools/gallery/src/worlds/team.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
  "temper/crates/temper-client-shell-terminal/src/tests/fixtures.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/render.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/team.txt",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 130
+++

# Team's gallery states and its terminal print

## Goal

The gallery's three Team states of `temper/docs/design/client/browser.md`,
7.2, `team`, `team-alone` and `night-team`, each stepped from hand-made
deliveries through Team's real reads, so that the page prints at a
laptop's and a phone's width, by day and by night; and Team's terminal
print, a golden of the terminal shell's fixtures.

Acceptance:

1. **The states** (browser.md 7.2's rows), in a new gallery world
   `temper/tools/gallery/src/worlds/team.rs`, its `mod` line and dispatch
   in `worlds.rs` and `samples.rs` as `changes_world` is dispatched, at
   `Address::Team { project: 1 }`, every page state reached by the
   client's root from deliveries (7.2: "No page state is set by hand"):
   - `team`: `me` naming the reader and the project "shop" (the mockup's
     `team/team.html`); the `Read::Project` answer with the reader as
     `Owner`, Sam as `Member` and a service; the standing inbox's delivery
     with two questions in project 1 and one in another project; the
     person's approvals with one row of project 1; Sam's `Tasks` watch's
     delivery with one live line waiting on Sam, a question. It shows
     "Two of you.", "You", "Owner of shop", "3 things wait on you.",
     "Sam", "Member", "1 question waits on Sam.", and no service.
   - `team-alone`: the reader alone, owner, nothing waiting: "Just you.",
     one box, "Nothing waits on you.".
   - `night-team`: `team` with the night look on, as `night-home` is made
     (`samples.rs`, `domain`).
   Their mockup in the index: `team/team.html` for `team` and
   `night-team`. `names.txt` holds the three, sorted as it is; `N` rises
   by three.
2. **Their rows** in `tests.rs`'s `each_state_holds_what_its_name_says`,
   each its heading "Who owns what on shop" and the part it is about,
   found by role and name: `team` the two boxes' names and both lines;
   `team-alone` its lede and its one line; `night-team` the night switch
   pressed.
3. **Their cost:** the gallery's tests are over their budget already
   (browser.status.md, 7.3). Measure `heavy cargo nextest run -p
   temper-client-gallery` CPU before and after, both in the report; a
   state costing more than 30 ms is made cheaper (its deliveries fewer)
   before the push. The **Departs** entry of 7.3 stays as it is.
4. **The terminal's print:** a fixture of Team in
   `temper-client-shell-terminal`'s `src/tests/fixtures.rs` and its row
   `("team", ...)` in `render.rs`'s `each_fixture_page_prints_as_its_golden_file`,
   at `Team { project: 1 }`, its read answered as the fixtures answer
   `Read::Project` today (`fixtures.rs`, about line 849); the golden
   `src/tests/golden/team.txt` generated with `SKEIN_CODEGEN_WRITE=1`,
   then the test run without it. Read the golden once: it holds the
   kicker, the heading, the lede, "People" and each person's box as lines.
5. **Status:** delete, whole, the **Not yet** entry "7.2: the gallery
   holds none of `team`, `team-alone` and `night-team`." in
   `temper/docs/design/client/browser.status.md`. No other design line.

## Inputs

- `temper/docs/plan/ui-p4a-team/notes/common.md`;
  `temper/docs/plan/ui-p4a-team/notes/contracts.md`, T3 and T4.
- `temper/docs/design/client/browser.md`, 7.1, 7.2 (its first bullets and
  the three rows), 7.3; 2.5, *Team's look*.
- `temper/docs/design/client/README.md`, 3.5, *Team*.
- The look: `temper/docs/design/client/web/whole-story/team/team.html`.
- The code: `temper/tools/gallery/src/worlds.rs` (`started`,
  `changes_world` and its draining of requests), `driver.rs` (`answer`
  for a read, `deliver` for a watch), `samples.rs` (`domain`, the night
  states, the mockups' index), `tests.rs` (`Holds`, `Part`);
  `temper-client-shell-terminal/src/tests/fixtures.rs` (`page`, the
  `Read::Project` answer) and `render.rs`.

## Owns and may change

Owns the files in the header. Not the gallery's other worlds, not
`worlds/whole_story.rs` (ui-p2-home's).

Options, each decided:

- **The gallery's generic reader answers `Read::Project` with `None`**
  (`worlds.rs`, about line 1793): Team's world answers it itself; the
  generic reader stays.
- **The terminal fixtures' project answer has one person and a
  service** (jig's `stories::project()`): print what it gives; no new
  document for the terminal.
- **A catch-up meets another plan's states in `names.txt`, `samples.rs`
  or `tests.rs`:** regenerate, keep both, sorted.

## Steps

1. Read the Inputs. Measure the gallery's CPU (3, before).
2. The world and its three states (1), their rows (2); bite: drop Sam's
   watch's delivery, see `team`'s row fail on Sam's line, restore.
3. The terminal's fixture and golden (4).
4. Measure again (3, after); the status entry (5).
5. Runs through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-gallery`, `-p temper-client-shell-terminal` (the
   gallery's own print test writes every page; no other print is run);
   clippy on both packages; then as
   `notes/common.md` says before the push. Commit, saying that Team's
   states print in the gallery and the terminal, and push.

## Report

The implementer's report: each state's deliveries, the rows, the bite and
its failure, the CPU before and after, the golden's lines, the runs'
counts, and what is left.
