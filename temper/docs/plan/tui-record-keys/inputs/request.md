# tui-record-keys: the TUI's recorder also takes Alt+m and Alt+s, for terminals that drop Shift from Meta

A small repair from the user's own recording on their laptop, 2026-11-08, in emacs vterm on Linux. The TUI's `record` build takes Alt+Shift+M as the mark key and Alt+Shift+S as the save key (`temper/docs/design/client/tui/journeys.md`, 2, 3.3 and 8.1). Emacs drops the Shift from a Meta+Shift key that has no binding of its own, so the keys arrived as `key alt "m" none` and `key alt "s" none`. Nothing was marked or saved until `q`, and the approval's frame was missing. This plan carries the priority mark.

## The change

- **The `record` build also takes Alt+m as the mark key and Alt+s as the save key,** beside Alt+Shift+M and Alt+Shift+S. Like those, they never reach the host, and are kept as their own records.
- **The shipped TUI is untouched:** the keys exist only in the `record` build, as today.
- **The walk's steps** (`journeys.md`, 8.1) keep pressing Alt+Shift+M and Alt+Shift+S. Recordings and prints don't change, so nothing is recorded again.
- **Design:** say so in `journeys.md` where the mark and save keys are named (2, 3.3, 8.1's table), and in its status if anything lags.
- **Tests:** a row each for Alt+m and Alt+s in the recorder's existing key test, beside the Shift forms, and a row showing that the shipped build passes Alt+m to the host as before.

## Size and overlaps

One task, direct. `tui-link-colours` is in flight beside it and may touch the recorder's start facts (the scheme): keep this change to the key match and its test rows, and name the overlap in `plan.md`.

## Walk

`walk = false`. The user records by hand on their laptop after landing.

## For every task of this plan

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, or a tier's CPU figure.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test (skein's `docs/foundation/testing-strategy.md`, 8). Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A small edit just outside the file list (a design line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
