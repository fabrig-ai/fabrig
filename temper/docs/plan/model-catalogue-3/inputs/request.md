# model-catalogue-3: temper reads each provider's list

At the user's request (2026-11-06, from their laptop; their words are at the top of `temper/docs/design/draft/model-catalogue.md`: "fetch from the provider all currently available models (so if a new model comes out we don't need a release/redeploy)").

**This is step 3 of `model-catalogue.md`, section 7: its section 5, on the store `model-catalogue-2` landed.** Read section 5, and `model-catalogue.status.md`'s entries under 5 and 7, which name what's left.

## What this plan makes true

1. **A worker reads each account's list,** as it checks a program's source (jig's `domain/accounts.md`, 3.4): the core sends the account's grant, and smith's agent makes the one request on the endpoint's address and transport. It isn't a run, and spends no budget. The table in section 5 says what each source reads and gives. **A Claude subscription reads nothing:** Anthropic allows no other use of its credentials (jig's `domain/providers.md`, 3.1).
2. **When:** as an account becomes usable, once a day, and on "Read again". A read that fails keeps what the store has and says so.
3. **What it keeps and changes, as section 5 says:** `keep` and `drop` patterns per provider; a new name comes in off the picker, "new" for a week; a name gone from every list stops being offered and is kept for tasks that hold it; a read that changes nothing makes no revision, and one that does is decided by the provider's list, moving no running work. **A catalogue change marks Frames only when it moved something** (`ui-fuzzy-joint`'s lesson, `routing/mod.rs`).
4. **Per account:** the store keeps which names each account's last read gave, which section 3's offer reads.
5. **What a list doesn't say:** a new model takes its provider's defaults: its dearest stock model's prices, the stock's window and output, no efforts. Settings marks its prices as estimated, and the configuring chat can set them.
6. **Settings, Models, the states step 2 left:** "Read again", "Couldn't read OpenAI's list · 2 hours ago", the "new" tag on Settings and All models, "Read from OpenAI 2 hours ago", from the look request `model-picker`'s page 07 and message (find it by its `Look-request: model-picker` trailer on `origin/look-requests`).

Bring what `model-catalogue.status.md` names under 5 and 7 into line: jig's `domain/accounts.md`, 3.4 and `domain/engine.md`, 17.7, and smith's `protocol/agent.md`.

## Limits

No credential is printed, logged or kept beyond what's already kept: a read carries the grant as a run does, and a live read is proved after landing. A fake provider's list proves the rest. A stored format changes in place, with no version bump. Focused and fuzzy alone, as below; `temper-client-journeys again` is generation; no walk or render check. No new browser test: the tier's at its cap. A new case is a row of a table test. `account-email` runs beside it, on accounts: whichever lands second catches up.

## Sources

- `temper/docs/design/draft/model-catalogue.md`
- `temper/docs/design/draft/model-catalogue.status.md`
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
