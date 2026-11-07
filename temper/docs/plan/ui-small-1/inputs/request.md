# ui-small-1: what a person met on their own box, and a flake

The user's own asks first, from their journeys on a real box, relayed from their laptop.

## 1. A sign-in link that can't sign you in says why

Opening a sign-in link that can't sign a person in always shows the same page: "Signed out. To sign in, open a sign-in link temper printed." The user met it when a fresh box replaced an older one on the same port. Their links didn't sign them in, and the page gave no hint why. Run in headless Chromium at `d893a93cc1`: a used link's `POST` answers 200, nothing signs in, and the box prints a fresh link, which the page doesn't mention. A link with a made-up secret sends no `POST`, the stream answers 401, and no fresh link is printed. A real link from an older box wasn't run, and may take the `POST` path.

**Ruling, the user agreed these, in these words:**
- A used link: "This link was already used. A fresh one is in your terminal."
- A link that isn't this box's, a real one from an older box included: "This link isn't from this temper. Use the one its terminal printed."

The design's list of signed-out paragraphs is in `temper/docs/design/client/README.md`, near line 3669, beside the words for a link's sign-in not being taken. If the client can't tell these apart from what it's told, jig's link view is where that changes (`jig/docs/design/client/domain/contract.md`). Say nothing that helps a stranger probe: these words say only what the person holding the link already knows, and a page reached with no link keeps today's words.

## 2. The configuring agent's approver line

Asked only "how does temper work? briefly", the configuring agent says: "Your forge identity isn't verified in temper yet, so you can't count as a human approver until that's sorted." It reads the inventory's "forge identity unknown: cannot count for approvals" (`temper-engine-smith/src/settings.rs`, 477). A new person reads that as something being wrong, and with no approval required it changes nothing. The user has met it three times.

**Ruling:** in `temper/crates/temper-service/src/stock/configuring.txt`, the agent raises a person's forge identity only when approvals are what's being asked about or proposed. Leave the inventory's fact as it is. The identity itself goes elsewhere: no build of it here.

## 3. A box printed its sign-in link twice: a flake that fails after landing

`a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart` flaked at `f594ab47c5`, `temper/crates/temper/tests/process.rs`, near line 577, "one printed link": the box's stdout held its link twice, the same length and port; its rerun passed. Each rerun costs the tier about 50 CPU s. Not proved: a box prints a fresh link when it restarts serve (`temper-box/src/tests/host.rs`, `a_restarted_serve_has_no_stdin_and_its_link_is_fresh`), so serve may have exited mid-story, which would be the real fault.

- Read the box's code for what else can print a second link, and fix it at the lowest tier if it's found.
- **Change that assertion to compare a count of links, not the bytes:** today it writes the link into the log as decimals, which a `secret=` filter doesn't catch. Other assertions on a printed link, by the same reading.
- On a second link, the panic carries the box's stderr, with `secret=` lines filtered, so that the next one says why. As a red's repair, this one story may run by name, alone, a few times.

## 4. Smaller, words alone

- Home with nothing to do says "Nothing needs you." twice, as its heading and under NEEDS YOU: once.
- A change's panel says "Repairing: a semantic conflict with ..." on a red the base had no part in, which can mislead: say what's repairing, with no blame.
- A read-only chat said a repository "does not exist in the checkout" when it was on Forgejo, listed twice, and gave up; pointing it at the forge tools worked (temper/qa, one sample). The chat's instructions, which are temper/ui's: for a repository not in its checkout, look on the forge before saying so.

## Limits

Three tasks at most. Focused and fuzzy alone, but for item 3's story by name: no browser, walk or render check, by the user's rule that a plan is complete when it lands. New cases are rows of an existing table test. Beside it, `ui-proposal-2` and `model-catalogue-2` change the same instructions files: whichever lands second catches up.

## Sources

- `temper/crates/temper-service/src/stock/configuring.txt`
- `temper/crates/temper-box/src/terminal.rs`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
