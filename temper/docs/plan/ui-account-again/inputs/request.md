# ui-account-again: a forgotten Claude Code account can be declared again

A finding from temper/qa's proof preflight (2026-11-08, dev.fabrig.ai at `7638040201`). A person forgot a Claude Code program account, and temper said "Signed out on every machine". Then they declared it again under the same program name:

    account declare 2 claude_code claude-6 --key ...
    account declare: This act is not offered

That exits 2. The worker still reports `claude-6` as `usable`, with `sign_out signed_out` and `cleanup true`, and provider 2's readiness as unavailable. Forgetting isn't meant to be final: a person who signs out of an account may sign it back in, under the name they gave it before.

## One task

1. **A failing row first**, beside the account acts' table (`temper/crates/temper-client-tools/src/tests/jig/account_acts.rs`), or at the lowest place the cause shows on the fake backends. It forgets a program account, then declares one of the same program and name, and today gets "This act is not offered".
2. **Find why the act is withheld and fix it at its lowest place.** Candidates: the forgotten source still held by the worker, a name kept after the forget, or the offer's rule reading a signed-out source as present. After the forget, the declare is offered and takes the name, and its "it works" check runs as for a first declare.
3. **A withheld act says why and what to do.** Where an account act stays refused for a reason a person can fix, its refusal names that reason and the next step, not "This act is not offered" alone. That goes in the words of `temper-client-tools/src/jig/render/outcome.rs`, ctl's and the web's alike, as a row each.

## Sources

- `temper/crates/temper-client-tools/src/tests/jig/account_acts.rs`
- `temper/crates/temper-client-tools/src/jig/render/outcome.rs`
- `temper/docs/design/client/README.md`


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
