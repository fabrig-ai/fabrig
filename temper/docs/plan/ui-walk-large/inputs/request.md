# ui-walk-large: what the first whole walk found too large for its own fixes

`ui-final-walk` walked the client on a live fake box. Its `findings.md`, W3 and W6, with reproductions, are on `origin/ui-final-walk/integration` at `temper/docs/plan/ui-final-walk/findings.md` (the evidence is on hetzner8, `~/src/fabrig/ui-final-walk/walk/`). Both break the flow the mock draws. There is no supported deployment: stored formats and protocols change freely; decide, and note it in `plan.md`.

## W3: a settings proposal doesn't reach the project's Needs you

A real `project-ways` proposal waits for project 1's owners in ctl, but its inbox row and proposal have project 0. So the project's Needs you says "Nothing needs you" and the rail has no count. "Configure temper" ends in exactly such a proposal, so this is its main path.

**Decided (fabrig/ui):** a settings proposal belongs to the project its items touch. An item-less or deployment-wide proposal stays deployment-level, and shows in the Needs you of every project whose owner can decide it. A proposal touching several projects shows in each of them. Set it when the proposal is made, in the engine, not by the client guessing. The card, count and Accept work from the project's Needs you, and the chat's card stays. Prove it with a world test and a browser walk: Configure temper's proposal, Needs you and the rail's count, Accept, the rail changing.

## W6: Home's brief never appears on a fresh project, silently

After a landing, 31 minutes away and a return, Home says "you last looked 32 minutes ago", delivers no brief, and shows no error. The default period has no unreserved funds, because the owner's pool reserves the whole period, and the sign-in made a personal account, unlike the process test's deployment account and funding. So a new user never sees the page's main idea.

**Decided (fabrig/ui):**
1. **The brief can run on a fresh project.** A stock project keeps room for it: the pool doesn't reserve the whole period, or the status agent's small allowance sits outside the pool, whichever is the smaller change. Find which it should be from `temper-service/src/stock.rs` and `limits.rs`, and say why.
2. **If it still can't run, Home says something true:** the computed sentence stands (`ui-home-status`: what landed since you last looked), never just "Nothing needs you" over a real landing. A failed or unfunded brief isn't an error to the person.
3. Check it with a personal account from a real sign-in path, as the walk had, not only a deployment account: fake ChatGPT sign-in, a landing, a return after the gap.

## How the cut shapes it

Two to four tasks. Design documents move with it (`client/README.md`, `domain/agent.md`, `protocol/configuration.md`). This plan follows `ui-final-walk`; expect a catch-up.

## Size

One cut is at most 30 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1); the pool's reservation above is a stock setting, so it's in scope, but no test budget or timeout is.

## Sources

- `temper/crates/temper-service/src/limits.rs`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
