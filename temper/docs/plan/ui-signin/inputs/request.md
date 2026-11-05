# ui-signin: the fake ChatGPT sign-in in a browser

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock. This is the small fault in the way of recording it: a fresh fake box's ChatGPT sign-in fails in a browser, so no recording reaches past Ready. Stand-alone: start now; the larger `ui-welcome` follows and records `project-landing`, not this plan.

**The failure.** On a builder, in headless Chromium, on a fresh `temper box --fake-backends`, "Continue with ChatGPT" returns to Ready with "Model none yet", "Try later". A retry stays on "Pending", the renderer stops answering. Reopened: "OpenAI refused temper’s sign-in. Check the configuration’s registration." A further press: "Try later". Native-client walks pass (`crates/temper/tests/fake_backends/walk/sign_in.rs`). Evidence: `fab/runs/fake-signin-model/record-landing-1/report.md`; logs on hetzner11 in `~/src/fabrig/fake-signin-model/project-home/walk/`.

**The diagnosis, read from the code and not yet run:** the fake issuer's connection cap.
- `temper-fake/src/oauth.rs` passes `connections: PLAN_SLOTS`, which is 2, a bound for token plans doing a second job. `skein-fake-peers/src/oauth.rs` answers `Request::Reject` at that cap, keeps connections alive (`close: false`) and reaps none.
- Chromium holds sockets to the issuer after the authorize 302, its kept-alive one and a spare it opens beside a navigation. The native visitor closes its one.
- `skein-oauth` signs in in this order: key set (`machine/fetch.rs`), visit, token POST. So: the first press's POST is rejected (sent, so not retried): `Later`. A retry's key-set GET is rejected and retried with backoff, and the visit waits behind it: "Pending", then `failure()`, `Unverified { Keys }`, `Misconfigured`. Closing the browser frees the slots, and the next press fails as the first.
- The hang is the driver's: `closing_loses` in `temper-client-shell-web/src/lib.rs` arms `onbeforeunload` while a request is pending, and the driver's navigation raised a dialog nothing answered. Handle `Page.javascriptDialogOpening` in a walk's driver.

**The task:**
1. **Red first:** a process-tier walk beside `a_chatgpt_browser_return_on_the_fake_box_leaves_ready_able_to_create`, which opens two idle sockets to the issuer's address and keeps them open from just before the visit until Ready is read. It must fail as the browser does, with `last_end` `Failed Later`. If it passes, the diagnosis is wrong: instead reproduce in Chromium through CDP, print `temper ctl account list --json`'s `last_end` after each step, and end the task with a `Question:` naming what you found.
2. **The fix, in the lowest tier:** `temper-fake/src/oauth.rs` gets `CONNECTIONS: u32 = 8` for the peer's `connections`: a browser's six per host, the engine's one, and one closing. `PLAN_SLOTS` keeps its own job, and its comment says so. 8 fits `Face`'s `valid` (`io.sockets >= connections + 1`) under the Small profile's 16 sockets. `CODE_SLOTS` is untouched.
3. **Say it next time:** a rejected connection writes one line to the fake issuer's stderr, so `fake-issuer.log` shows it. It is empty today, which is why nobody could tell.
4. **Prove it in a browser:** the walk of `record-landing` in headless Chromium, through "Create your project", to Home with "work/hello is in Hello".
5. Remove the `journeys.status.md` "Not yet" paragraph's cause once recorded (section 10).


Step 5 above is `ui-welcome`'s; here, the walk of step 4 proves it, with its screenshots on the builder in `~/src/fabrig/ui-signin/`, outside any worktree, the report saying which builder.

## How the cut shapes it

One or two tasks. No design change beyond a line in `temper/docs/design/client/journeys.status.md` if a cause there changes. A throwaway walk driver stays untracked and is gone before the push. A change to a stored format, a protocol or a public interface needs no `Question:`: decide, and say so in `plan.md`.

## Size

One cut is at most 45 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/crates/temper-fake/src/oauth.rs`
- `temper/crates/temper/tests/fake_backends/walk/sign_in.rs`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
