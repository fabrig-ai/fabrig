# ui-forge-reviewer: a review counts only from the forge it was made on

The first piece of `temper/docs/design/draft/forge-identity.md`, section 3: its hole, which that draft says to close first. Sections 4, 8 and 9 are later plans: don't build the link, `Reconcile`'s login, `AppMade` or the approval's card here.

**The hole, read in the code at `d893a93cc1` and confirmed by the draft's review, not run:** provider 0's subject doesn't name a forge, and `landing_reviewers` (`temper-engine-domain/src/route/forge_route.rs`) looks up `{ provider: 0, subject: review.author }` whichever forge the review is on. In a project on both an owned Forgejo and GitHub, GitHub's user 3 is taken for the party adoption seeded from Forgejo's user 3, role and all. Anyone may post a review on a public repository's pull request, so a stranger's approval can count toward a landing.

## What this plan makes true

1. **Provider 0's subject names the forge:** the forge's number then the user's, 10 bytes, opaque to everything but one function of the forge's. It's a stored format changing in place, with no migration: there's no supported deployment.
2. **One lookup, by forge and user,** which all three readers use: `human_forge_identity` and `landing_reviewers` in `forge_route.rs`, `forge_user` in `route/host_route.rs`, and the inventory's line, which now names each forge. Leave a seam in that function for the draft's later link, without building it: it answers the party whose key is that pair's subject, else unknown.
3. **Reproduce it first, failing:** in the lowest world, a project on two forges with the same user number on each, where the stranger's review must not count, and the same person's review on their own forge still does. Adoption seeds with the forge's number in the subject.
4. Bring `temper/docs/design/domain/forge.md`, section 4, 8.7 and 12.1, `protocol/forge.md`, 2.1 and `protocol/store.md`, section 2 in line, and the draft's status.

## Limits

One or two tasks. Focused and fuzzy alone: no browser, walk or render check, by the user's rule that a plan is complete when it lands; slow-tier stories that read the subject change with it, compiled and not run. Beside it, temper/migration's `goal-delivery` writes `forge_route.rs` too: whichever lands second catches up. New cases are rows of an existing table test.

## Sources

- `temper/docs/design/draft/forge-identity.md`
- `temper/crates/temper-engine-domain/src/route/forge_route.rs`

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
