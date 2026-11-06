# ui-public-2: the rules every task names

The request is `temper/docs/plan/ui-public-2/inputs/request.md`: read it, it is short. `temper/docs/plan/ui-public-2/notes/contracts.md` holds the shapes and words this plan binds.

## Departures from the implementer's text

- **Status lines:** the design for this plan is written, in the sections your spec cites. You change no design text, but your task removes the **Not yet** line your spec names from its status file, in the same branch.
- **Decide, don't ask (the user, 11-06).** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility and no `Question:`. A small edit your task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in your report, not parked. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If the base passes and your branch fails, it's yours.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in your report.

## Only focused and fuzzy run (the user, 11-06)

Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A story in those tiers is written and compiled (`heavy cargo check -p PKG --tests`), not run; say so in your report. This overrides anything else here. A look's render check by a throwaway driver isn't a tier.

## Not this plan

- **Titles:** what a title is, whether it crosses and the words about it are with the user (`jig/docs/design/domain/people.md`, 13.3). Change none of them, and leave `title`, `title_cut` and `PublicTitleCut` as they are.
- **Going public:** public reading stays off. Change no default, profile or figure of `clients.public`.
- **The three things left out:** landed work from before a project went public, the history a restart empties, and a stock queue's empty Landings all remain. `browser.status.md` says so already.
- **`docs/help/`:** nothing here should change a page. If a generated `docs/help/reference/*` goes stale, regenerate it. If a page grows past its bound, shorten it, losing no facts and never raising the bound.

## Beside this plan

These files have other plans' work landing beside yours. Merge, don't reorder.

- `temper/crates/temper-client-view/src/words.rs` and `look.rs`: add to the public block, and take the next free id or token at your base. A collision after a merge is caught by the words' and look's tests.
- `temper/crates/temper-client-shell-web/temper.css`: see your spec's marker.
- `temper/docs/design/client/browser.status.md`: remove only the line your spec names.

## Generated files

Generated files your change makes stale (prints, goldens, counted baselines) are yours to regenerate, by their own tool. If a catch-up meets a conflict in one, regenerate rather than hand-merge.

## Limits

No budget or timeout is raised. A heap ceiling that moves is named in your report, with its measurement before and after.
