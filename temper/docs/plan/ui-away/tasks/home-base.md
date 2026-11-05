+++
id = "home-base"
part = "home-base"
kind = "migration"
needs = []
files = ["**"]
effort = "high"
minutes = 60
+++

# `ui-home-status`' code on this trunk

## Goal

This plan builds on `ui-home-status`: `ProjectFacts.landed`, the store's bytes 36 and 37, Home's one intro
(`intro_words`) and its Goals section. It is in the merge queue and has not landed as this is cut. Bring it to
this trunk, so that every later task of the plan finds it. The plan's `after` keeps those commits from landing
ahead of `ui-home-status`.

Departure from the implementer's text: this task is a merge of another plan's trunk, `files` is everything the
merge touches, and that is its whole diff. It changes no behaviour of either side.

Acceptance:

- The tip of `origin/ui-home-status/integration` at your start is an ancestor of your branch, by a merge commit.
- No conflict marker is left, and both sides' intent stands in every file (below).
- The tests under Steps pass.
- No stale golden, print or recording: `heavy cargo run -p temper-client-journeys` exits 0.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`.
- `temper/docs/plan/ui-home-status/plan.md`, once merged: what that side meant.

## Owns and may change

Everything the merge touches. Merge `origin/main` only as Steps says: fab's catch-up does the rest.

A trial merge at the cut's time, of `ui-home-status`' tip `4972cad36a` onto main, conflicted in these files.
Yours may differ a little, and these are the pre-approved resolutions:

- **Both sides added, keep both:** `temper/crates/temper-client-view/src/look.rs`, `src/words.rs`;
  `temper/tools/gallery/src/samples.rs`, `src/worlds.rs`; `temper/docs/design/protocol/configuration.status.md`.
- **`temper/docs/help/settings.md`:** both sides' facts, within its bound; shorten rather than raise it.
- **`temper/tools/gallery/names.txt`:** both, sorted as it is.
- **Generated, regenerated and never hand-merged:**
  - `temper/crates/temper-client-protocol-wire/src/generated/v1.rs` and its `golden/**`:
    `SKEIN_CODEGEN_WRITE=1 TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-protocol-wire`, then the
    same without the variable.
  - `temper/crates/temper-protocol-store/src/generated/v11.rs` and its `golden/**`: the same, with
    `-p temper-protocol-store`.
  - `temper/tests/playground/tests/fixtures/story-a.log`: `TEMPER_BUNDLE=none SKEIN_CODEGEN_WRITE=1 heavy cargo
    nextest run -p temper-playground-world --test suite -E
    'test(=walk::story_a_walks_from_the_welcome_to_the_csv_change_landed)'`, then the replay,
    `test(=walk::story_a_kept_log_replays_natively)`, without the variable. Take the tests' full names from
    `temper/tests/playground/tests/suite/walk.rs` if they differ.
  - A journey `temper-client-journeys` names as stale: `heavy cargo run -p temper-client-journeys -- again NAME`,
    then `heavy cargo run -p temper-client-journeys`.
  - `temper/scripts/old-forms.txt`: `old_forms.py --write` when the combined count is lower or equal to the
    higher side; a raised count against both sides is reported.

Options, each pre-approved:

- **If `ui-home-status` has landed when you start** (`git merge-base --is-ancestor
  origin/ui-home-status/integration origin/main`), merge `origin/main` instead: the acceptance's ancestor holds
  through it.
- If `ui-home-status/integration` moved since the cut, take its tip when you start and stay on it.
- If `origin/ui-away/integration` moves while you work, the cut has released more of this plan: merge it into
  your branch before you push. Its design text and yours keep both, with each section's number.
- If a test the merge didn't touch fails on one side alone, before the merge (run it there once), list it under
  `## Left` with the file and which plan's it is.
- If two sides name one thing differently in a way that's more than a merge (a rule's value both changed), take
  `ui-home-status`' on Home and main's on Settings, and say so.

## Steps

1. `git fetch origin`, then the merge, as above. Resolve as above.
2. `cargo fmt`. `heavy cargo check --workspace --tests` once, while errors last, since both sides moved types.
3. Regenerate what is generated.
4. `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p
   temper-client-shell-terminal -p temper-client-gallery -p temper-client-journeys -p temper-engine-domain-forge
   -p temper-engine-domain -p temper-service -p temper-protocol-store -p temper-client-protocol-wire`; the two
   playground tests above.
5. The check of the implementer's text (old forms, clippy on the packages you edited by hand, fmt);
   `python3 -I -B tools/citations/citations.py --base origin/main`. Commit the merge, then any regeneration as
   its own commit, and push.

## Report

As `notes/common.md`, Your report: each file resolved other than by keeping both, each figure measured, and what
each regeneration moved.
