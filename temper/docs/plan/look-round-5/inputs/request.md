# look-round-5: "Configure temper"

At the user's request (through their laptop session, 2026-11-04): land the look request `configure-temper-name` (`cea9fdb5d2`, on `origin/look-requests` at `73e4f0e235` or later), released: the user accepted the name and asked for the page's opening to read as a page. **"Configure temper", its opening laid out as Home's, with the ways to begin as cards.** Amended after the user's nits: no kicker, the opening starts at the conversation's top so that it shows whole on a laptop, and the conversation and the composer share one centred column, with a scrollbar's gutter on both edges. It sits directly over `configure-chat-in-settings`, which `look-round-4` landed.

**The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Read the commit and its whole message. Every commit carries `Look-request: configure-temper-name` directly above `Co-Authored-By:`, a later fix and a red's repair included.

`new-menu` and `configure-chat-draft` stay held: do not land them.

**Before the cut's tasks,** check `origin/look-requests` for any other released request above `configure-chat-in-settings`, and name it in the report rather than landing it.

Screenshots come back to the user, beside the request's pages at 1440, 1000 and 420 wide with open states, in `~/src/fabrig/look-round-5/configure-temper-name/` on the builder, outside any worktree; the report says which builder. Never committed. Throwaway render scripts stay untracked.

## Size

One or two tasks. Started by fabrig/ui, which owns the client's plans from here (2026-11-05): decide rather than ask; a change to a stored format, a protocol or a public interface needs no `Question:`, only a note in `plan.md`. Put new `temper.css` rules under the `/* ==== ui: settings ==== */` marker. Landing soon beside this, `ui-chats` takes the same `.temper-session` layout to every chat; keep these rules in `temper.css` plainly named so it can.

## Sources

- `temper/docs/design/draft/look-requests.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
