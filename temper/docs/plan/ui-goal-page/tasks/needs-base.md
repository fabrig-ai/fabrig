+++
id = "needs-base"
part = "needs-base"
kind = "migration"
needs = []
files = ["**"]
effort = "high"
minutes = 75
+++

# ui-needs' merged code on this trunk

## Goal

The goal's page builds on `ui-needs`: its parsed card (`temper-client-domain/src/question.rs`), its
`pages/ask.rs`, `pages/needs.rs`, its choices and its toast. This trunk was made from `ui-goals`' alone, and
neither has landed. Merge `origin/ui-needs/integration` into your branch, as its tip stands when you start,
so that every later task of this plan finds both. The plan's `after` keeps those commits from landing ahead
of `ui-needs`.

Departure from the implementer's text: this task is a merge of another plan's trunk, `files` is everything
the merge touches, and that is its whole diff. It changes no behaviour of either side.

Acceptance:

- `origin/ui-needs/integration`'s tip at your start is an ancestor of your branch, by a merge commit.
- No conflict marker is left; both plans' intent stands in every file (below).
- The tests under Steps pass: `heavy cargo nextest run` on the named packages, whole.
- No stale golden, print or recording: `heavy cargo run -p temper-client-journeys` exits 0.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`.
- `temper/docs/plan/ui-goals/plan.md`; `temper/docs/plan/ui-needs/plan.md` once merged: what each side meant.

## Owns and may change

Everything the merge touches. Don't merge `origin/main`: fab's catch-up does, later. Don't merge
`ui-goals/integration` again either: it reaches this trunk through main once it lands.

A trial merge at the cut's time conflicted in these files, one to three hunks each; yours may differ a
little, and these are the pre-approved resolutions:

- **Both sides added, keep both:** `temper-client-domain/src/pages.rs`, `src/tests/fixtures.rs`;
  `temper-client-view/src/look.rs`, `src/words.rs`, `src/tests/rail.rs`; `temper/tools/gallery/src/samples.rs`.
  Where both drew the rail, `ui-goals`' Goals entry and `ui-needs`' Needs entry both stand.
- **The stock's words, `temper-service/src/stock/chat.txt`:** `ui-goals`' report paragraph and `ui-needs`'
  sentence on a question's form, both.
- **The figures, `temper-service/src/limits.rs`, `src/tests/limits.rs`, `src/tests/stock_brief.rs`:** whatever
  the combined text measures. A limit that both raised takes the combined measure, as the tests say; say the
  figure in your report. This is a fixture's measure of a stock's size, not a budget.
- **`temper/crates/temper/tests/fixtures/cluster/engine.json`, `tests/support/configuration.rs`,
  `tests/support/github/engine.json`, `temper/tests/client-browser/src/configuration.rs`,
  `temper/tests/client/src/fixtures.rs`, `temper/tests/real-loop/src/fixtures.rs`:** the larger of what each
  side raised, or the combined measure where the fixture measures the stock.
- **Design, `temper/docs/design/client/README.md`, `browser.status.md`, `protocol/configuration.md`,
  `protocol/configuration.status.md`:** both. This branch may edit them for the merge alone. Keep each
  section's number; `python3 -I -B tools/citations/citations.py --base origin/main` reports no new broken one.
- **Generated, regenerated and never hand-merged:**
  - `temper/tests/playground/tests/fixtures/story-a.log`: `TEMPER_BUNDLE=none SKEIN_CODEGEN_WRITE=1 heavy cargo
    nextest run -p temper-playground-world --test suite -E
    'test(=walk::story_a_walks_from_the_welcome_to_the_csv_change_landed)'`, then the replay,
    `test(=walk::story_a_kept_log_replays_natively)`, without the variable. Take the test's full name from
    `temper/tests/playground/tests/suite/walk.rs` if it differs.
  - `temper/docs/design/client/journeys/welcome-playground/recording`: `heavy cargo run -p
    temper-client-journeys -- again welcome-playground`, then `heavy cargo run -p temper-client-journeys`.
    Any other journey it names as stale, the same way.
  - `temper/scripts/old-forms.txt`: `old_forms.py --write` when the combined count is lower or equal to the
    higher side; a raised count against both sides is a question of which side's code, and is reported.

Options, each pre-approved:

- If `ui-needs/integration` moved since the cut, take its tip when you start and stay on it.
- If a test the merge didn't touch fails on one side alone, before the merge (run it there once), list it under
  `## Left` with the file and which plan's it is.
- If two sides name one thing differently in a way that's more than a merge (a rule's value both changed), take
  `ui-needs`' on its Needs page and `ui-goals`' on its Goals page, and say so.

## Steps

1. `git fetch origin`, `git merge --no-ff origin/ui-needs/integration`. Resolve as above.
2. `cargo fmt`. `heavy cargo check --workspace --tests` once, while errors last, since both sides moved types.
3. Regenerate what is generated.
4. `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p
   temper-client-shell-terminal -p temper-client-tools -p temper-service -p temper-client-gallery -p
   temper-client-journeys -p temper-engine-domain`; the two playground tests above.
5. The check of the implementer's text (old forms, clippy on the packages you edited by hand, fmt). Commit the
   merge, then any regeneration as its own commit, and push.

## Report

As `notes/common.md`, Your report: each file resolved other than by keeping both, each figure measured, and
what each regeneration moved.

## Rebase (2026-11-05, step merge, foreman ui-goal-page/needs-base/foreman-1)

resolve the three design conflicts by retaining Needs changes alongside the newer goal-page contract.
The merge conflicts with trunk commit `64b1133e5fb99d62ca544a90dedbcad5b59e5e98` (“ui-goal-page: the design and the rest of the chain”), incorporated by `f90bc9abbe`.
Resolve these paths under `temper/docs/design/client/`:

- `README.status.md`: keep the trunk’s separate `ui-goal-page` Not yet entry and the branch’s Needs/Home gaps; retain the remaining status text from both sides.
- `browser.status.md`: keep the trunk’s `ui-goal-page` gaps for task-address filtering, A goal’s look and gallery states, alongside the branch’s Needs/Home gallery gaps.
- `browser.md`, 2.3: retain “its unfiltered address undoes” from Needs, but remove “and a goal’s plan’s” from the query inventory as the trunk requires. Preserve the trunk’s chats-only filter paragraph and `/tasks/9?state=live` normalization contract.
- `browser.md`, 7.2: retain the branch’s `inbox-solo` Needs description and every `needs-*` row; add all trunk `goal-*` and `start-goal*` rows. Keep one `task-solo-T3` row, using the trunk’s `goal.html` description.

Preserve the automatic merges, including `README.md`, the goal stylesheet contract and the other updated gallery descriptions. Keep existing section numbers.
These are design conflict resolutions; implementation gaps remain recorded for later tasks.
Check for remaining conflict markers and run `python3 -I -B tools/citations/citations.py --base origin/main`, then complete the spec’s validation.
