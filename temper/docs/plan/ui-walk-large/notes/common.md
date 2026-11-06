# ui-walk-large: what every task holds

Every task names this note. The request's own list, `inputs/request.md`, *For every task of this plan*, holds whole; read it first. Its points, with what this plan adds:

- **Decide, don't ask.** Nobody is awake to answer. A stored format, a protocol or a public interface may change freely. A small edit just outside your list (a re-export, a `mod` line, a design line, a golden) is made and named in your report. `Question:` only for what truly blocks the task.
- **No budget or timeout moves.** The stock's period is in scope, and only `stock-room` changes it. No test budget, tier budget or timeout is. Over budget is red: `skein/docs/foundation/testing-strategy.md`, 8.1.
- **Generated files** your change makes stale are yours to regenerate by their own tool, in your task, except where your task says another task regenerates them. On a catch-up, regenerate goldens, prints and the gallery's lists rather than hand-merge.
- **`temper.css`:** new rules under this plan's marker comment, `/* ==== ui: AREA ==== */`, never at the file's end; replaced rules go where they stand.
- **Recording flakes:** a re-record stopped on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.
- **Render and walk proofs:** a throwaway script, untracked and gone before the push. Chromium 147's new headless won't go below 500 px by `--window-size`: a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **The look comes from** `temper/docs/design/client/greenfield/`, never the old page.
- **Help pages:** one your change grows past its bound is yours to shorten, losing no fact; never raise a bound.

## Departures from the implementer's text

- **The design is on the trunk already,** written with this plan; you don't write it. Your task names the status lines (`*.status.md`) its work closes: remove those lines, and nothing else under `docs/design/`, as the last edit before you push. Where the code taught something the design doesn't say, it goes in your report.
- **`docs/help/`** stays closed but for the lines your task names.

## Fakes and worlds

- The fake box is `temper box --fake-backends`. Its model is the document `temper-fake/src/backends.rs` writes (`model_document`), whose `cued` entries answer a person's exact words with scripted turns, tool calls among them.
- The worlds are `temper/docs/design/domain/testing.md`'s. A test of the engine's rule goes in the lowest tier that shows it: jig's world for jig's rule, temper's for what temper makes of it.

## Your report

As the implementer's text has it, and:

- the rule each new test pins, how you broke it and the failure you saw;
- each edit outside your `Owns` list;
- under `## Left`, what stays stale or red and who takes it, with the file.
