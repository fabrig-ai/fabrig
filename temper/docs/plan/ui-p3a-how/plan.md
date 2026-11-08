# ui-p3a-how: How work runs, the page and the path, on today's settings

The request: `inputs/request.md`, milestone P3a of
`temper/docs/design/draft/whole-story.execution.md`: `surfaces.md`, section 7, but
for what P3b to P3d add, and `stock-ways.md`, section 4, the look
`web/whole-story/how.html`. `notes/common.md` holds the rules every task follows;
`notes/contracts.md` the shapes that cross tasks.

The design this cut wrote, on the trunk with the first push:

- `temper/docs/design/client/README.md`: 3.5's page table row for `Settings {
  project }` and *Settings*' heading row; 5.13, *The page* (new: its heading and
  lede, "Ask temper to change this", the path of a change, a path in lanes,
  Budgets, the live rows kept, the foot), *Its parts* (new), *Its rows* (today's
  live rows, the disabled Status, Funds and Telling you gone), *A way's acts*
  (both pages watch the ways); section 11, *How work runs' later parts* (new), in
  place of *How work runs' disabled rows*.
- `temper/docs/design/client/browser.md`: 2.5, *How work runs' look* (new); 7.2,
  the states `how` to `night-how`.
- `temper/docs/design/client/greenfield/README.md`: what How work runs now takes
  from `how.html`, and what it still takes from `settings.html`.
- Status: **Not yet** entries in `README.status.md` (5.13's parts and page) and
  `browser.status.md` (2.5's look, 7.2's states), each deleted by its task;
  `draft/surfaces.status.md`'s lag 7 narrowed to what P3b to P3d and section 11
  leave.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `parts` | `tasks/how-parts.md` | | `pages::how::{parts, facts, lanes}`; the `ways` watch on `Settings { project }`, its tab and stop acts heard there | 80 |
| `look` | `tasks/how-look.md` | | `look.rs`'s `temper-how-*` names; `temper.css`'s `ui: how`: the lanes, who acts' colours day and night, the ways on Settings | 60 |
| `page` | `tasks/ways-shared.md` | | the ways' tabs, card, act and refusal moved to `pages/ways.rs`, Ask temper's page unchanged | 40 |

Released in the first batch. Planned next (names may still move until released):

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `page` | `how-page` | how-parts, how-look, ways-shared | How work runs drawn: heading and lede, Ask temper's offer, the path section with tabs, lanes, card and act, Budgets; the disabled rows gone, the switches at its foot; the terminal's golden and stale journeys regenerated |
| `checks` | `how-fuzz` | how-parts | section 12's rules over `how::parts` and `how::lanes`, random settings and their moves |
| `checks` | `how-world` | how-page | the client world: How work runs' lanes over a project with CI and tiers and with neither, each stock way |
| `checks` | `how-gallery` | how-page | the gallery's six states, `how` to `night-how` |

**Critical path as it stands:** `how-parts` (80) → `how-page` → `how-gallery`;
`how-look` and `ways-shared` end before `how-parts` does.

## What was decided

- **How work runs is `Settings { project }`,** the page `ui-p1-rail`'s rail row
  opens: this plan changes no address, route or rail, and meets P1 at that row.
  The frame (kicker, tabs) stays P1's; the page gives the frame its heading and
  lede and draws its body.
- **The ways stay on Ask temper's page too.** How work runs takes up
  `look-round-24`'s ways, the one `ways` read and its acts, drawn in lanes as
  `how.html` draws a path; Ask temper's opening keeps its pictures
  (`whole-story.md`, 5.1, step 2). Both pages watch the read; the tabs, card and
  act are one module of the view (`ways-shared`).
- **The path in lanes is a table,** a column a stop of the engine's derivation
  (`forge.md`, 8.13) and a row a lane, the authority line a row of one cell
  between Agents and Code: a table is what a screen reader and the text shell
  read as rows, and what the mock draws as a grid. A person's cell says its stop's
  words, an agent's or code's its card's name ("the coding agent", "CI"); a column
  header names the step. The stop's card is Ask temper's.
- **The heading names the way,** the read's name ("Agents review"), or "A way of
  its own"; its chief departure is P3c's, since no rule by path exists before it.
  The lede is the way's line. Nothing is written by hand.
- **Budgets is a goal's alone,** while Goals is shown and its template gives an
  amount or a cap; a role's budget is P3d's. The stock's goal templates give none,
  so a stock project draws no Budgets.
- **Left out, as the rule of every milestone says:** What reaches you, Rules by
  path, What follows a landing and a role's budget; "History · N changes", since no
  page lists the settings' revisions; the next step's suggestions (`surfaces.md`,
  12). What needs you and While you're away, today's live rows, stay until What
  reaches you takes their place; the disabled Status, Funds and Telling you rows go.
- **Repositories stays How work runs' second tab,** as `ui-p1-rail` designed it,
  not the page's foot: `surfaces.md`, 15's default places it in How work runs, and
  folding it into the foot would rewrite P1's frame and the 38 files that read
  `Address::Repositories`. Section 11 and the draft's status say so.
- **No stored format, protocol or public interface of the engine changes:** the
  page reads the `ways` watch the ways page reads.

## Who hears first

- **The TUI's owner (`temper-client-domain`):** `pages::how` (new, `how-parts`):
  `parts`, `facts` and `lanes`; `StockWays` now opens on `Settings { project }` as
  on `Ask { project }`, and `Act::WayTab` and `Act::WayStop` are heard on either.
  No `Address` variant, act or route is added.
- **The owner of coding runs:** no change to `temper-engine-domain` or
  `forge_route.rs`.

## Overlaps with drafts beside this one

- **`ui-p1-rail`'s `deployment`** owns `temper-client-view`'s `pages/settings.rs`
  (the frame and tabs) and `tests/settings.rs`: `how-page` changes `settings.rs`'s
  `how_work_runs` and, in `open`, How work runs' heading and lede alone, and
  `tests/settings.rs`'s How work runs row. Unavoidable: How work runs' body and
  head are drawn there; the second to land merges them.
- **`ui-p1-rail`'s `addresses`** and **`ui-p2-home`** change
  `temper-client-domain`'s `pages.rs`: `how-parts` changes the one block opening
  `StockWays` and the two way-act arms, and adds `pub mod how;`.
- **Shared registries:** `look.rs`'s `NAMES` (all three plans append; the second
  to land renumbers its own), `words.rs` (this plan's keys 4900 to 4949, P1's 4600
  to 4799), `temper.css` (this plan's `ui: how` after the ways' rules; P1 writes
  under `ui: settings` and adds `ui: rail` and `ui: work`).
- **Generated and gallery files:** the gallery's `samples.rs`, `tests.rs`,
  `worlds/*.rs` and `names.txt` (P1's and P2's gallery tasks), the text shell's
  golden `settings.txt` (P1's `regenerate`) and the `chat-draft` journey's prints:
  regenerated on a catch-up, never hand-merged.
- **Design and status:** `README.md` 3.5's *Settings* table (P1 changes its later
  rows; this plan its first), 5.13's switches table (P1 changes the Changes row's
  words; this plan moves the table under *Its rows*); `surfaces.status.md`'s lag 7,
  next to P1's removal of lag 8; `README.status.md` and `browser.status.md`, this
  plan's entries at their ends; the greenfield README, this plan's paragraph after
  the page list, P1's before it.
- **`ui-reaches`** will change Away and kind 7's `away` member, which How work
  runs' foot keeps as today's rows in `settings.rs`; nothing of theirs is released
  that this plan touches.
- `whole-story.execution.md`'s state column for P3a is left to the landing, as
  P1's and P2's are.
