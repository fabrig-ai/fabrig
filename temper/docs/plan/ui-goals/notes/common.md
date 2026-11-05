# ui-goals: the rules every task names

The request is `temper/docs/plan/ui-goals/inputs/request.md`; the design this plan cut is
`temper/docs/design/client/README.md`, 3.5 (*Goals*, *A goal's report*) and 5.3,
`temper/docs/design/client/browser.md`, 2.3 and 2.5, `temper/docs/design/domain/agent.md`, 8.3, and
`temper/docs/design/protocol/configuration.md`, 13.4. Read this note, then
`temper/docs/plan/ui-goals/notes/contracts.md`, then your spec. Where this note and the implementer's
text differ, this note wins.

## Decide, don't ask

The user is asleep and takes the plan's judgement. There is no supported deployment: a stored format,
a protocol or a public interface may change freely, with no compatibility and no `Question:`. A small
edit your task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is
made and named in your report, not parked. A `Question:` is only for what truly blocks the task.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit the `docs/design/` status lines its spec
  names, as it closes them, and a design line a small edit needs; say which in your report. A draft's
  own text stays as it is.
- **`docs/help/` and generated files:** `docs/help/reference/*`, prints, goldens and counted baselines
  your change makes stale are yours to regenerate in the same task. Don't park on them.
- **A throwaway render script** is allowed in any task, untracked and gone before the push.

## Reach for the mock

The target is `temper/docs/design/client/greenfield/` (`goals.html`, drawn by `goalsPage` and `goalRow`
in `app.js`; `mock.css`; the words in `data.js`). Open its pages from the checkout; nothing needs
extracting. Reach for it, not the old page. Where old tests walk a part the mock removes, change them
with it.

## Things that conflict across plans

Other plans (`ui-needs`, `ui-changes`, `look-round-5`, `ui-settings`, `ui-home-status`) run beside this
one and touch the same files.

- **`temper.css`:** new rules go under this plan's marker comment, `/* ==== ui: goals ==== */`, never
  at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`:** on a catch-up, regenerate rather
  than hand-merge, and keep `names.txt` sorted as it is.
- **The names others build on:** `goals::row`, `goals::ways`, `goals::state`, `Domain::has_goals` and
  `report::parse` keep those names (`notes/contracts.md`).

## Not this plan's

- `PublicBoard` (signed out) is another thing: leave it.
- No goal page: plan `ui-goal-page` rebuilds it, and adds the page's "Start one from a draft" link.
- No spend on this page, no Home "Goals" section (the Home area's plan), no switch for goals
  (`ui-settings`' kind 7), and no stored state for the filter: it lives in the address.

## Render proofs

Chromium 147's new headless won't go below 500 px by `--window-size`: a 420 px proof uses CDP's
`Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's
`onbeforeunload` hangs a CDP driver. The mock's Google Fonts link needs the network; the client's own
fonts are what's compared. Screenshots go to `~/src/fabrig/ui-goals/` on your builder, outside any
worktree, never committed; your report names the builder.

## Your report

As the implementer's text, with: the status lines you closed; each small edit outside your file list;
the options you took.
