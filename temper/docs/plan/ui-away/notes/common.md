# ui-away: the rules every task names

The request is `temper/docs/plan/ui-away/inputs/request.md`. Read this note, then
`temper/docs/plan/ui-away/notes/contracts.md` once it exists, then your spec. Where this note and the
implementer's text differ, this note wins.

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
  change makes stale are yours to regenerate in the same task, by their own tools. Don't park on them. A
  help page your change grows past its bound is yours to shorten, losing no fact, and the bound never rises.
- **A throwaway render script** is allowed in any task, untracked and gone before the push.

## Reach for the mock

The target is `temper/docs/design/client/greenfield/` (`home.html`, `settings.html` and `phone.html`, drawn
from `data.js` by `app.js`; `mock.css`). Open its pages from the checkout; nothing needs extracting. Reach for
it, not the old page. Where old tests walk a part the mock removes, change them with it.

## Things that conflict across plans

Other plans (`ui-home-status`, `ui-settings`, `ui-goal-page`, `ui-chat-draft`) run beside this one and touch
the same files.

- **`temper.css`:** new rules go under the marker comment `/* ==== ui: away ==== */`, never at the end of the
  file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`:** on a catch-up, regenerate rather than
  hand-merge, and keep `names.txt` sorted as it is.
- **The store's bytes:** a family byte may move if another plan lands first. Take the next free one and say so
  in your report; the design's table moves with it.
- **Recordings:** a wire change makes journey recordings stale: re-record them in that task. A re-record that
  stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all
  fail, widen the mark's wait in the recording driver and say so.

## Render proofs

Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's
`Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's
`onbeforeunload` hangs a CDP driver. The mock's Google Fonts link needs the network; the client's own fonts are
what's compared.

## Your report

As the implementer's text. Beside it, each small edit outside your `files`, and each option you took.
