# ui-proposal-card: for every task

The request: `temper/docs/plan/ui-proposal-card/inputs/request.md`. Its design is written and on the trunk: the sections each
spec cites.

- **Only focused and fuzzy run in this plan (the user, 2026-11-06).** Never `tools/tiers/run`, a browser, process or live tier,
  one of their stories, or a tier's CPU figure: we run optimistically and repair if after-landing flags. A new story for those
  tiers is written and compiled (`heavy cargo check --tests`), not run. This overrides anything below or in a spec. A look's
  render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface
  may change freely: no compatibility, no `Question:`; say so in the report. A small edit a task needs just outside its file
  list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only
  for what truly blocks the task.
- **A passing scripted story proves little here.** The request's faults showed on a real model and not in the scripted stories.
  A real model's calls name no proposal, and neither may a test that stands for one: leave `call.objects` empty unless the test
  is about a call that names it.
- **Failing tests first,** at the lowest tier: write each test, see it fail for the rule's reason, then the code. The report
  names the rule and the failure seen.
- **`temper.css`:** a new rule goes under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes,
  settings, welcome, phone), never at the file's end, so that parallel plans merge cleanly. A replaced rule goes where it stands.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`** conflict across plans: on a catch-up, regenerate rather than
  hand-merge, and keep `names.txt` sorted as it is.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) your change makes stale are yours to
  regenerate in the same task; don't park on them. A wire change makes journey recordings stale: re-record them in that task.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh
  fake box; if all fail, widen the mark's wait in the recording driver and say so.
- **A new `Word`:** the variant in `pub enum Word`, the entry in `ALL` (`SKEIN_CODEGEN_WRITE=1` on
  `english_phrase_declarations_match_the_vocabulary` rewrites it), its code in the `Self::X => N` table, the next free number when
  you add it and again on a catch-up, its English in the phrase table, and the declared counts, as a counted baseline.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's
  `Emulation.setDeviceMetricsOverride`. A throwaway render script is allowed, untracked and gone before the push. Handle
  `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page** (`temper/docs/design/client/greenfield/`); where an old test walks a part the mock
  removes, change it with it. "charter 1" stays, as 5.13 says.
- **Help pages:** a `docs/help/` page your change grows past its bound is yours to shorten, losing no fact, never raising it.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it
  checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your
  branch fails, it's yours.
- **Files beside other plans:** `chat-reading-c` (forge's) changes how a chat reads back and may land while you work: keep to the
  card's own code, and on a catch-up take its reading window as it is; Older's thin first page isn't this plan's.
  `long-words` is reshaping the composer (`temper-client-domain`'s `pages/composer.rs`, `pages/chat.rs`; the view's
  `pages/chat.rs`, `pages/session.rs`), and `model-catalogue-1` and `model-check` touch `words.rs`. Expect catch-ups; keep your
  edits to those files to the lines your spec names.
