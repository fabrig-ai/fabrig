# test-speed: the slow tiers' bottlenecks that need no cache

The user (2026-11-06): "Let's also try to see where the bottlenecks are and try to make them faster." A survey, by reading code and logs and running nothing, is at `/home/free/src/fabrig/coord/ui/report/state-graph.md` on hel8: read it first, and treat its causes as unconfirmed. Its verdict on a cached state graph for the slow tiers: not yet, since the cheap fixes below are worth more; this plan is those. **No budget, timeout or limit changes: those are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1). `browser-trim` has the browser tier: stay out of it.

Measure before and after each, with the same tier on one slot, and report both.

## 1. `a_story_panicking…`: about 30 s idle in every run

31–35 s wall, 1.3 s CPU, in all 23 runs read (`process-temper`); a similar restart test takes 1.7 s. Its hook calls `ctl account list` on a stopped box, probably waiting out `limits::DEADLINE`, 30 s (unconfirmed). **A person may meet the same wait:** `temper ctl` against a stopped box should say so at once when nothing listens. Fix the product if that's the cause, red first, at the lowest tier; else the test.

## 2. Fake-box walks' idle waits

Best wall against median CPU (M): away 23.7 s against 4.1, plan status 52 against 32, home status 18.4 against 6.3, goal 11.3 against 2.6. Goal and away keep `forge.client.poll_ms` at 5 s, which the status walks override. Take a timeline from `box.log` first: home status stays slow despite the override. Make each wake on the event where one exists, or override the poll as the status walks do. If a wait is a person's too, say so.

## 3. `process-temper` runs one test at a time

`test-threads = 1` on 8 cores uses 0.54 of a core. Three threads could save about 300 s of wall per run (E), with no CPU change. The blocker found: fake-box walks bind fixed port 8367 (`AddressInUse` in `94fdd7b1647d`). No stated reason for the setting was found: look in git history and `temper/docs/design/domain/testing.md` first. If nothing forbids it, give those walks a free port as the other tests have, and raise the threads. If a test can't share a machine, group it, as `temper-real-loop` is, and name why. Run the tier three times at the new setting and on a loaded builder: **no new flake, and no test's CPU worse.** If it flakes and the cause isn't quick, keep the port fix and leave the threads, and report it. Note that CPU contention may ease or worsen.

## 4. `again` with a setup file

`chat-draft` has been re-recorded by hand three times because `temper-client-journeys again` can't do its setup (fresh box, Forgejo, sign-in, project). Give a journey an optional setup file beside its recording: controls by role and name, then the reload where recording starts. `project-landing` and `chat-draft` share it, and `welcome-set-up` walks its start. `again chat-draft` then runs alone. Prove it by re-recording `chat-draft` with no hand driver. This saves no test seconds; it ends the hand re-records.

## How the cut shapes it

One task each, unchained where they can be. Decide, don't ask. Design documents (`temper/docs/design/client/journeys.md`, `domain/testing.md`) move with them.

## Size

One cut is at most 20 minutes.

## Sources

- `temper/docs/design/client/journeys.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
