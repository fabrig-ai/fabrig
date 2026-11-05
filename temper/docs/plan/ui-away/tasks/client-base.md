+++
id = "client-base"
part = "client-base"
kind = "migration"
needs = ["home-base"]
files = ["**"]
effort = "high"
minutes = 60
+++

# `ui-settings`' and `ui-goal-page`'s code on this trunk

## Goal

The switch, Home's form and Settings' section build on `ui-settings`' *How work runs* and `Ways.away`, and on
`ui-goal-page`'s toast kinds and Start goal sheet. Bring both to this trunk before any client task of this plan writes
the files they share. The plan's `after` keeps those commits from landing ahead of them.

Departure from the implementer's text: this task is a merge of other plans' trunks, `files` is everything the merges
touch, and that is its whole diff. It changes no behaviour of any side.

Acceptance:

- The tips of `origin/ui-settings/integration` and `origin/ui-goal-page/integration` at your start are ancestors of
  your branch, each by a merge commit.
- No conflict marker is left, and every side's intent stands in every file.
- The tests under Steps pass, and `heavy cargo run -p temper-client-journeys` exits 0.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`.
- `temper/docs/plan/ui-settings/plan.md`, `temper/docs/plan/ui-goal-page/plan.md`: what each side meant.

## Owns and may change

Everything the merges touch. A trial at the cut's time: `ui-goal-page` against `ui-home-status` conflicted in
`temper-client-view/src/look.rs`, `src/words.rs`, `temper/docs/design/client/README.status.md`,
`temper/tests/playground/tests/fixtures/story-a.log`, `temper/tools/gallery/src/samples.rs`, `src/worlds.rs`; `ui-settings`
against it in those, `temper-client-domain/src/pages.rs`, `src/pages/home.rs`, `src/tests/pages/home.rs`, the view's
`src/tests/home.rs`, `temper/tests/client/src/front.rs`, `temper/docs/design/client/journeys/project-landing/*`,
`temper/docs/design/protocol/configuration.status.md`, `temper/docs/help/settings.md`, `temper/tools/gallery/names.txt`, and
the generated `v1.rs` and `v11.rs`. Yours may differ. The pre-approved resolutions:

- **Both sides added, keep both.** Where both drew Home, `ui-home-status`' intro and foot and `ui-settings`' Settings
  links both stand.
- **Generated, regenerated and never hand-merged:** as `tasks/home-base.md`, *Owns and may change*; a journey named
  stale, `heavy cargo run -p temper-client-journeys -- again NAME`.
- **Design and status:** both, with each section's number.

Options, each pre-approved:

- **A plan that has landed** (`git merge-base --is-ancestor origin/PLAN/integration origin/main`): merge `origin/main` in
  its place, once, for both.
- **If `origin/ui-away/integration` moves while you work,** merge it into your branch before you push.
- If a test the merges didn't touch fails on one side alone, before the merge (run it there once), list it under
  `## Left` with the file and which plan's it is.
- If two sides name one thing differently in a way that's more than a merge, take `ui-settings`' in Settings,
  `ui-goal-page`'s on a goal's page and `ui-home-status`' on Home, and say so.

## Steps

1. `git fetch origin`, then `ui-settings`' merge, then `ui-goal-page`'s, resolving each as above.
2. `cargo fmt`. `heavy cargo check --workspace --tests` once, while errors last.
3. Regenerate what is generated.
4. `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p
   temper-client-shell-terminal -p temper-client-gallery -p temper-client-journeys -p temper-engine-domain -p
   temper-service -p temper-protocol-store -p temper-client-protocol-wire`.
5. The check of the implementer's text; `python3 -I -B tools/citations/citations.py --base origin/main`. Commit each merge,
   then any regeneration as its own commit, and push.

## Report

As `notes/common.md`, Your report: each file resolved other than by keeping both, and what each regeneration moved.
