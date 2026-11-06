# ui-chat-accept: what every task holds

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **No protocol or stored format changes here.** The fix is the client's: jig's refusal is right. If a task finds it needs one, it may make it: no compatibility, no `Question:`, and it says so at the top of its report, for temper/forge.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task, by their own tool.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** on a catch-up, regenerate rather than hand-merge; keep `names.txt` sorted.
- **Beside this plan:** `configuring-words-2` changes the same chat's stock words and `ui-pause-close` its bar. Find words and controls by the view's `Word` and the design, and on a catch-up take theirs. `finding-400` ("the configuring chat can't run after its proposal is accepted") is another plan's: its cause is a startup fault after acceptance, in `temper-worker-domain`'s `agents.rs` and jig's host. Touch neither.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. A throwaway render script is allowed, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Screenshots** go in `~/src/fabrig/ui-chat-accept/` on the builder, outside any worktree.
