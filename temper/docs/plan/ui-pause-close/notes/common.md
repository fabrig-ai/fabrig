# ui-pause-close: for every task

The request: `temper/docs/plan/ui-pause-close/inputs/request.md`. Its design is written and on the trunk: the sections each
spec cites.

- **Decide, don't ask.** The user is asleep and takes this plan's judgement. A stored format, a protocol or a public
  interface may change freely: no compatibility, no `Question:`; say so in the report. A small edit a task needs just outside
  its file list (a re-export, a `mod` line, a golden) is made and named in the report, not parked. A `Question:` is only for
  what truly blocks the task.
- **No `Look-request` trailer** on any commit.
- **Failing tests first,** at the lowest tier: write each test, see it fail for the rule's reason, then the code.
- **What stays, whatever it says:** `Act` names and `ActKind`s (`Act::Stop` stays a chat's); jig's words and `temper ctl`'s
  commands and answers, which are jig's names (`jig_client::words`, `HoldWord::Stopped` "Task stopped",
  `temper-client-tools`); `Word::ToastPaused`, "Paused: running tasks stop, nothing new starts.", which is true; the welcome's
  checks and the model check's stops (`Word::FlowStopped`, `Word::HelloStopBound`, `pages/model_check.rs`); a forge's
  `Stopped:` (`temper-client-domain/src/forge/words.rs`); `tests/real-loop`'s oracle, which reads the engine's facts. These
  keep working unchanged.
- **`temper.css`:** a new rule goes under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes,
  settings, welcome, phone), never at the file's end, so that parallel plans merge cleanly. A replaced rule goes where it stands.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`** conflict across plans: on a catch-up, regenerate rather than
  hand-merge, and keep `names.txt` sorted as it is.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) your change makes stale are yours to
  regenerate in the same task, but for a journey's recording and prints in `chat-pause`, which says otherwise.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's
  `Emulation.setDeviceMetricsOverride`. A throwaway render script is allowed, untracked and gone before the push. Handle
  `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page** (`temper/docs/design/client/greenfield/`); where an old test walks a part this
  removes, change it with it.
- **Help pages:** a `docs/help/` page your change grows past its bound is yours to shorten, losing no fact, never raising the
  bound.
- **Files beside other plans:** `ui-walk-3` lands first; `chat-reading-c` and `model-catalogue-1` touch
  `temper-client-view/src/pages/chat.rs` and `words.rs`, and `model-check` `temper-client-view/src/tests/`. Expect catch-ups;
  keep your edits to those files to the lines this plan names.
