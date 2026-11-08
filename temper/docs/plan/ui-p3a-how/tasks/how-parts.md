+++
id = "how-parts"
part = "parts"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-domain/src/pages/how.rs", "temper/crates/temper-client-domain/src/tests/pages/how.rs"]
effort = "high"
minutes = 80
+++

# How work runs' parts and lanes, and the ways watched there

## Goal

Give `temper-client-domain` How work runs' one pure function of its parts and the
lanes of a path (`temper/docs/design/client/README.md`, 5.13, *Its parts*), and
watch the project's `ways` on `Settings { project }` as on `Ask { project }`, so
that the view (`how-page`) draws the page from them and the fuzz test
(`how-fuzz`) checks them.

Acceptance:

1. `pages::how` exists exactly as `notes/contracts.md`, C1, sketches it: `Read`,
   `Facts`, `Head`, `Path`, `Budget`, `Parts`, `parts`, `facts`, `Lane`, `Column`,
   `lanes`.
2. `parts` holds 5.13's rules: `head` is `Way(i)` while the read is shown with
   `current` `Some(i)`, `Own` while shown with none, `Reading` otherwise; `ask`
   is `asking`; `path` follows the read; `budget` is `Some` only while `goals`
   and the budget is `Amount(n)` (`Budget::Amount(n)`) or `AsAllowed { cap:
   Some(n) }` (`Budget::UpTo(n)`), `None` for `Given`, `AsAllowed { cap: None }`,
   no template, or Goals hidden.
3. `lanes` gives one column a stop, in order: a person's stop in `You`, an
   agent's in `Agents`, code's in `Code`, a bar as `Bar`; every stop once.
4. `facts` gathers as C1 says: `asking` from `me`'s templates
   (`jig_client::shapes::template(me, project, chats::ASKING_TEMPLATE)`);
   `read` from the `StockWays` held; `goals` and `budget` from the forge's kind
   7 (`root.forge().ways(project)`) and the template
   `forge::ways::default_way(me, project, &ways)` names.
5. C2: the `ways` watch opens on `Settings { project }` and is left when the
   address leaves it, as on `Ask`; `Act::WayTab` and `Act::WayStop` are heard
   on `Settings { project }`.
6. The focused tests of `temper-client-domain` pass, clippy clean;
   `heavy cargo check --tests` passes for `temper-client-view`,
   `temper-client-tui` and `temper-client-world`. The entry "**Not yet:** 5.13,
   *Its parts*" in `temper/docs/design/client/README.status.md` is gone.

## Inputs

- `temper/docs/plan/ui-p3a-how/notes/common.md`; `notes/contracts.md`, C1 and C2.
- `temper/docs/design/client/README.md`, 5.13: *The page* (its first bullets, for
  what the parts mean), *Its parts*, *A way's acts*; `../draft/surfaces.md`,
  section 12, the rule "How work runs' rules by path, follows, budgets | one is
  set".
- `temper/docs/design/domain/forge.md`, 8.13 (a path's actors and bars).
- The code: `temper-client-domain`'s `forge/stock_ways.rs` (`StockWays`, `State`,
  `tabs`), `forge/ways.rs` (`Ways`, `Side::ways`, `default_way`), `pages.rs`
  (`went`'s block opening `stock_ways` on `Ask`, the `WayTab`/`WayStop` arms,
  `Pages::me`), `pages/chats.rs` (`ASKING_TEMPLATE`); the shape of a page's pure
  function in `pages/start_goal.rs`'s `default_way` and `unfunded`; jig's
  `contract::TemplateBudget`.
- The tests: `src/tests/forge_stock_ways.rs`
  (`ways_watch_has_project_scope_and_the_pages_lifetime`, the fixture's `panel`
  and `stops`), `src/tests/pages.rs`'s module list.

## Owns and may change

Owns: `temper-client-domain`'s new `src/pages/how.rs` and
`src/tests/pages/how.rs`.

Reader and neighbour files, and what may change there:

- `src/pages.rs`: `pub mod how;` beside `pub mod home;`; the `went` block that
  makes `StockWays` matching `Address::Settings { project }` too; the
  `WayTab`/`WayStop` arms' address check admitting `Settings { project }`.
  Nothing else: `ui-p1-rail` and `ui-p2-home` change this file beside you.
- `src/tests/pages.rs`: `mod how;`.
- `src/tests/forge_stock_ways.rs`: rows or assertions in
  `ways_watch_has_project_scope_and_the_pages_lifetime` for `Settings`; no new
  test function.
- `temper/docs/design/client/README.status.md`: the one entry above, deleted.

Pre-approved options:

- **If `Facts` needs one more fact** to decide a part 5.13 gives (no other): add
  the field, keep C1's others, and say so in the report.
- **If going from `Ask` to `Settings` of one project** would keep the watch
  rather than leave and open it again: either is fine; keep `StockWays`' shown
  tab and open stop as its own rules have them, and say which.
- **If `facts` can't name the `StockWays` held for another project** (it holds
  one for the address's project): `read` is `Reading` for any other project.

## Steps

1. `pages/how.rs`: C1's types and functions. `lanes` matches `w::Actor`'s four
   variants; `parts` matches `Read` and `c::TemplateBudget` whole.
2. `pages.rs`: C2's three edits.
3. Tests, `src/tests/pages/how.rs`: one table test of `parts`, a row per rule of
   acceptance 2 (each `Read`, `asking` both ways, each `TemplateBudget` with
   Goals shown and hidden, no template), and in it the rows of `lanes` over a
   path of every actor, a goal's bar and main's, checking each stop placed once,
   in order, in its actor's lane; then `facts` over a fixture's `me`, kind 7 and
   ways deliveries, a row each for Ask temper listed and not and a template's
   budget read. Rows in `forge_stock_ways.rs` for C2. Break two rules by hand (a
   cap's budget given as `None`; an agent placed in `Code`) and see their rows
   fail; restore.
4. Check as `notes/common.md` says. Delete the status entry. Commit and push:
   the message says How work runs' parts and lanes are one pure function each,
   and the ways are watched on How work runs.

## Report

Follow the implementer's text. Also give the rules you broke and the failures you
saw, and each test's CPU time.
