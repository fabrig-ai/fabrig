+++
id = "decided-client"
part = "client"
kind = "feature"
needs = ["decide-forge", "classes-value"]
files = [
  "temper/crates/temper-client-domain/Cargo.toml",
  "temper/crates/temper-client-domain/src/pages/decided.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/pages/away.rs",
  "temper/crates/temper-client-domain/src/forge/decided.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests/decided.rs",
  "temper/crates/temper-client-view/Cargo.toml",
  "temper/crates/temper-client-view/src/pages/decided.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/pages/needs.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/shared/mod.rs",
  "temper/crates/temper-client-view/src/shared/reach.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/tests/needs.rs",
  "temper/crates/temper-client-view/src/tests/toast.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
  "temper/tests/client/tests/suite/away.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 160
+++

# Decided for you, on Home and Needs you, for any task

## Goal

The client shows what agents decided for the person, any task's, for 24 hours on Home and on *Needs you*, each row saying what allowed it, with Overturn from either page; Home shows its rows whether or not the person was away. The `decided` watches move from Home's page to one page-held store that Home, *Needs you* and, later, a need's reason line (`needs-why`) read (`temper/docs/design/client/README.md`, 3.5, **Decided for you**).

Acceptance:

1. **The store,** a new module `temper/crates/temper-client-domain/src/pages/decided.rs`, `pub mod decided` in `pages.rs`, held by `Pages` (sketch below). It watches the subject `decided` (`forge::decided::watch`) of each project the open page shows, at most `DECIDED_PROJECTS` (4): `Address::Home { project }`, that project; `Address::Inbox { filter }` with `filter.project`, that project; unfiltered, the shown need's project (`needs::Session::shown`, its card's `project` or its approval row's) and then `me`'s projects in their order; any other address, none. It reconciles after every step, beside `home::look`'s calls in `route.rs`: a watch whose project stays wanted stays open across a page change (Home to its filtered *Needs you* keeps it); one no longer wanted is unwatched. A watch refused or ended reads as no rows and is not opened again while its page stays. The shown rows' tasks are read (`Read::Task`) as Home reads them today, at most `DECIDED_SHOWN` (7), a read kept across pages while its row shows.
2. **Home leaves its own watch:** `Home`'s `decided`, `decisions` and `reads` fields, `decided_told`, `away` and `read_tasks` go; Home's facts read the store. `home::Facts.away: Option<AwayForm>` becomes `decided: Option<decided::Shown>` (Home's project's rows showing, none while To begin shows) and `spell: Option<home::Spell>`; `AwayForm`, `DecisionLine` and `DecisionTask` move to `pages::decided` (`DecisionLine` gains `project` and `reach`).
3. **The spell** (3.5, *Decided for you*, *On Home*): `Facts.spell` is `Some` while rows show and the party's Away row there is on at the wall, or its `ended(wall)` is less than 24 hours before it: `since`, the end (`off`, or `until` once past), and `landed` less the row's `landed` once `status` is delivered. With rows showing and no spell, the row undelivered or the `away` watch refused among them, Home's intro is *Home on any project*'s (its kicker, its headline with the needs sentence, its lede and byline) and the needs' head "Needs you"; with a spell, today's away form ("While you were away · 23:10 to now", the landed and decided sentences, "Still needs you").
4. **The region,** one view function both pages call, `temper/crates/temper-client-view/src/pages/decided.rs` (moved from `pages/home.rs`'s `decided` and `decision`): "Decided for you · N", "N+" where a shown document's `cut` is set or, unfiltered, `me` has more projects than are watched; "Limits" (`away::Act::Limits`) on Home and a filtered *Needs you*, none unfiltered; the mono line "PROJECT · TITLE · 01:05" on the unfiltered *Needs you* alone; after the took, `temper-decision-allowed` (a new look name, `look::DECISION_ALLOWED`) holding `shared::allowed`'s sentence, then `temper-decision-why` as today. Its right-hand part, struck took and terminal form as today.
5. **What allowed it,** `temper/crates/temper-client-view/src/shared/reach.rs`, re-exported from `shared`: `object` and `allowed` (sketch), the words of 3.5's *Stop asking* OBJECT and *Decided for you*'s *What allowed it*: "Allowed: test-only changes are decided and listed." for class `Decide`, "Allowed: retries and splits are decided while you’re away." for `Ask` with `away`; empty for any other reach. This task passes no classes (`None`): a set reads "changes to a path set" and `under` adds no words; `needs-why` passes the read table.
6. **Needs you** draws the region after its *Answered* section in the flow, and in the done state after the answered lines and before "Home" (3.5, *Needs you*), scoped by the filter's project or unfiltered.
7. **Overturn** (`pages/away.rs`, `overturn`) is accepted at `Address::Home` or `Address::Inbox` for a row `Pages::decided` shows there, live and unmarked; its words and flow as today. The toast `Toasted::Overturned` says "Overturned: it asks you again, under Needs you." The session's marks apply to both pages' rows.
8. **Styles:** `temper-decision-allowed` (13.5 px, muted, shown on a phone) under `/* ==== ui: needs ==== */` in `temper/crates/temper-client-shell-web/temper.css`; the existing `.temper-decision*` rules stay where they stand.
9. **Rows, no new test function** (the request: a new case is a row):
   - the domain: in `src/tests/pages/home.rs`'s decided tests (about lines 1921 to 2363; they may move beside the module, same functions): the watched set's table (Home; filtered and unfiltered *Needs you*, the shown card's project first, five projects in `me` keeping four; another page none; Home to its filtered *Needs you* keeping its watch open), rows of two projects merged newest seven oldest first, `reach` decoded and a reach that fails `reach_from` giving `None`, and the spell's table (on; ended 23 hours ago; ended 25 hours ago, no spell; row undelivered, no spell);
   - the view: `src/tests/home.rs`'s away rows (about lines 3339 to 3698): `the_kicker_alone_with_no_row` becomes the everyday kicker with rows showing and no row; a row with no spell keeps "Needs you"; the allowed line's rows (decide, ask with away, a set with no classes, a reach `None` with no line); `src/tests/needs.rs`'s `done_with_and_without_lines_and_home_project` and `next_line_last_at_and_later_order` (or the nearest flow table) gain the region in both states and the unfiltered mono line; `src/tests/toast.rs`'s `overturned_says_its_goal_asks_again` the new words (rename it to what it says); `src/tests/look.rs`'s `decision_tokens_name_the_away_rows` the new name;
   - the gallery (its tests are over budget, `temper/docs/design/client/browser.status.md`, 7.3: rows only): states `home-decided` and `needs-decided` (`temper/docs/design/client/browser.md`, 7.2) in `samples.rs`, `names.txt` (sorted as it is) and `each_state_holds_what_its_name_says`; `home_away_deliveries_draw_the_mock_words_and_off_span`'s table gains `home-decided` and the allowed line; the fixture decisions of `worlds.rs` (`home_away_record` and its neighbours) carry meaningful reaches: `d-split` a `retry` at `Ask` with `away`, `d-words` a `wording` at `Decide`, the others `implementation` at `Ask` with `away`.
10. **Readers kept to what they check:** `temper/tests/client/tests/suite/away.rs` reads `facts.away` (about lines 640 to 769): read `facts.decided` and `facts.spell` in its place, each assertion kept; the TUI's launchpad calls `shared::home_headline` on Home's facts and is unchanged in code: its goldens move only if a state of theirs shows rows, then regenerated by their test with `SKEIN_CODEGEN_WRITE=1` and named.
11. **Runs,** counts in the report: `heavy cargo nextest run` for `-p temper-client-domain`, `-p temper-client-view`, `-p temper-client-gallery`, `-p temper-client-world`, `-p temper-client-tui`, `-p temper-client-shell-terminal` and `-p temper-client-journeys` (Home's facts change meaning: every reader runs); `heavy cargo check --tests -p temper-client-browser-world -p temper-one-process-world -p temper`; `heavy cargo check -p temper-client-shell-web --target wasm32-unknown-unknown`; clippy on the packages changed, and on `temper-client-shell-web` for wasm32.
12. **Status:** delete the **Not yet** entries naming `ui-reaches/decided-client` in `temper/docs/design/client/README.status.md` and `browser.status.md`.

## Inputs

- `temper/docs/plan/ui-reaches/notes/common.md`, all of it.
- `temper/docs/plan/ui-reaches/notes/contracts.md`, section 3 (`reach_from`) and section 4 (`Decisions`, `Decision.reach`, `Asked`, `cut`).
- `temper/docs/design/client/README.md`, 3.5: **Decided for you** (all of it), *Home on any project* (its parts' order), *Needs you* (*Answered*, *Decided for you*, *Done*, and *Stop asking*'s OBJECT words), *Away* (*Its toasts*); 4.6, *What was decided for a person, the subject `decided`*.
- `temper/docs/design/client/browser.md`, 7.2 (`home-decided`, `needs-decided`) and 7.3.
- `temper/docs/design/domain/forge.md`, 5.9 (the rows and their reach) and 5.10 (`Reach`, `Class`, `Reason`).
- The code: `temper-client-domain`'s `pages/home.rs` (`Home::new`, `decided_told`, `away`, `read_tasks`, `facts`, `look`), `pages/away.rs` (`overturn`, `overturn_told`), `forge/decided.rs`, `pages/needs.rs` (`Session::shown`), `route.rs` (`home::look`'s three calls); `temper-client-view`'s `pages/home.rs` (`decided`, `decision`, `intro_words`, `needs`) and `pages/needs.rs` (`main`, `flow`); survey 3 of this cut's lead found the tests named above.

## Owns and may change

Owns the store and its watches, Home's facts' new fields, the region and its words, Overturn's address rule and toast, the CSS rule, the rows and states above, and the status entries.

Sketch, binding at its outer layer:

```rust
// temper/crates/temper-client-domain/src/pages/decided.rs
//! Decided for you: the open page's projects' decisions (temper/docs/design/client/README.md, 3.5, Decided for you).
pub const DECIDED_PROJECTS: usize = 4;
pub const DECIDED_SHOWN: usize = 7;
#[derive(Debug, PartialEq, Eq)]
pub struct Shown { pub showing: u32, pub more: bool, pub rows: Box<[DecisionLine]>, pub expires: Wall }
#[derive(Debug, PartialEq, Eq)]
pub struct DecisionLine {
    pub project: u32, pub number: u32, pub task: u64, pub title: Option<Box<[u8]>>, pub at: Wall,
    pub question: Box<[u8]>, pub took: Box<[u8]>, pub why: Box<[u8]>, pub overturned: bool,
    pub state: DecisionTask, pub reach: Option<temper_protocol_reaches::Reach>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionTask { Unknown, Live, Done(Wall), Ended(Wall) }
/// A project's `decided` document as the open page holds it.
#[derive(Clone, Copy, Debug)]
pub enum Document<'a> { Unwatched, Unknown, Ended, Delivered(&'a temper_client_protocol_wire::v1::Decisions) }
impl Pages {
    /// The rows showing at the wall, of one project or, with `None`, of every watched one, the session's marks applied.
    #[must_use] pub fn decided(&self, project: Option<u32>, wall: Wall) -> Option<Shown>;
    /// A project's document, for a need's reason (`needs-why`).
    #[must_use] pub fn decisions(&self, project: u32) -> Document<'_>;
}
pub(crate) fn reconcile(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>); // after each step
// and a `told` the pages' routing calls first for its handles, as Home's `decided_told` was.

// temper/crates/temper-client-domain/src/pages/home.rs
pub struct Spell { pub since: Wall, pub end: Option<Wall>, pub landed: Option<u64> }
// Facts: `away` goes; `pub decided: Option<decided::Shown>`, `pub spell: Option<Spell>`.

// temper/crates/temper-client-view/src/shared/reach.rs
/// A subject's plural, as Stop asking and "Allowed:" name it (README 3.5, Needs you, Stop asking).
pub fn object(w: &Words, subject: temper_protocol_reaches::Subject, under: u8, classes: Option<&temper_protocol_reaches::Classes>) -> Box<[u8]>;
/// What allowed a decided reach (README 3.5, Decided for you); empty for a reach that asked.
pub fn allowed(w: &Words, reach: &temper_protocol_reaches::Reach, classes: Option<&temper_protocol_reaches::Classes>) -> Box<[u8]>;
```

May change, each only as said:

- `temper/crates/temper-client-domain/Cargo.toml` and `temper/crates/temper-client-view/Cargo.toml`: the dependency line `temper-protocol-reaches = { path = "../temper-protocol-reaches" }` (and their test worlds' manifests where a test names the kit's types).
- `temper/crates/temper-client-view/src/pages.rs`: the `mod decided` line.
- A test or fixture elsewhere that builds `home::Facts` or reads `facts.away`, `AwayForm` or `home::DecisionTask` (found by the compiler): its field renamed, its assertion kept, each named in the report.
- The shells' limits (`temper/crates/temper-client-shell-web/src/limits.rs`, `temper/crates/temper-client-shell-terminal/src/limits.rs`, `temper/crates/temper-client-tools/src/limits.rs`) and the client world's `fixtures::domain_limits`: only under the option below.

Options, each decided:

- **The watch table is full** on the unfiltered *Needs you* (a test or world sees a watch refused for room): raise `watches` and `handles` by 3 in the three shells' limits and in the worlds' fixture limits that need it, each comment saying it holds four projects' `decided`; say so. Do not lower `DECIDED_PROJECTS`.
- **The reads table is full** with sixteen cards and seven decisions: it is not (`reads: 32` counts both); if a test shows it is, say so and stop for that test alone, naming it under `## Left`.
- **Home's existing decided tests** may stay in `tests/pages/home.rs` or move to a `tests/pages/decided.rs` beside the module, the same functions, with no new one.
- **A journey's print moves** (none is expected: survey 3 found no recording showing decided rows): regenerate it with `temper-client-journeys again` (the request's retries holding) and name it.
- **The words' numbers** in `words.rs` (each `Word`'s id): take the next free ones as the file does.

## Steps

1. Read the Inputs. Build the store and move Home onto it (acceptance 1, 2, 3); its domain rows (acceptance 9, the domain's), and bite the watched set's row: keep Home's watch on the unfiltered *Needs you* by dropping the address match, see the row fail, restore.
2. The view: the shared region and its words, Home and *Needs you* (acceptance 4, 5, 6); Overturn (acceptance 7); the CSS rule (acceptance 8); the view's rows, and bite the allowed line's row (return an empty sentence for `Decide`, see it fail, restore).
3. The gallery's states and rows; the client world's readers (acceptance 9, 10).
4. The runs of acceptance 11; the status entries; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`; push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report: the store's watched-set rule as written, Home's facts' fields, the region's function, Overturn's rule, the options taken (the watch table above all, with the counts that decided it), the rows and their bites, the gallery states, any golden or print regenerated, and the runs' counts.

## Rebase (2026-11-08, step merge, foreman ui-reaches/decided-client/foreman-1)

resolve the How work runs conflicts, preserving both features and correcting combined counts.
Rebase onto main; the merge refused three paths:
- `temper/crates/temper-client-view/src/look.rs`
- `temper/crates/temper-client-view/src/words.rs`
- `temper/tools/gallery/src/samples.rs`

Main’s overlapping commits are `7ae1fdcc0b` (How work runs look), `db801753b4` (page and vocabulary), and `db11d6bc71` (six gallery states).

In `look.rs`, keep main’s twelve `temper-how-*` names at tokens 646–657, then append `temper-decision-allowed` and change `DECISION_ALLOWED` to token 658. Keeping its current token 646 would silently name `HOW_WIDE`.

In `words.rs`, retain both features’ variants, `ALL` entries, English phrases and IDs. Keep main’s removal of obsolete Settings words and this branch’s removal of obsolete Away words. The combined declarations are `ALL: [Word; 1436]`, `PHRASES = 2118`, `SEGMENTS = 3741`, and `ENGLISH_BYTES = 51491`; verify with the existing declaration check. `SEGMENTS` merges without a conflict but still needs updating. Preserve reach IDs 4318–4333 and How IDs 4900–4929.

In `samples.rs`, retain main’s six How samples, mockup mappings and domain routing alongside `home-decided`, `needs-decided` and their deliveries. Set `N = 352`.

Validate through heavy with focused view vocabulary/look tests, including `decision_tokens_name_the_away_rows`, and gallery tests `each_state_holds_what_its_name_says` and `home_away_deliveries_draw_the_mock_words_and_off_span`. Preserve both features in the automatically merged gallery names, tests and worlds. Regenerate and report any resulting stale generated artifacts using their existing tools.
