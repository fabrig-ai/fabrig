# model-catalogue-3: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins. The request is `temper/docs/plan/model-catalogue-3/inputs/request.md`; the draft is `temper/docs/design/draft/model-catalogue.md`, section 5.

## Departures from the implementer's text

- **Decide, don't ask (the user, 2026-11-06, asleep).** A stored format, a protocol or a public interface may change freely: no compatibility, no version bump, no migration, no `Question:`. A family's schema, generated code and one set of golden bytes change in place (`jig/docs/design/protocol/store.md`, 5). A `Question:` is only for what truly blocks the task.
- **A small edit your task needs just outside its file list** (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked.
- **Design text.** A task edits the `docs/design/` lines its `Owns and may change` lists, the status line of what it finished, and any single design line your work proves wrong; the rest of `docs/design/` stays as this plan's cut wrote it. What the code taught goes in the report.

## What holds in every task

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, a `<tier>-<tree>` profile, `scripts/browser.sh`, or a tier's CPU figure. A new story for those tiers is written and compiled (`heavy cargo check -p PKG --tests`), not run; say so in the report. This overrides anything else. No walk, no render check, no throwaway driver: those follow landing. `temper-client-journeys again` is generation, and a task whose change makes a recording stale runs it.
- **Tests stay cheap.** A new case of one behaviour is a row of an existing table test, not a new test (`skein/docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate. **No new browser test:** the tier's at its cap.
- **No credential is printed, logged or kept beyond what's already kept.** A read carries a grant as a run does. Nothing here reads a key, a token or a provider's error body into a log, a fact, a record or a failure's words; a failure is a type with a status. **No request goes to a real provider:** a fake's list proves everything here, and a live read follows landing.
- **`temper.css`:** new rules go under the marker `/* ==== ui: settings ==== */`, never at the end of the file. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`** conflict across plans: on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) your change makes stale are yours to regenerate in the same task, by their own tool.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.
- **Help pages:** a `docs/help/` page a change grows past its bound is that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper your change's correct behaviour trips** is yours to repair in the same task, keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **A fixture's document needs no new member.** `keep` is optional, and a provider without it reads nothing, so no fixture changes unless your spec says so.
- **Neighbours:** `account-email` runs beside this plan on accounts: whichever lands second catches up. `limits-worlds` has `jig/crates/**` and `temper/crates/**` hunks on the trunk beside you. Keep your hunks in a file you don't own small.
