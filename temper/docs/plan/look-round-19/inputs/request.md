# look-round-19: Start's page, To begin in two steps, and a night mode

A look plan, step 1 of `temper/docs/design/draft/stock-ways.md`'s section 7: three look requests on `origin/look-requests`, each found by its trailer, with nothing needed before them. Follow `temper/docs/design/draft/look-requests.md`, sections 3 and 5, for each. Read each commit's whole message: what its pages show, and the states they don't show, which this plan builds too.

## Tasks

1. **`Look-request: ask-temper-start`** (`e9da9f87a8`): what Start opens on a new project. Its message says it adds nothing beyond `ui-ask-m1`'s move of Configure into Ask temper. So this is its account (section 5): each state it shows matched against what's on main, and the account committed. If one differs, make it match.
2. **`Look-request: to-begin-two-steps`** (`4332a26265`): To begin in two steps, the repository first, and the chat step gone (`Facts::left()` counts two). Build every line and note it names, the `HomeNote*` words by its rule, with `HomeNoteThree` and `HomeNoteChats` gone. While To begin is unfinished, the rail on Home and on Ask temper's page holds Home, Ask temper and Settings, and the phone's tab bar Home, Ask and Settings, three equal tabs. Home's Needs you and Chats sections stay out while the card shows.
3. **`Look-request: night-mode`** (`3c5333485a`): a first pass, kept by the user to refine later. Tokens first, so `temper.css`'s colours come from tokens with a night set, after fabrig.ai's colours as the request's pages draw them. The switch is at the rail's foot, a moon by day and a sun by night, and at the top bar's right on a phone. The glow is small and faint, at the window's top right, over the whole window, with a fine grain and no rings. Build it on the request's three pages: the landing, Ask temper, and a chat beside its draft. The rest of the client takes the tokens as they fall, with no page left unreadable at night. Where the switch's choice is kept, and whether a first visit follows the system's scheme, are this task's to decide, as the smallest thing that holds across reloads. Say which in `plan.md`.

The browser tier's cap of 12 holds: a new case is a row of an existing table test, and the gallery gains states for night.

## Sources

- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/draft/stock-ways.md`


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
