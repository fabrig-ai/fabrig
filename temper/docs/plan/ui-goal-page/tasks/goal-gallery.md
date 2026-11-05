+++
id = "goal-gallery"
part = "goal-gallery"
kind = "feature"
needs = ["goal-route"]
files = [
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/src/lib.rs",
]
effort = "high"
minutes = 90
+++

# A goal's page in the gallery

## Goal

The gallery holds the goal's page, every read-only state, in `data.js`' words (first-run-projects, welcome-stall, forge-cards,
pane-chat-line), and its old samples move.

Acceptance:

- States, as `temper/docs/design/client/browser.md`, 7.2, words them: `goal-running`, `goal-needs-funds`, `goal-needs-question`,
  `goal-planning`, `goal-paused`, `goal-landed`, `goal-landed-open`, `goal-stopped`, `goal-failed`, `goal-no-report`. `names.txt`
  sorted as it is, `tests.rs` holding each one's headline or its telling word.
- `task-solo-T3` and `task-reference-T12` are goal's pages; `task-held` and `task-releasing` an agent's task's plan with a held
  task, as 7.2 says; `task-reference-T28` is unmoved.
- `heavy cargo nextest run -p temper-client-gallery` passes whole; `tools/gallery`'s print of the new states holds no error or empty page.
- `temper/docs/design/client/browser.status.md`'s 7.2 line loses these ten states.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/browser.md`, 7.2.
- The mock's `data.js`, the goals at lines 84 to 150, the needs at 35 to 66.
- `temper/tools/gallery/src/samples.rs` (`Then`), `worlds.rs` (`facts`, `document`, `plan_tree`, `shown_task`, its report notes and goals
  deliveries from `ui-goals`); an open fold's patch in `lib.rs`, as `needs-saw-open`.

## Owns and may change

Owns, as `files` lists them.

Options, each pre-approved:

- **Numbers:** tasks as the Solo and Reference worlds number them; a new goal takes the next free number in its world, and its delegates the
  next after it. Keep other samples' deliveries as they are.
- **Delivering the page's reads:** the report's notes, its own line (`own_line_query`) and the inbox filtered by goal, as `goals-needs`'
  deliveries do. If a page of `Delivered` is short, add to the world's lookup by query.
- **`goal-landed-open`:** the fold opened as `needs-saw-open`'s patch, or by its `Then`, whichever the gallery has.
- **A delegate's dependency words in `goal-running`:** `data.js` has "waits on the first two": that's a phrase the page doesn't say. Draw
  "waits on N tasks" and keep a dependency of one for "waits on TITLE".
- Budget words stand as the mock's numbers, without `$`.

## Steps

1. The worlds' deliveries, one state at a time; re-point the old four.
2. `heavy cargo nextest run -p temper-client-gallery`. Print the states you added and read two.
3. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report: the worlds' numbers you took.
