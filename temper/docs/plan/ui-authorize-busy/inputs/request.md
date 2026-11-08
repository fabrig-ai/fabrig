# ui-authorize-busy: a cross-site authorization isn't refused as busy under load

A red's repair, `red-72`, after `red-71`, same test. `browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state` (`temper/tests/client-browser/tests/browser_all/browser_authorize.rs`, near line 106) fails at step 7. It finds no heading "Authorization received", and the page shows `Word::AuthorizeBusy`: "temper couldn't sign you in just now". That word is the service's 503 for a sign-in it can't serve (`temper-service`'s sign-in pages; `temper/crates/temper-service/src/tests/sign_in.rs`, near line 306).

**What's known, from after-landing on 2026-11-08:**
- It failed at 00:09Z, 00:21Z and 01:23Z (tip `d956d76b6f`), and passed at 00:32Z, 00:44Z and 00:55Z.
- In the full suite, where it runs third of 12 beside the others, a failing run takes about 12 s. Run alone, it passes in about 2 s. At 00:09Z the rerun alone failed too, at 12.8 s.
- So it's load and time, not order. Something on the authorize path waits about 10 s, then answers busy. No code between the passing and failing tips touches it.

The first repair, `ui-authorize-red`, ran it alone, saw it pass, and changed nothing.

## One task

1. **Find which bound answers busy,** by reading the authorize path: the clients' admission, the engine's gate or readiness, the people store's answer, and a deadline of about 10 s. Then hold that bound in a **focused test**, red first: the authorize arrives while the bound is held, or while its answer is slow past that deadline, and today gets the 503.
2. **Decide at the lowest place, and say which in `plan.md`:**
   - If a person could meet it, it's the product's. A box just started or under load refuses a sign-in. Then the authorize waits for the engine within a sign-in's own bound, or its page says temper is starting and retries by itself. It must not refuse as busy while the engine is merely slow.
   - If only the story's world meets it, it's the story's. It starts the authorize before the box is ready, which a person's box never does. Then the story waits for the readiness a person's client waits for.
3. **The story's checks stay as they are.** Run it by name, alone, through the build wrapper, as a red's repair may. Run it at least five times under a load you make, such as a parallel focused build, and say how often it passed.

## Sources

- `temper/tests/client-browser/tests/browser_all/browser_authorize.rs`
- `temper/crates/temper-service/src/tests/sign_in.rs`
- `temper/crates/temper-client-view/src/words.rs`


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
