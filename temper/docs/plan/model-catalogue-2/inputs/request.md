# model-catalogue-2: models live in the store, a person can add one by name, and Settings has a Models tab

At the user's request (2026-11-06, relayed from their laptop: the catalogue's later steps are to be taken care of). The user's words are at the top of `temper/docs/design/draft/model-catalogue.md`: "the models list should live in the store and not in the config file."

**This is step 2 of `model-catalogue.md`, section 7: its sections 4 and 6, and the page on Settings.** Step 1 landed as `model-catalogue-1`. Step 3, section 5, reading each provider's list, is the next plan, `model-catalogue-3`, built on this one's store: don't build it here, and leave its seams plain.

## What this plan makes true

1. **`model` is a new kind of setting** (section 4; jig's `domain/engine.md`, 17.1), with what section 4 lists. `picker`, a configuration member until now, moves to it.
2. **The defaults are the binary's,** as a charter's are, and a model a person turned off stays off across a release (17.7).
3. **The models leave both files:** `engine.llm.models` and each provider's `models` from `temper.json`; each endpoint's `models` from `agent.json`. Providers, endpoints and sources stay.
4. **smith takes a model from the charter,** as section 4 says, and checks the charter against its limits in place of a list.
5. **A charter names its first choices and "the rest",** so that a new model needs no charter's change, within the bound of 64 (`MODELS`). Settings says how many it left out.
6. **A change is as any setting's:** a proposal the configuring chat can make, or an owner's act on Settings, Models. A task keeps the model it has. The configuring agent's instructions learn this: they're temper/ui's.
7. **"Add a model by name"** (section 6): a provider and a name, on the picker, the name's shape checked alone, reading "added by you".
8. **Settings, Models,** page 07 of the look request `model-picker` (`47c23ff6ed` on `origin/look-requests`, or by its `Look-request: model-picker` trailer; read its whole message, and `temper/docs/design/draft/look-requests.md`, sections 3 to 5): a tab between Accounts and Builders; a section per provider saying what pays; a row per model with its name, default effort and whether it's the default, and a switch for the picker; "estimated prices" where its prices aren't known; "Add a model by name"; and the line sending a change of default to Configure temper.

**With one account** (the message's last paragraph; `model-catalogue.md`, section 3): the page holds the connected provider's models alone, the other provider being its "Connect" line, with no switch or row for its stock models. An account that stops being usable keeps its line with its own fix, and no models.

## What waits for step 3

"Read again", "Couldn't read OpenAI's list · 2 hours ago", the "new" tag, a section's clause saying where its list came from beyond temper's own, and the per-account record of names. The page draws none of them yet. Say so in `model-catalogue.status.md`.

## Beside it

- `look-round-11` (`model-picker`, pages 01 to 06) is on its way to main, and its All models links to Accounts until this page exists. Whichever lands second catches up; if it's on main when you reach it, change that link to Settings, Models. If it isn't, say so in `plan.md`, and the link moves at catch-up.
- `model-check` is also on its way to main, with `chat-draft`'s prints re-recorded. A print conflict at catch-up is regenerated, not merged.
- No compatibility: there's no supported deployment. The stored format changes in place, with no version bump or migration.
- The docs named under "Design lags" in `model-catalogue.status.md`, sections 4 and 6, are this plan's to bring in line: jig's `domain/engine.md`, `protocol/configuration.md`, smith's `protocol/agent.md` and `protocol/charter.md`, temper's `protocol/configuration.md`, `client/README.md`, 3.5, and the help's "Offering another model" in `account.md` and `deployment.md`.

Only focused and fuzzy run in the plan, as below. A browser story for the page is written and compiled, not run: a switch turned off and back, a model added by name. Screenshots beside page 07 at 1440, 1280, 1000 and 420 wide, in `~/src/fabrig/model-catalogue-2/` on the builder, outside any worktree, the report saying which builder.

## Sources

- `temper/docs/design/draft/model-catalogue.md`
- `temper/docs/design/draft/model-catalogue.status.md`
- `temper/docs/design/draft/look-requests.md`
- `jig/crates/jig-domain-tasks/src/llm.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
