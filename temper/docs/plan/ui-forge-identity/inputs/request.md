# ui-forge-identity: nothing lands without a box's person's approval, if they say so

The user's word, 2026-11-06 and 11-07, through temper/draft: build `temper/docs/design/draft/forge-identity.md`. Its section 3's hole is closed (`ui-forge-reviewer`), and section 4's generic part is in skein (`ui-forge-setup`): `Reconcile` given a login and answering an id, `owner` from `GET /app`, and `AppMade`. This plan is the rest: sections 4 to 9, temper's side.

**The story:** a box's person says nothing lands without their approval, the rule is accepted, a change waits under Needs you with its link, they approve on the forge, and it lands. Owned Forgejo and GitHub both, with no `temper ctl`, no collaborator and no adopting again. Read sections 3, 4 and 9 first, then its status file, which lists the documents to change, and section 8's reasons against a person task and a hold.

**Section 6 is decided:** the user's word, 2026-11-07, "accept it for now". On an owned Forgejo the rule stops an agent's mistakes, not one trying, until agents and CI are contained. The proposal's card and the help say so in the draft's words. Mark it decided in `forge-identity.status.md`, with that date.

## What's in it

- The link, as a field of the forge's record (section 3), set from `Reconcile`'s id on an owned Forgejo (4.1) and from `GET /app`'s `owner` on a manifest-made GitHub App (4.2), with 4.3 and 4.4's rules. A stored format changing in place.
- The one lookup gains the link (section 3): the seam `ui-forge-reviewer` left.
- Section 9's scope: a box's one person only, the root handing the connector the sole owner's party at each start, the start refusing `machine.owned.sign_in` beside `application.sign_in`, and jig's small generic accessor for the deployment's owners.
- Section 7's refusal words, and section 8's card and inbox merge for an approval waiting on the forge.

## Limits

Two or three parts; jig's accessor first, generic, with a heads-up named in `plan.md` if it touches the crates hearth/core watches. Focused and fuzzy alone, by the user's rule that a plan is complete when it lands; slow-tier stories are written and compiled, and `temper-client-journeys again` is generation. No new browser test: the tier is at its cap of 12, so a story is a step of an existing one or goes lower. temper/migration's `goal-delivery` and `call-room` write `forge_route.rs` and the stock beside it: whichever lands second catches up. New cases are rows of an existing table test.

## Sources

- `temper/docs/design/draft/forge-identity.md`
- `temper/docs/design/draft/forge-identity.status.md`

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
