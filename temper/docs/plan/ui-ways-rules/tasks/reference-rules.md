+++
id = "reference-rules"
part = "reference"
kind = "feature"
needs = ["forge-rules", "stock-roles"]
files = ["temper/docs/examples/reference/project.json", "temper/docs/examples/reference/README.md", "temper/crates/temper/src/tests/reference_settings.rs", "temper/crates/temper-docs/src/tests/rules.rs", "temper/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 45
+++

# The reference's project sets its own rule 1

## Goal

`temper/docs/design/protocol/configuration.md`, 13.9 (*The landing
rules*): every project now holds the stock's rule 1 over every branch,
whose CI and agent's review would hold every merge into the reference's
goal branches. So the reference's `project.json` sets its project's rule
1 to main's rule, naming a branch, as `notes/contracts.md`, section 5 has
it. `settings.json` keeps its own editions over the stock and its two
deployment rules (`temper/docs/design/draft/stock-ways.md`, 3.3), and its
`README.md` says what the stock now holds of the delegation it widens.

Acceptance:

- `project.json` holds two items, its kind 7 item as it stands and the
  contract's rule 1, in that order.
- `the_reference_project_file_shows_its_goals`
  (`temper/crates/temper/src/tests/reference_settings.rs`) holds two
  items; the second decodes (`temper_engine_smith::kinds::decode(2, …)`)
  to `["branch"]`, exact `main`, enforced, CI, up to date, no gates and no
  approvals, of scope 1, number 1; and the forge's
  `temper_engine_domain_forge::settings::judge` admits it as kind 2 and
  refuses the same value as kind 1 with `Refusal::Pattern`. Rename the
  test `the_reference_project_file_shows_its_goals_and_lands_main_alone`.
- `the_reference_settings_file_decodes` and
  `the_reference_sets_main_and_goal_branch_landing_rules` hold as they
  are: `settings.json` is unchanged.
- `README.md`: one paragraph says that `project.json` sets the case's
  project's ways and its rule 1, main's, in place of the stock's rule 1
  over every branch, its project number set as the project-policy's is,
  with the command; and the paragraph starting `Expect: widened rules`
  says that the stock's delegation already holds coordinator charter 5,
  foreman charter 6, change procedure 2, merge job 3 and person role 0,
  and that the widening adds designer charter 7, keeping each kind once.
  The phrases `the_reference_readme_grants_its_tools_and_note_scopes`
  (`temper/crates/temper-docs/src/tests/rules.rs`) pins stay in the
  README.
- The **Not yet** entry you close is deleted: in
  `temper/docs/design/protocol/configuration.status.md` the bullet starting
  `13.9: \`docs/examples/reference/project.json\` holds the project's kind
  7 alone`. The entry after it, on stage one's golden, stays.
- Clippy clean, fmt clean, the focused tests below green.

## Inputs

- `temper/docs/plan/ui-ways-rules/notes/common.md` and `notes/contracts.md`,
  sections 1 and 5 (binding).
- `temper/docs/design/protocol/configuration.md`, 13.9: *A settings file*,
  *The rules and policy 1*, *The landing rules* and *The profile, whole*
  (its `project.json` point).

Departures from the implementer's text, and why:

- **You delete one status entry,** named above. Touch no other design text.
- **`temper/docs/examples/reference/README.md` is read by tests**
  (`.config/gate/read-by-tests.txt`); its tests are yours to run.

## Owns and may change

Owns:

- `temper/docs/examples/reference/project.json` and `README.md`.
- `temper/crates/temper/src/tests/reference_settings.rs`: the project
  file's test. `init_limits_hold_reference_delegation` is
  `stock-roles`', merged before you; leave it.
- `temper/crates/temper-docs/src/tests/rules.rs`: a pinned README phrase,
  only if one you must change is pinned.
- The status entry above.

Reader and neighbour files (edits allowed as stated):

- Any other test that decodes `project.json` and counts its items, found
  with `rg 'reference/project.json'`: the count and the new item's
  assertions only. The cut's survey found only the one above.

Pre-approved options:

- **The README's command:** `jq --argjson project PROJECT
  '.items[].setting.project = $project' project.json > FILE`, beside the
  project-policy's, is the default; if the README's form of commands
  differs, follow it.

## Steps

1. Read `project.json`, `settings.json`'s two `landing-rule` items,
   `reference_settings.rs`' project file test and the README.
2. Add the item; extend the test; run `heavy cargo nextest run -p temper
   --lib -E 'test(/reference_settings::/)'` (check the count). Break it
   once by hand (the item's first segment `forge`) and see the judge row
   refuse it; restore.
3. Edit the README; run `heavy cargo nextest run -p temper-docs`.
4. Once, at the end: `cargo fmt`; `heavy cargo clippy -p temper -p
   temper-docs --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`. Commit
   (`ui-ways-rules: the reference's project sets its own rule 1`, the
   body saying why: the stock's rule over every branch would hold its
   goals' branches on CI and a review) and push.

## Report

As the implementer's text says: the first line, the commit pushed, each
run with its count, the rule broken by hand and what failed, and the
minutes.

## Rebase (2026-11-08, step merge, foreman ui-ways-rules/reference-rules/foreman-1)

resolve the status conflict by preserving both tasks’ adjacent deletions
The merge conflicts only in `temper/docs/design/protocol/configuration.status.md`.
Commit `6837c2d7ed` deletes the 13.9 gap beginning “`docs/examples/reference/project.json` holds the project's kind 7 alone”.
Integration commit `940f27188e` deletes the adjacent 13.4 gap beginning “charter 5 and template 4 are not in the stock”.
Resolve with both bullets absent. Keep the following 13.9 stage-one golden gap unchanged.
Preserve integration’s other status updates, including the main changes incorporated by `aae6d4c8a2`; do not replace the whole file with the task’s version.
Keep the reference project, README and test changes. The log reports automatic merges for both Rust test files, with no conflict there.
