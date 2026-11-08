+++
id = "away-all"
part = "away-all"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-domain/src/pages/away_all.rs", "temper/crates/temper-client-domain/src/pages/away.rs", "temper/crates/temper-client-domain/src/pages.rs", "temper/crates/temper-client-domain/src/pages/rail.rs", "temper/crates/temper-client-domain/src/pages/needs.rs", "temper/crates/temper-client-domain/src/tests/away.rs", "temper/crates/temper-client-domain/src/tests/pages/rail.rs", "temper/crates/temper-client-view/src/frame.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/tests/rail.rs", "temper/tools/gallery/src/samples.rs", "temper/tools/gallery/src/worlds/whole_story.rs", "temper/tools/gallery/src/tests.rs", "temper/tools/gallery/names.txt", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 110
+++

## Goal

On a page across projects (All projects' Home, Needs you unfiltered),
the rail's Away switch and the phone's chip are the person's projects'
together: they read "Away on 2 of 4" while the projects differ and as a
single project's Away when all agree, and a press sets every project the
person is in through the existing per-project `SetAway`, one request
each; the toast says how many projects changed. Today the switch there is
the rail's open project's alone.

Acceptance:

- `temper-client-domain` has `pages/away_all.rs` (sketched below):
  `Across` and `across`, one pure function counting the person's
  projects and those on; `away::Act::ToggleAll`; the requests out in
  `Pages`; `Toasted::AwayAll`. `rail::facts` reads `across(..).shown`
  for `away` on a page across projects (`rail::across`), and the open
  project's `away_shown` elsewhere, as now.
- `ToggleAll` sends a `SetAway` for each of `me`'s projects not already
  as asked (with all on, each off; otherwise each on), each the very
  request a project's `Toggle` sends for it; while any is out every Away
  act waits and `away_busy` holds; once the last answer is in, the toast
  is `AwayAll { on, changed, sent }`, or `AwayRefused` with none changed.
- `temper-client-view`'s `frame.rs`, `away`, on a page across projects,
  draws the across words and state, binds `ToggleAll`, and asks to wake
  at the earliest `until` of the rows on; the toast speaks `AwayAll`.
- The tests below pass: domain table tests (one for `across`, one for
  `ToggleAll`) and rows of existing tables; the view's switch test made a
  two-row table; two gallery states, `all-home-away` and
  `all-home-away-on`, with their `Holds` rows.
- The **Not yet** entry 3.5, *Away*, *On All projects*, in
  `temper/docs/design/client/README.status.md` is removed, its lines
  alone. This departs from the implementer's text, which closes
  `docs/design/` to you: the entry is this task's.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Away*: *The control*,
  *Its act*, *Past `until`*, *Its toasts*, and *On All projects*, which
  this task builds; 5.3, the table of the rail's parts, its Away row.
- `temper/docs/design/client/browser.md`, 7.2: the rows `all-home`,
  `all-home-away` and `all-home-away-on`.
- `temper/docs/design/draft/surfaces.md`, section 2, the rail's foot.
- `temper/docs/plan/ui-away-all/inputs/request.md`.
- `temper/docs/plan/ui-away-all/notes/common.md`.

## Owns and may change

- Owns the files in `files`.
- Compiler-forced arms for the new `away::Act` and `Toasted` variants in
  any file of `temper/` (today: `temper-client-view`'s `frame.rs`,
  `pages/settings.rs`; `temper-client-domain`'s `pages/needs.rs`); each
  edited for that alone.
- `temper/tools/gallery/src/worlds.rs`: only if a helper there
  (`sample_attempt`, `sample_forge_answer`) needs to be reachable from
  `worlds/whole_story.rs` or to answer several attempts; prefer calling
  them as they are (a child module sees its parent's private items).
- Pre-approved options:
  - If the client bounds the requests out below the person's projects,
    so that sending them all at once refuses some, send them one after
    another, each on the last's answer, keeping `Pending`'s counts; say
    so in the report.
  - If the gallery's `Part` has no check that a named button is not
    pressed, add `Part::NotPressed { name }` beside `Part::Pressed`, its
    check the inverse.
  - If moving `rail::facts`' `away` changes a row of a test whose root
    sits on a page across projects, the new value is the rule: change
    the row's expected `away`, keep the rest, and name it in the report.
  - If an `all-home-away*` state's heading or kicker differs from
    `all-home`'s because rows are on, its `Holds` row takes what the
    state draws; name it in the report.
- Not yours: `away::ask` and `forge/away.rs` (the per-project request is
  reused, never restated), `Domain::away_on` and `Domain::away_shown`,
  `temper.css` (no rule is needed: the words fit the switch and chip as
  drawn), `temper/docs/help/**`, `temper/tests/client/**` (its stories
  run unchanged), any other design text.

## Steps

1. **The count,** a new `pub mod away_all;` beside `away` in `pages.rs`:

   ```rust
   //! Away on All projects (temper/docs/design/client/README.md, 3.5, Away, On All projects).

   /// The person's projects' Away together.
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub struct Across {
       /// M: the projects `me` lists; 0 before `me`.
       pub projects: u32,
       /// N: those whose row is on at the wall (`Domain::away_on`).
       pub on: u32,
       /// The earliest `until` of the rows on; None where none has one.
       pub until: Option<Wall>,
       /// 3.5's *Away* shows for any of them (`Domain::away_shown`).
       pub shown: bool,
   }

   #[must_use]
   pub fn across(root: &Domain, wall: Wall) -> Across;
   ```

   `rail::facts`: `away` is `away_all::across(root, wall).shown` when
   `rail::across(root)`, else `root.away_shown(project, wall)` as now.

2. **The act,** `away::Act::ToggleAll`, a variant after `Toggle`.
   `away::act` hands it to `away_all::act` before anything else, as it
   does `Overturn` and `Limits`. The lines of `away::act` that turn a
   project's `Toggle` (and `Kinds`, `Until`) into its `on`, `until` and
   `SetAway` become one `pub(super)` function that both `away::act` and
   `away_all::act` call, so the per-project rule (the row's `until` and
   `kinds`, the defaults, the next 08:00) is written once; its shape is
   yours. `away_all::act`:
   - returns at once while `away_busy()` or `!root.forge.away_available()`;
   - reads `across`: `on` is `across.on < across.projects`;
   - for each of `me`'s projects whose `root.away_on(project, wall)` is
     not `on`, builds that project's `Toggle` request and sends it with a
     fresh page key (`Owner::Pages`, `pages.serial`), as `away::act`
     does, counting it in `pages.shown` as `away::act` does;
   - keeps the keys out in `Pages`, a field `away_all:
     Option<away_all::Pending>` beside `away`, `None` at start:

   ```rust
   /// Away on All projects' requests, one a project that changes.
   #[derive(Debug)]
   pub(super) struct Pending {
       on: bool,
       out: List<Key>,
       sent: u32,
       changed: u32,
   }
   ```

   `away_busy()` holds while either `away` or `away_all` is out, and
   `away::act`'s own check reads `away_busy()`, so a project's act waits
   behind `ToggleAll` and `ToggleAll` behind a project's request.

3. **The answers,** `away_all::told(pages, env, client, told) -> bool`,
   asked by `away::told` first, as it asks `overturn_told`. An answer to
   a key out (or `Told::Pending` whose progress is `Unknown`, read as
   `away::told` reads it) takes the key out and counts `changed` where
   `away::answer` is `Answer::Away`. When none is left: `pages.away_all
   = None` and the toast, `Toasted::AwayAll { on, changed, sent }`, or
   `Toasted::AwayRefused` with `changed` 0. `Toasted::AwayAll { on: bool,
   changed: u32, sent: u32 }` goes after `AwayRefused`.

4. **The view,** `frame.rs`, `away(..)`: when `rail::across(root)`,
   read `away_all::across(root, times.now)` and draw:
   - `on` 0: the off words as now ("Away"; the chip's "Away: off"), not
     pressed;
   - `on == projects` (and `projects` above 0): the on words with
     `across.until` ("Away until 08:00", or "Away" with none), pressed;
   - between: "Away on N of M", a new `Word` (holes N, M), not pressed;
   - `times.next` takes `across.until` as it takes a row's `until`;
   - the binding `PageAct::Away(away::Act::ToggleAll)`; disabled while
     `away_busy()`, as now.

   Elsewhere `away` is as it is. The toast, in `frame.rs`'s toast:
   `AwayAll` with `changed == sent`: "Away turned on for N projects:
   agents decide within their limits." or "Away turned off for N
   projects: questions wait for you again.", each with its one form,
   "for 1 project"; with `changed < sent`: "Away changed for N of M
   projects. Try again." (N `changed`, M `sent`). No "Limits" act. Words
   go in `words.rs` as its others do; the one form as
   `Word::WaitingForDelegates` has one, or a word of its own, your
   choice.

5. **Domain tests,** `src/tests/away.rs`, with a `me` of three projects,
   `f::PROJECT`, 17 (`OTHER`'s) and one more, delivered on the frame's
   watch (as `tests/pages/rail.rs`' `two_projects` builds two), and a
   `sends` beside `sent` that collects every `SetAway` in order:
   - **`across_counts_the_projects_on`,** a table: no rows, `{3, 0,
     None, false}`; `WHAT`, `{3, 1, None, true}`; all three on, untils
     480, none and 600, at 479, `{3, 3, Some(480), true}`, and at 480,
     `{3, 2, Some(600), true}`; `me` undelivered, `{0, 0, None, false}`.
   - **`toggle_all_sets_each_project_that_differs`,** a table of rows,
     answers in send order, and the toast:

     | row | rows | sends | answers | toast |
     |---|---|---|---|---|
     | none on | none | the three on, `until` the next 08:00, `kinds` 7 | Away ×3 | `AwayAll { on: true, changed: 3, sent: 3 }` |
     | one on | `WHAT` | 17 and the third, on | Away ×2 | `AwayAll { on: true, changed: 2, sent: 2 }` |
     | all on | the three, `until` 480, `kinds` 8 | the three off, keeping `until` and `kinds` | Away ×3 | `AwayAll { on: false, changed: 3, sent: 3 }` |
     | one refused | none | the three on | Away, wire `Unknown`, Away | `AwayAll { on: true, changed: 2, sent: 3 }` |
     | none changed | none | the three on | `Outcome::Lost`, `Dismissed`, `ForgeRefused` | `AwayRefused` |

     Each row also asserts: `away_busy()` and no toast until the last
     answer; a project's `Toggle` and a second `ToggleAll` sent while
     out send nothing; an answer to another key changes nothing; after
     the toast `away_busy()` is false. Each send equals what a lone
     `Toggle` for that project sends from the same rows (compare with
     `sent(&act(.., A::Toggle { project }, ..))` on a fresh root).
   - **`a_second_toggle_waits`:** `A::ToggleAll` joins its list of acts
     that send nothing while a project's request is out.
   - `src/tests/pages/rail.rs`, **`facts_gather_the_open_projects_domain_values`:**
     a row "across", `project_domain()` gone to `Address::AllHome`, for
     `OTHER`: `away: true` (the open project's Away shows), where the row
     "other project" at Settings reads `false`.

   Break each rule once by hand and see its row fail for that reason:
   count `on` over rows not on at the wall (the 480 row fails); send to
   every project, changed or not (the "one on" row fails); toast on the
   first answer (busy fails); read the open project in `rail::facts` on
   All projects (the "across" row fails). Restore each.

6. **View tests,** `src/tests/rail.rs`: turn
   `the_switch_shows_with_ways_away` into two rows: as now, on
   `Address::Home { project: f::PROJECT }`, pressing to `Toggle {
   project: f::PROJECT }`; and the same root gone to `Address::AllHome`
   (`f::root_event` with `Event::Went`, as
   `the_panel_puts_all_projects_first_and_links_after_the_forms` goes
   across), pressing to `ToggleAll`. Both are named "Away" and not
   pressed: no row is on.

7. **The gallery,** `World::WholeStory`, `Address::AllHome`, `then:
   Shown`, beside `all-home-one-project` in `samples.rs`, sorted in
   `names.txt`:
   - `all-home-away`: `whole_story.rs`' Aways delivery adds vsr-rs (2)
     and cowsql (3) on (`since` 30 minutes before now, `until` none,
     `kinds` 7, `landed` 0, `off` none) beside vsr's ended spell. Its
     `Holds`: `Part::Named { role: Role::Button, name: "Away on 2 of 4"
     }`, and not pressed.
   - `all-home-away-on`: `all-home-away`, then `ToggleAll` pressed at
     the world's now, each `SetAway` sent answered `w::Outcome::Away` (as
     `rail_away_toggle` answers one), and the Aways watch delivered again
     with each sent request's row on (its `project`, `until` and
     `kinds`, `since` now) beside vsr-rs's and cowsql's. Its `Holds`:
     `Part::Pressed { name: "Away until 08:00" }` and `Part::Text {
     text: "Away turned on for 2 projects: agents decide within their
     limits." }`.

   Regenerate what the new states make stale by the gallery's own tool
   (`names.txt` as `names_are_the_promised_ones` writes it).

8. **The status entry:** remove the **Not yet** 3.5, *Away*, *On All
   projects*, from `temper/docs/design/client/README.status.md`, its
   lines alone.

9. **Runs.** `rail::facts` changes what `away` means on a page across
   projects, so its readers' tests run too, narrowing the implementer's
   text's "direct dependents" run to these:
   - `heavy cargo nextest run -p temper-client-domain --lib`
   - `heavy cargo nextest run -p temper-client-view --lib`
   - `heavy cargo nextest run -p temper-client-gallery --lib`
   - `heavy cargo nextest run -p temper-client-shell-terminal --lib`
   - `heavy cargo nextest run -p temper-client-world --test suite -E 'test(/rail/) | test(/density/) | test(/^away::/) | test(/^team::/)'`
   - `heavy cargo check --tests -p temper-client-tui -p temper-client-shell-tui -p temper-client-shell-web`

   Check each run's count; one that selected nothing proves nothing. If
   a journey recording goes stale, run `temper-client-journeys again`
   (`notes/common.md`).

10. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy
    cargo clippy -p temper-client-domain -p temper-client-view -p
    temper-client-gallery --all-targets -- -D warnings`; commit saying
    why (Away on All projects sets every project, as `surfaces.md`, 2,
    has it); push.

## Report

As the implementer's text has it, and: the shape of the shared
per-project function; whether the requests went out at once or one after
another, and why; each rule broken for a bite and the failure seen; each
reader test row you changed and why; the gallery states' headings if
they differ from `all-home`'s; the counts of the runs in step 9.
