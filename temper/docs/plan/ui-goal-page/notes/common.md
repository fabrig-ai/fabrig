# ui-goal-page: the rules every task names

The request is `temper/docs/plan/ui-goal-page/inputs/request.md`. Read this note, then
`temper/docs/plan/ui-goal-page/notes/contracts.md`, then your spec. Where this note and the implementer's
text differ, this note wins.

## Decide, don't ask

The user is asleep and takes the plan's judgement. There is no supported deployment: a stored format, a
protocol or a public interface may change freely, with no compatibility and no `Question:`. A small edit
your task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and
named in your report, not parked. A `Question:` is only for what truly blocks the task.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit the `docs/design/` status lines its spec names,
  as it closes them, and a design line a small edit needs; say which in your report. A draft's own text
  stays as it is.
- **`docs/help/` and generated files:** `docs/help/reference/*`, prints, goldens and counted baselines your
  change makes stale are yours to regenerate in the same task, by their own tools. Don't park on them.
- **A throwaway render script** is allowed in any task, untracked and gone before the push.

## Reach for the mock

The target is `temper/docs/design/client/greenfield/` (`goal.html`, drawn from `data.js` by `app.js`;
`mock.css`). Open its pages from the checkout; nothing needs extracting. Reach for it, not the old page.
Where old tests walk a part the mock removes, change them with it.

## Things that conflict across plans

Other plans (`ui-chats`, `ui-settings`, `ui-welcome`, `ui-changes`, `ui-home-status`) run beside this one
and touch the same files.

- **`temper.css`:** new rules go under the marker comment `/* ==== ui: goals ==== */`, never at the end of
  the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`:** on a catch-up, regenerate rather than
  hand-merge, and keep `names.txt` sorted as it is.

## Render proofs

Chromium 147's new headless won't go below 500 px by `--window-size`: a 420 px proof uses CDP's
`Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's
`onbeforeunload` hangs a CDP driver. The mock's Google Fonts link needs the network; the client's own fonts
are what's compared. Screenshots go to `~/src/fabrig/ui-goal-page/` on your builder, outside any worktree,
never committed; your report names the builder.

## Your report

As the implementer's text, with: the status lines you closed; each small edit outside your file list; the
options you took.
