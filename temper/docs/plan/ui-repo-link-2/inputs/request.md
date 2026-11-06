# ui-repo-link-2: the box's lifetime and its Forgejo sign-in settings

`ui-repo-link` gave a box one-click sign-in to its owned Forgejo. A code review (temper/ui, by reading) found these. Each is plain engineering. `temper/docs/design/protocol/sign-in.md`, 9.2, 9.3 and 9.9, and `setup.md`, 7.1, are the design: bring them with each change.

1. **Closing the terminal leaves Forgejo and serve running.** The box handles only SIGINT and SIGTERM, and each step has its own process group, so a hangup ends the box alone (`skein-shell/src/linux/process.rs`, about 83 and 266; `temper/docs/design/domain/forge.md`, about 762; `temper-forge-setup-shell/src/parent.rs`, 44-52). The next `temper box` then refuses on the held ports.
   **Do:** the box treats SIGHUP as a stop, with its usual stop order. On Linux, its steps start through `forge-child --parent BOX --signal terminate`, as `temper/crates/temper/tests/support/boxed.rs`, about 1102-1120, does, so they end with the box however it ends. macOS stays as it is, with no parent-death signal: say so in the design. A step test for each.
2. **A moved port leaves Forgejo's sign-in source on the old address.** `temper/crates/temper/src/forge_sign_in.rs`, about 137-139: a source already listed is left as it is, while the box's own refusal (`box_command.rs`, about 204-210) invites editing the port. Then one-click sign-in goes to the old port.
   **Do:** in `prepare`, before Forgejo loads the source, set its discovery address to the current one (`forgejo admin auth update-oauth`; check the flags against the throwaway Forgejo binary on the builder, since I couldn't). Still never fails the set-up. A step test.
3. **`public` with `sign_in` writes `auto`, adds the source, serves nothing and shows no off fact** (`temper/crates/temper/src/forge.rs`, about 239; `temper-service/src/clients/forge_sign_in.rs`, about 195 and 235). **Do:** refuse the pair at its path, in the configuration's words, until it's proved. A focused test.
4. **Forgejo's session lasts far longer than one click needs.** Every press signs in again. **Do:** in the owned Forgejo's settings (`temper-forge-setup/src/ini.rs`, about 89-120), add `[session] SESSION_LIFE_TIME = 3600`; check the key against the binary. In `sign-in.md`, 9.9, and `temper/docs/help/box.md`, one plain sentence each: a box trusts every program listening on this machine's loopback, since browser cookies ignore ports. Adjust `sign-in.md`'s "one person holds the machine" to match (about line 831). Focused test for the setting.
5. **An empty OWNER or NAME still builds a repository link.** `temper-client-view/src/forge/repository.rs`, about 45-64: return `None` on an empty part. A focused test.
6. **A walk of a default box.** By a throwaway driver, at 1440: `temper init --box` with its default runner and separate config, data and cache directories; one click lands signed in; restart the box; a click still lands; stop the box, edit temper's port, start again; a click lands by the new port; close the box's terminal (a pty hangup), and nothing of the box is left running. Report what each step showed. Nothing committed names a lab path.

Two tasks at most. No `Look-request` trailer. The review's own file lands separately.

## Sources

- `temper/crates/temper/src/forge_sign_in.rs`
- `temper/crates/temper-forge-setup/src/ini.rs`

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
