+++
id = "project-stock"
part = "stock"
kind = "migration"
needs = ["forge-rules", "stock-roles"]
files = ["temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper/src/tests/help.rs", "temper/docs/help/configure.md", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/client/tests/suite/settings_tools.rs", "temper/docs/design/client/journeys/*/recording", "temper/docs/design/protocol/configuration.status.md", "temper/docs/design/protocol/setup.status.md"]
effort = "high"
minutes = 110
+++

# The binary gives every project its rule 1, and the deployment none

## Goal

`temper/docs/design/draft/stock-ways.md`, 3.1, in the binary: the stock
module gives the forge kind 2's rule 1 for every project, `(2, 0, 1)`,
in place of kind 1's rule 1, as `notes/contracts.md`, section 4 has it.
`forge-rules` built how the forge holds such a rule; this task makes the
binary give it, and moves every test and world that assumed the
deployment's stock rule. A deployment's owners still add rules of kind 1,
which every project's merges meet beside its own.

Acceptance:

- `stock::forge` gives the contract's three entries and no entry of kind
  1; `the_binary_gives_the_forges_stock_at_every_start`
  (`temper-service/src/tests/stock.rs`) holds them, the rule's encoded
  size measured.
- **The stock's rule 1 per project and its reset,** on the binary's stock:
  one new test in `temper/tests/engine/domain/tests/suite/configuring.rs`,
  on `World::stock()`, beside `the_asking_chat_reads_each_kind`: a run's
  `settings landing-rule 1 PROJECT` reads the stock's rule (`["branch"]`
  open, enforced, CI, up to date, gate 1) for project 1 and for a second
  project; the deployment's `settings landing-rule 1` reads no rule;
  project 1's rule 1 changed by `World::change` to one asking CI alone
  reads that rule for project 1 and the stock's for the other; reset, it
  reads the stock's again.
- The tests that read the deployment's stock rule read the project's
  instead, keeping what they check: in `configuring.rs`, `approval_items`
  and its callers (`a_box_person_known_by_the_forges_link_approves_with_nothing_added`
  among them), `the_asking_chat_reads_each_kind` and
  `an_asking_chat_reads_each_address`; in
  `temper/tests/client/tests/suite/settings_tools.rs`, the `KINDS` entry
  for `landing-rule 1` and `each_kind_through_ctl_proposed_shown_accepted_and_restored`.
- `temper/docs/help/configure.md`'s example that keeps the stock's landing
  requirements while it selects charter 2 sets project 1's rule 1 to the
  stock's value (`"project": 1`, the pattern `["6272616e6368"]` open), and
  `configure_example_preserves_stock_landing_gate`
  (`temper/crates/temper/src/tests/help.rs`) compares it with the stock's
  `(2, 0, 1)` entry. The page says "project 1's" where it said the stock's
  rule was kept, within its bound.
- The journey recordings regenerated if their settings inventory went
  stale (it lists the stock's defaults, and the deployment's rule 1 is no
  longer one), and their focused replay green.
- The two **Not yet** entries you close are deleted: in
  `temper/docs/design/protocol/configuration.status.md` the bullet starting
  `13.4: the stock module gives the forge kind 1's rule 1`, and in
  `temper/docs/design/protocol/setup.status.md` the bullet starting `8.3:
  each project's landing rule 1 is not the binary's stock`.
- Clippy clean, fmt clean, the focused tests below green.

## Inputs

- `temper/docs/plan/ui-ways-rules/notes/common.md` and `notes/contracts.md`,
  sections 2 and 4 (binding).
- `temper/docs/design/protocol/configuration.md`, 13.3 (*So a change of
  landing rules touches nothing of jig's*) and 13.4 (the rows *kind 1* and
  *kind 2, rule 1*; the bullets *A project's rule 1 is given to the
  forge* and *Built at each start*).
- `temper/docs/design/protocol/setup.md`, 8.3 (*Its projects*).
- `temper/docs/design/domain/agent.md`, 8.1 (the `settings landing-rule N
  [project]` read).

Departures from the implementer's text, and why:

- **You delete two status entries,** named above. Touch no other design
  text.
- **Help pages:** `temper/docs/help/configure.md` is yours, for its example
  and the sentence before it, since the stock's place moves.

## Owns and may change

Owns:

- `temper/crates/temper-service/src/stock.rs`: `fn forge` alone. Charter 5,
  template 4 and the delegation are `stock-roles`', merged before you.
- `temper/crates/temper-service/src/tests/stock.rs`:
  `the_binary_gives_the_forges_stock_at_every_start`.
- `temper/crates/temper/src/tests/help.rs` and `temper/docs/help/configure.md`:
  as above.
- `temper/tests/engine/domain/tests/suite/configuring.rs`: the new test and
  the three tests and helper above.
- `temper/tests/client/tests/suite/settings_tools.rs`: `KINDS` and the
  walk's test.
- `temper/docs/design/client/journeys/*/recording`: regenerated.
- The status entries above.

Reader and neighbour files (edits allowed as stated):

- Any other test, fixture or golden that a focused run shows reading the
  binary's forge stock (`stock::forge`, `Given::Stock`) as a deployment
  rule: the assertion moves to the project's rule 1, keeping what it
  checks; name each in the report. The cut's survey found
  `temper/crates/temper-playground/src/tests/stories.rs`'
  `story_as_forge_keeps_the_product_stock`, which compares the whole list
  and should need nothing, and `temper/crates/temper-service/src/tests/welcome_policy.rs`,
  whose forge-stock count stays 3.
- Worlds that give their own forge defaults (`Given::Own`, the domain
  world's `forge_routes` and `settings/b2.rs`, the client world's
  `front.rs`, the system and real-loop worlds) are left as they are: none
  of the binary's stock reaches them.
- `temper/tools/stage-one/**`: its fixtures are recorded ctl output, not the
  binary's stock; touch them only if a focused run of `temper-stage-one` fails for
  this change, and say why.

Pre-approved options:

- **The ctl walk's entry:** the default is
  `{"kind":"landing-rule","project":1,"number":1}`, shown with `--project
  1`. If restoring a setting jig had no default for cannot pass through
  the walk's restore step, keep a deployment rule in the walk by giving
  its world a kind 1 rule of its own in its fixture, say so, and keep the
  project's rule 1 as a row the walk shows and changes.
- **A second project in the new test:** made as the world's other tests
  make one, if the world has a helper; otherwise a project made through
  `MakeProject` as `configuring/fixture.rs` makes the first, and say so.
- **The asking reads' expected text:** if the see form's heading for a
  project's rule differs from the deployment's (`landing-rule 1 project
  1`, say), assert the heading it prints and say so.

## Steps

1. Read `stock.rs`' `fn forge`, `the_binary_gives_the_forges_stock_at_every_start`,
   `configuring.rs`' `approval_items` and the two asking-read tests,
   `settings_tools.rs`' `KINDS` and walk, `help.rs`'
   `configure_example_preserves_stock_landing_gate` and `configure.md`
   around its example.
2. Write the new test on the base: it fails there (the deployment holds
   the stock's rule, and a project's rule 1 reads none). Then change
   `fn forge` to the contract and see it pass:
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/configuring::/)'` (check the count). Break it once by hand
   (give the stock's entry scope 1, not 0) and see the second project's
   read fail; restore.
3. Move the readers above, package by package, each with its focused run:
   `-p temper-service --lib -E 'test(/stock::/)'`; `-p temper --lib -E
   'test(/help::/)'`; `-p temper-client-world --test suite -E
   'test(/settings_tools::/)'`; `-p temper-playground`.
4. If the journey replay is stale, run `temper-client-journeys again`
   (common rules), then its focused replay.
5. **A change of meaning runs its readers' tests:** every merge on the
   binary's stock now meets a project's rule, not the deployment's. Run
   the focused tests of `temper-service`, `temper`, `temper-playground`,
   `temper-engine-domain`, `temper-engine-domain-world`,
   `temper-client-world` and `temper-stage-one`.
6. Once, at the end: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-service -p temper -p
   temper-engine-domain-world -p temper-client-world --all-targets -- -D
   warnings`; `python3 -I -B tools/citations/citations.py --base
   origin/main`. Commit (`ui-ways-rules: the binary gives every project
   its rule 1, and the deployment none`, the body saying why: a project's
   way must be able to loosen the stock, and rules add) and push.

## Report

As the implementer's text says: the first line, the commit pushed, each
run with its count, each test moved and how, the rule broken by hand and
what failed, the options you took, and the minutes.

## Answers

2026-11-08 (temper/ui). The first: stock-roles owns the fix. It was answered at 05:12Z to accept a waiting of 0 in temper-engine-domain/src/limits.rs's run_policy_bytes, with a regression row. Once that lands on the trunk, merge it and resume here: your world test should then start, and your assertions stand as written. Don't widen this task to the validator.

## Rebase (2026-11-08, step merge, foreman ui-ways-rules/project-stock/foreman-1)

merge current integration, preserve both status edits, and regenerate conflicting journey and playground artifacts.
The merge failed on branch-owned conflicts; retrying unchanged will fail again.
Merge the current `origin/ui-ways-rules/integration` (observed tip `d311d03dbb`, including main catch-up `854d6bffb7`).
Artifact changes overlap stock-roles commit `940f27188e`; playground log changes also overlap main commit `b3e5a171a7`.
In `temper/docs/design/protocol/configuration.status.md`, retain trunk’s current text and remove the task’s 13.4 deployment-stock-rule gap.
Preserve trunk’s deletions of the stock-roles and reference-project gaps; do not restore them.
In `temper/docs/design/protocol/setup.status.md`, remove the task’s 8.3 project-stock gap while retaining trunk’s sign-in and approval updates.
Resolve `temper/docs/design/client/journeys/chat-draft/recording` by recording the combined code with `temper-client-journeys again chat-draft`.
Regenerate `chat-draft/01.html` through `20.html` with the journey tool; do not hand-merge generated HTML.
The regeneration must include both the project-scoped stock rule and trunk’s stock charter/template changes.
Regenerate `temper/tests/playground/tests/fixtures/story-a.log` through `walk::story_a_walks_from_the_welcome_to_the_csv_change_landed`, using `SKEIN_CODEGEN_WRITE=1` in `temper-playground-world`’s `suite` binary.
Then verify `walk::story_a_kept_log_replays_natively` without write mode.
Review the automatic merges in `configuring.rs`, `configuring/fixture.rs`, and `host_calls.rs`; retain the new stock/reset assertions, second-project capacity, decoded change kinds, and stock restart helper.
Run the focused configuring and journey replay checks and the spec’s affected focused checks through `heavy`; report regenerated files and selected test counts.

2026-11-08 (temper/ui). Neither: the fix is already on main. The CSP-blocked night images were red-74's, repaired on main at 06:24Z by post-landing-probes/red-74 (merge df5803a0ad). They replace the data: SVG moon, sun and grain with images the page policy allows. This trunk was cut before it. Take main into your branch by merging origin/main as the trunk's catch-up would, keeping both histories and hand-merging no generated file. Then run temper-client-journeys again chat-draft from the combined code, as generation, retried up to three times on a moving key mark, check its focused replay, commit and push. Change no stylesheet or page policy here.
