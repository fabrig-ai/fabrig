+++
id = "gallery-stock"
part = "first-run-gallery"
kind = "feature"
needs = []
files = [
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 90
+++

# The gallery's first-run worlds offer what a fresh box offers

## Goal

The request's second miss, in the gallery (`temper/docs/plan/look-round-18/inputs/request.md`, task 2): `home-two-left` and every other state of the first run's worlds draw no "Ask temper" in the rail and no "Ask" tab, because those worlds' `me` lists the chat template alone, where a fresh box's lists the stock's chat (1), goal (2) and Ask temper (5, charter 11; `temper/crates/temper-service/src/stock.rs`, `templates`). Give the first run's worlds those templates, as `temper/docs/design/client/browser.md`, 7.2, *The worlds are the mockups'*, now says. Beside it, the same paragraph's reference world: its own way "Plan, then wait for me" is template 5 by charter 1 today, so the client takes every charter 1 chat of that world for an Ask temper chat (`temper_client_domain::pages::chats::asking`), tags it, and makes the rail's "Ask temper" current on it; its own ways move to numbers the stock leaves free and it lists the stock's Ask temper at 5.

Acceptance:

1. `worlds.rs`, `frame`: every `FirstRun { .. }` world's project lists templates 1, 2 and 5, in number order: its chat as `base_frame` makes it; the goal (2) as the reference world's loop makes its `(2, "goal")` (the chat cloned, `tracked: Some(1)`, label 2, `conversation: false`, `listed: true`); and Ask temper (5): the chat cloned, number `temper_client_domain::pages::chats::ASKING_TEMPLATE`, name "ask" as the stock names it, executor `c::TemplateExecutor::Charter(11)`. They are listed before the `FirstRun { step: 7 | 8 }` branch copies the project, so both of its projects have them.
2. The reference world's ways are `(2, "goal")`, `(4, "goal for a draft")`, `(6, "Plan, then wait for me")` and `(7, "One agent, one branch")`, and it lists Ask temper at 5 as acceptance 1 makes it, in number order; the solo world keeps 1, 2 and 4 (`number > 4` skipped, as now).
3. `configure_frame`, `configuring_chip`, `settings_chat_frame`, `picker::frame` and the sample names are unchanged (`notes/common.md`, *Leave alone*): the Ask temper states keep the templates `configure_frame` gives them.
4. Rows and assertions, each made to fail once by hand (Steps 4):
   - in `each_state_holds_what_its_name_says`'s `HOLDS`, `home-two-left` and `chats-first-run-1` each gain `Part::Named { role: Role::Link, name: "Ask temper" }` (the rail's and the tab's link are both named so);
   - `reference_lede_names_the_worlds_templates_in_number_order` expects `(6, b"Plan, then wait for me")` and `(7, b"One agent, one branch")`, its lede's words unchanged, and asserts that the reference project's template 5 is by `c::TemplateExecutor::Charter(11)`.
5. Every other test of `temper-client-gallery` passes: `heavy cargo nextest run -p temper-client-gallery --lib`, its count in the report, with the base's count of tests.
6. `names.txt` is unchanged: no state is added, renamed or removed.
7. The **Not yet** entry under 7.2, *The worlds are the mockups'*, in `temper/docs/design/client/browser.status.md` (naming `look-round-18`) is deleted whole, and no other line.
8. One commit at least, `Look-request: ask-temper-place`, its last carrying the account (below).

## Inputs

- `temper/docs/plan/look-round-18/notes/common.md`.
- `temper/docs/plan/look-round-18/inputs/request.md`, task 2, and *For every task of this plan* (goldens, prints and `names.txt`; a test's oracle your change trips).
- `temper/docs/design/client/browser.md`, 7.2: *The worlds are the mockups'* (this cut's sentence) and *Names*, binding: a state shows what its name says.
- The request: `git show 180f76e480`, *States the page does not show*: "With no such template in `me`, no row and no tab".
- The code: `temper/tools/gallery/src/worlds.rs` (`frame`, about lines 67 to 167; `base_frame`, about 3782; `world_in_layout`'s `Subject::Me` arm, about 424); `temper/tools/gallery/src/tests.rs` (`HOLDS`, `each_state_holds_what_its_name_says`, `reference_lede_names_the_worlds_templates_in_number_order`); `temper/tools/gallery/src/samples.rs` (which samples use `FirstRun` and `Reference`); the client's readers of template 5, read only: `temper_client_domain::pages::chats::{ASKING_TEMPLATE, asking}`, `pages.rs`'s chats configuring on a rail page (about line 1010), `temper-client-view`'s `frame.rs` (the rail's and tab bar's Ask temper row, about line 931).
- The stock, read only: `temper/crates/temper-service/src/stock.rs`, `chat`, `goal`, `ask` and the `templates` line (about 379).

## Owns and may change

Owns the header's files: in `worlds.rs`, `frame` and any helper it calls that this task adds; in `tests.rs`, the rows and the test acceptance 4 names, and other rows by the options below; in `browser.status.md`, its one entry.

May also change `temper/tools/gallery/src/samples.rs`, only in a state's own deliveries (never a name, an order or `configure_domain`), by the second option below; name each edit in the report.

Options, each decided:

- **A helper:** default, one helper builds the stock's Ask temper from a chat template, called for the first run and the reference world; the first run's goal may share the reference loop's construction through a second helper. Leave `configure_frame` building its own (`ui-ask-drop`'s `gallery` renames it; a later change may make it call yours).
- **A state that draws more than Ask temper's row and tab:** with template 5 listed, a project's pages also read its chats for Ask temper (`pages.rs`, about 1010), and with template 2 a goal's ways may be offered. Where a state then draws what a fresh box draws at that state (a To begin note's words by the request's `HomeNote*` rule, "Start goal…" on a first-run chat), that is the state now: update its `HOLDS` row if it pinned the old drawing, keeping what else it checks, and name it. Where the change contradicts the state's name (`home-two-left` with a step done, a `chats-*` state with another count), make that state's world deliver what the name says, in its own deliveries, and name it; a row is never loosened to pass.
- **A reference state that drew the wrong tag or current row** (a charter 1 chat tagged "Ask temper", the rail's "Ask temper" current on it): its row, if it pinned them, follows the fix; name it.
- **Another test of the gallery** that the templates trip (`no_state_reads_of_zero`, `each_chat_reads_its_allotment`, `navigation_links_have_addresses_and_install_action_links_keep_their_binding`): repair its expectation by the rule of the second option, keeping what it checks for everything else, and name it.
- **More than about twenty `HOLDS` rows to change:** stop at that point, commit and push what you have, and report `Question:` with the list and why each moved; it means the templates reach further than this cut read.

## Steps

1. Read the request's message; count the samples on `FirstRun` and `Reference` worlds (`samples.rs`).
2. The rows and assertions of acceptance 4, then `heavy cargo nextest run -p temper-client-gallery --lib -E 'test(=tests::each_state_holds_what_its_name_says) | test(=tests::reference_lede_names_the_worlds_templates_in_number_order)'`: both red for the rows' reason.
3. The worlds' change (acceptances 1 to 3); `heavy cargo check -p temper-client-gallery --tests` while errors last; the two tests green; then the whole `--lib`, reading each failure's log and repairing it by the options.
4. Bites: list the first run's templates as the chat alone again, see `home-two-left`'s and `chats-first-run-1`'s rows fail for "Ask temper"; put "Plan, then wait for me" back at 5, see the lede test fail on the numbers and the charter; restore.
5. Delete the status entry (acceptance 7).
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-gallery --all-targets -- -D warnings`; acceptance 5's run; `python3 -I -B tools/citations/citations.py --base origin/main`.
7. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| `home-two-left` and the first run's other states: the rail's "Ask temper" and the phone's "Ask" tab | landed: the first run's worlds list the stock's templates 1, 2 and 5 |
| The reference world's charter 1 chats read as Ask temper's | landed: its ways at 4, 6 and 7, the stock's Ask temper at 5 |
| Each other state whose drawing moved, by name | landed, and what it now draws; or its world made to say its name |
| `names.txt` | unchanged: no state added or renamed |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4); the gallery's print is not committed, and its tests establish no rendered equality |

## Report

The implementer's report: the rows broken by hand and the failures seen; every `HOLDS` row or test changed and why, every state whose drawing moved beyond Ask temper's row and tab; the gallery's test count before and after; the account as committed.
