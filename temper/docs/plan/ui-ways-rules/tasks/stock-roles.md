+++
id = "stock-roles"
part = "roles"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/stock/coordinator.md", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper-service/src/tests/stock_brief.rs", "temper/crates/temper-service/src/tests/welcome_policy.rs", "temper/crates/temper-service/src/tests/fixtures.rs", "temper/crates/temper-service/src/tests/root_checks.rs", "temper/crates/temper-service/src/tests/startup.rs", "temper/crates/temper-playground/src/documents.rs", "temper/crates/temper-playground/stories/a/deployment.json", "temper/crates/temper/tests/support/configuration.rs", "temper/crates/temper/tests/suite/startup.rs", "temper/crates/temper/tests/suite/creating.rs", "temper/crates/temper/src/tests/settings_fields.rs", "temper/crates/temper/src/tests/reference_settings.rs", "temper/crates/temper-init/src/tests/documents.rs", "temper/tests/client-browser/src/configuration.rs", "temper/tests/client-browser/src/lib.rs", "temper/tests/engine/domain/tests/suite/settings.rs", "temper/tests/engine/domain/tests/suite/settings/b2.rs", "temper/tests/engine/domain/tests/suite/configuring/fixture.rs", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/engine/domain/tests/suite/forge_routes/goal_branch.rs", "temper/tests/engine/domain/tests/suite/forge_routes/per_wake.rs", "temper/docs/help/charter.md", "temper/docs/help/settings.md", "temper/docs/design/client/journeys/*/recording", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 170
+++

# Draft first's coordinator and goal template join the stock

## Goal

`temper/docs/design/draft/stock-ways.md`, 3.3: the stock holds charter 5,
the coordinator, in its first edition, the words of this plan's
`inputs/charter5.md` exactly, in its shared instructions and four parts;
template 4, a goal for a draft, run by charter 5 at the stock's models
with its own goal branch among its resources; and the rules' and policy
1's delegation reaches charter 5, role 0 and the merge job, so that a goal
for a draft runs on any project with no settings change. Built to
`notes/contracts.md`, section 3. Every fixture and count that pins the
stock's charters, templates and delegation moves with it.

Acceptance:

- `temper/crates/temper-service/src/stock/coordinator.md` is a byte copy of
  `temper/docs/plan/ui-ways-rules/inputs/charter5.md` (`cmp` silent; the
  report says so). No task edits its words. **If a stock bound or test
  refuses any of it** (`root.journal.run_bytes`' 20,480 at `temper init`'s
  document, `core.run_bytes`, a host part's room of at least 2,048 bytes,
  `tasks.charter_parts`, jig's check of a charter), stop with `Question:`,
  naming the bound, its figure and the bytes charter 5 needs, after
  pushing what you have.
- **Charter 5's parts loaded as written, with their `on` sets:** a new test
  in `temper/crates/temper-service/src/tests/stock.rs`: charter 5's
  `policy.instructions` is the file's `## shared` section and its `parts`
  are four, in the file's order, their `on` sets `FIRST`; `QUESTION |
  QUESTION_ANSWERED | QUESTION_ENDED | QUESTION_OWED`; `RESULT | NOTICE |
  NOTHING_AWAITED | ESCALATION | PROPOSAL | PERSON_STALLED`; and `NEWS |
  TIMER`, each part's instructions its section's text by the contract's
  rule. The test splits the file by its own reading, not by calling the
  stock's splitter, and checks that the five sections and their headings
  are the whole file. Charter 5's other values (resume, waiting, run,
  tools, wake, documents) are rows of the stock's charter tests where
  they have one.
- **The stock's delegation to charter 5:** `stock_and_init_admit_the_landing_procedure_and_its_effects`
  holds the contract's fourteen kinds for the rules, policy 1's ceiling
  and its roles 0 to 2; and the domain world's
  `reference_template_four_starts_on_the_configured_forge`
  (`forge_routes/goal_branch.rs`) starts its goal from the stock's own
  charter 5 and template 4 under the stock's own delegation, adding
  neither (rename it `the_stock_template_four_starts_on_the_configured_forge`).
- Template 4 is as the contract has it, its report the deployment's
  `result_bytes` (`the_stock_reports_follow_the_deployment_result_bound_and_keep_its_floor`
  holds it).
- Every test that pinned seven charters, three templates or twelve
  executor kinds, the stock policy's encoded bytes (5,822 today) or the
  stock brief's per-charter figures holds the new figures, measured, not
  guessed; the report lists each with its old and new figure.
- The **Not yet** entry you close is deleted: in
  `temper/docs/design/protocol/configuration.status.md` the bullet starting
  `13.4: charter 5 and template 4 are not in the stock`.
- Clippy clean, fmt clean, the focused tests below green; the browser
  tier's fixtures compile (`heavy cargo check -p temper-client-browser-world --tests`),
  not run.

## Inputs

- `temper/docs/plan/ui-ways-rules/notes/common.md` and `notes/contracts.md`,
  section 3 (binding).
- `temper/docs/design/protocol/configuration.md`, 13.4: the rows *charter
  5, the coordinator's*, *template 4, a goal for a draft* and *the rules*,
  and the bullets *Each charter's host families*, *The stock sets floors
  under the limits* and *Built at each start*; 13.9, the bullets *A
  settings file* and *The rules and policy 1*, for the reference's
  editions over the stock.
- jig's `jig/docs/design/domain/tasks.md`, 8.6 (a charter in parts).
- `temper/docs/design/draft/stock-ways.md`, 3.3, for why.

Departures from the implementer's text, and why:

- **You delete one status entry,** named above. Touch no other design text.
- **Help pages:** `temper/docs/help/charter.md` and
  `temper/docs/help/settings.md` list the stock's charters, templates and
  executors in words: bring those lines to the new stock, within each
  page's bound.

## Owns and may change

Owns:

- `S/stock.rs` (`S` is `temper/crates/temper-service/src`): charter 5
  and its splitter, template 4, the delegation in `ceiling`, the
  `charter()` tools arm for 5, and `floors` (option below).
- `S/stock/coordinator.md` (new, a copy).
- `S/tests/stock.rs`, `S/tests/stock_brief.rs`, `S/tests/welcome_policy.rs`,
  `S/tests/fixtures.rs`, `S/tests/root_checks.rs`, `S/tests/startup.rs`:
  the new test and the counts and figures that follow.
- The fixtures that pin the stock's counts, as the cut's survey found
  them: `temper/crates/temper-playground/src/documents.rs` (run, templates,
  charters, executors) with its checked-in counterpart
  `temper/crates/temper-playground/stories/a/deployment.json`;
  `temper/crates/temper/tests/support/configuration.rs`;
  `temper/crates/temper/tests/suite/startup.rs` (the 5,822-byte message);
  `temper/crates/temper/tests/suite/creating.rs`
  (`the_forgejo_fixture_offers_every_stock_charter_but_the_trial`);
  `temper/crates/temper/src/tests/settings_fields.rs`
  (`every_member_ctl_prints_for_the_stock_is_a_field`, thirteen values);
  `temper/crates/temper/src/tests/reference_settings.rs`
  (`init_limits_hold_reference_delegation` alone: its variants keep the
  stock's `Job(3)` and `Charter(5)`);
  `temper/crates/temper-init/src/tests/documents.rs`
  (`the_box_has_room_for_reference_creating_executors`);
  `temper/tests/client-browser/src/configuration.rs` and `src/lib.rs`
  (edited and compiled only);
  `temper/tests/engine/domain/tests/suite/settings.rs`, `settings/b2.rs`
  and `configuring/fixture.rs` (capacities and their comments);
  `configuring.rs`' `a_proposal_selects_the_template_it_installs`, which
  takes template 4 as absent: give it a number the stock leaves free;
  `forge_routes/goal_branch.rs` (above) and `forge_routes/per_wake.rs`,
  whose fixture appends charter 5 and its delegation: it sets the
  reference's edition over the stock's charter 5 in place.
- `temper/docs/help/charter.md`, `temper/docs/help/settings.md`: the lines
  above.
- `temper/docs/design/client/journeys/*/recording`: regenerated with
  `temper-client-journeys again` if your change makes them stale (their
  settings inventory holds the stock's charters and templates).
- The status entry above.

Reader and neighbour files (edits allowed as stated):

- Any other test or fixture the compiler or a focused run shows pinning
  the stock's charter or template numbers, counts, executor kinds or the
  policy's bytes: that figure only, measured.
- `temper/crates/temper-docs/src/tests/*.rs`: only if a help line you
  change is pinned there; the pin follows the page.

Pre-approved options:

- **Reason names:** a `match` on the names in the stock module is the
  default, since `temper-service` does not depend on
  `temper-settings-form`, whose settings-file decoder has the same table;
  add no dependency for it.
- **The splitter:** a private function of the stock module, panicking with
  `expect` on a heading it doesn't know, is the default; the file is
  compiled in and the new test holds it.
- **`floors`' `core.run_bytes`:** jig's check counts a charter's policy
  with its parts (`jig/crates/jig-domain-core/src/settings/check.rs`,
  around line 568; `jig/crates/jig-domain-core/src/limits.rs`). If jig
  exposes that measure publicly, the stock's floor uses it, so that a
  start too small for charter 5 names `core.run_bytes` and charter 5; if
  it does not, leave the floor counting the policy alone and say so in
  the report.
- **The stock brief's rows:** charter 5's rows are added as the test
  measures them; if charter 5 becomes the largest stock charter's
  non-brief host part, the figures that follow it move with it and the
  report says so.

## Steps

1. Read `S/stock.rs` whole, `S/tests/stock.rs`' charter, delegation and
   floor tests, `S/tests/stock_brief.rs`, and jig's `CharterPart`
   (`jig/crates/jig-domain-core/src/settings/charters.rs`) and
   `tasks::Reasons` (`jig/crates/jig-domain-tasks/src/reasons.rs`).
2. `cp temper/docs/plan/ui-ways-rules/inputs/charter5.md
   temper/crates/temper-service/src/stock/coordinator.md`.
3. Sketch, to the contract:

   ```rust
   fn coordinator(models: &[CatalogueModel]) -> Charter {
       let (shared, parts) = sections(include_bytes!("stock/coordinator.md"));
       let mut value = charter(5, WORK_TURNS, WORK_TIME, &shared, models, FIRST_CHOICES);
       value.parts = parts;
       value.policy.resume = false;
       value.policy.shell = false;
       value.wake = COORDINATOR_WAKE;
       value
   }
   fn sections(text: &'static [u8]) -> (Box<[u8]>, Box<[CharterPart]>) { /* contract, section 3 */ }
   fn draft_goal(result_bytes: u32) -> Template { /* contract, section 3 */ }
   ```

   The charters' box gains `coordinator(&models)` after `reviewer`, the
   templates' `draft_goal(result_bytes)` between `goal` and `ask`, and
   `ceiling`'s kinds `Charter(5)` and `Job(3)` where the contract puts
   them.
4. Write the parts test; see it fail on the base (no charter 5), then
   build to green: `heavy cargo nextest run -p temper-service --lib -E
   'test(/stock::/)'` (check the count). Break the rule once by hand
   (drop the last part, or join two parts' `on` sets) and see the test
   fail for that reason; restore.
5. Move the fixtures and counts, package by package, each with its
   focused run: `-p temper-service --lib`; `-p temper-playground`;
   `-p temper --lib` and `--test suite -E 'test(/startup::/) |
   test(/creating::/)'`; `-p temper-init --lib`;
   `-p temper-engine-domain-world --test suite -E 'test(/settings::/) |
   test(/configuring::/) | test(/forge_routes::goal_branch::/) |
   test(/forge_routes::per_wake::/)'`.
6. If the replay of the journey recordings is stale, run
   `temper-client-journeys again` (common rules), then the focused replay
   it feeds.
7. Once, at the end: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-service -p temper-playground -p temper
   -p temper-init -p temper-engine-domain-world --all-targets -- -D
   warnings`; the focused tests of those packages and of
   `temper-client-world`; `heavy cargo check -p temper-client-browser-world --tests`; `python3 -I -B tools/citations/citations.py --base
   origin/main`. Commit (`ui-ways-rules: Draft first's coordinator and
   goal template join the stock`, the body saying why: a goal for a
   draft runs on any project, with no settings file of one project) and
   push.

## Report

As the implementer's text says: the first line, the commit pushed, the
`cmp` of the copy, each run with its count, each figure moved (old, new),
the rule broken by hand and what failed, the choices you made inside the
sketch and the options above, and the minutes. Name the delegation's
change in one line, for the owner of the goal's charters, who reviews it
before landing.
