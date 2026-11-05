# ui-needs: the rules every task names

Every task of this plan brings part of `temper/docs/design/client/greenfield/` into temper's
client: Needs you one card at a time, and answering on Home. Read this note, then
`temper/docs/plan/ui-needs/notes/contracts.md`, then your spec. Where this note and the
implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit only the `docs/design/` files its
  `Owns and may change` lists: its status entries as it closes them, or what its spec names. A
  draft's own text stays as it is.
- **Generated files** your change makes stale (`docs/help/reference/*`, prints, goldens, counted
  baselines) are yours to regenerate in the same task; don't park on them.
- **A small edit just outside your files** (a re-export, a `mod` line, a design line, a golden) is
  made and named in the report, not parked.
- **Throwaway render scripts** are allowed in any task, untracked and gone before the push.

## Decide, don't ask

The user is asleep and takes this plan's judgement. A stored format, a protocol or a public
interface may change freely, though none is expected here: no compatibility, no `Question:`; say so
in your report. A `Question:` is only for what truly blocks the task.

## The mock is the reference

- `temper/docs/design/client/greenfield/` (`app.js`, `mock.css`); where the old page differs, the
  mock wins, and where old tests walk a part the mock removes, change them with it.
- Its values go into `temper.css` under the client's selectors, with the `--gf-*` tokens there.
- Budgets matter least: no "Add $10", funds use the held acts.

## `temper.css`

New rules go under this plan's marker comment, `/* ==== ui: needs ==== */`, never at the end of the
file, so that parallel plans merge cleanly. Replaced rules are removed where they stand.

## Goldens, prints and the gallery

`temper/tools/gallery/names.txt`, `temper/tools/gallery/src/tests.rs`, the terminal's golden pages
and the journeys' pages conflict across plans. On a catch-up, regenerate rather than hand-merge, and
keep `names.txt` in the order it has.

## Render proofs

Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof sets device
metrics through CDP's `Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`,
or a pending request's `onbeforeunload` hangs a CDP driver. Screenshots go to
`~/src/fabrig/ui-needs/` on your builder, outside any worktree, and are never committed; your report
names the builder. The mock's Google Fonts link needs the network: the client's own fonts are what's
compared. No `Look-request:` trailers.

## Your checks

Besides the implementer's text: a task that touched design text runs
`python3 -I -B tools/citations/citations.py --base origin/main`. A change of a page's markup
reprints: `heavy cargo run -p temper-client-journeys` reprints every journey in place, and the task
commits the pages it changed.
