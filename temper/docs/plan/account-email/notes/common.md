# account-email: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins. The request is `temper/docs/plan/account-email/inputs/request.md`; the shapes that cross tasks are `temper/docs/plan/account-email/notes/contracts.md`.

## Departures from the implementer's text

- **Decide, don't ask (the user, 2026-11-06, asleep).** A stored format, a protocol or a public interface may change freely: no compatibility, no version bump, no migration, no `Question:`. Each family's schema, generated code and one set of golden bytes change in place (`jig/docs/design/protocol/store.md`, 5). A `Question:` is only for what truly blocks the task.
- **A small edit your task needs just outside its file list** (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked.
- **Design text.** The cut wrote this plan's design. A task edits the `docs/design/` lines its `Owns and may change` lists, and any single design line your work proves wrong; what the code taught goes in the report.
- **A required field crosses trees.** A branch stays in its tree, but it carries the other trees' lines that name a field or arm its change requires, each edited only for that. Run `heavy cargo check -p <dependent> --tests` for each dependent the compiler names.

## What holds in every task

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, a `<tier>-<tree>` profile, `scripts/browser.sh`, or a tier's CPU figure. A new story for those tiers is written and compiled (`heavy cargo check -p PKG --tests`), not run; say so in the report. No walk, render check, screenshot or throwaway driver: they follow landing. This overrides anything else.
- **Tests stay cheap.** A new case of one behaviour is a row of an existing table test, not a new test (`skein/docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost is rejected there.
- **An email in code is `example.com`'s.** No real address, name or organization, in a test, a golden or a log. Never print a real program's status: read it by its fields' shapes.
- **`temper.css`:** this plan adds no rule; if yours must, it goes under its area's marker, `/* ==== ui: AREA ==== */`, never at the end of the file.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`** conflict across plans: on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) your change makes stale are yours to regenerate in the same task, by their own tool. A codec family's schema regenerates with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'`.
- **`temper-client-journeys again` is generation, not a walk:** it regenerates what the gate's focused replay checks read, so a task whose change makes a recording stale runs it, though it drives a browser, through `heavy`. A re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.
- **Help pages:** a `docs/help/` page a change grows past its bound is that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper your change's correct behaviour trips** is yours to repair in the same task, keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **Beside this plan:** `model-catalogue-3` touches accounts too, reading each provider's list per account, and may take the accounts' family's next key tag or the same lines of `jig-domain-accounts`. Whichever lands second catches up: a key tag moves to the next free one, in place.
