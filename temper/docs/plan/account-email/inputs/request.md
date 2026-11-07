# account-email: temper names an account by its email

At the user's word (2026-11-06, relayed from their laptop). **Scope: `jig/docs/design/draft/account-facts.md`, section 6, steps 1 and 2.** The user settled the format and the audiences on 11-04, and the draft has no open questions. There's no supported deployment, so formats change in place.

## What this plan makes true

1. **The format (step 1):** the record, its key and codec, and the client's line with its audiences.
2. **The email (step 2):** skein's OAuth reads the verified email claims its caller asks for; jig keeps kinds 1 and 2 at sign-in and refresh; temper names the account by its email.
3. **The look:** `ready-account-email`, whose gallery display is on main (`cc0311966f`): Ready says "ChatGPT plan" alone today. `model-check-pane`'s "who it is" and the picker's heading for two accounts of a provider read from the same place, so they read the email too.

## One gap to close: a Claude subscription

Those steps read the email from a sign-in's ID token, so a Claude subscription, the user's own box, still shows none. smith reads no email from Claude Code today. Find out whether Claude Code's status reports one, on a box the task may read: it runs the program and reads what it says, and reads no credential file. If it does, keep it as kind 1 with origin 5. If it doesn't, say so in the draft's status, and the Claude line stays as it is.

## Left out

Section 6's step 3: usage windows, plan, renewal and next reset. The user is still deciding on those. The format holds them, so nothing is lost.

## Beside it

`model-catalogue-3` touches accounts too, reading each provider's list per account. Whichever lands second catches up. A journey whose prints this changes is regenerated with `temper-client-journeys again`, as below. A browser story changed by this is compiled, not run.

## Sources

- `jig/docs/design/draft/account-facts.md`
- `jig/docs/design/domain/providers.md`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **`temper-client-journeys again` is generation, not a walk:** it regenerates what the gate's focused replay checks read, so a task whose change makes a recording stale runs it, though it drives a browser.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
