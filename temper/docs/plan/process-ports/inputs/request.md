# process-ports: tests can't take each other's ports

The way to `process-temper` at three threads, which agents/loop measured as after-landing's biggest lever: 204 s of wall against 504 s, about 5 minutes a run (`temper/docs/design/domain/testing.md`, 2.6 already specifies three concurrent stories). One of the two failures in that trial: `a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path`:

```text
box.err: temper box: .../held-login-guard/config/temper.json: port 45949 (127.0.0.1:45949): cannot listen: AddressInUse
```

Figures and logs: `fab/retired/test-speed/temper/docs/plan/test-speed/figures/tier-threads.md` on hel8. 45949 was the box's retained webhooks listener, a port the kernel chose earlier, and the story restarts on it. Who took it isn't established. The likely reading: a port from the kernel's ephemeral range, free while the box was stopped, went to another process's outgoing socket or listener.

## The change

A test's chosen port never comes from the ephemeral range, and two concurrent test processes never choose the same one. In the test support (`support::free_addresses`, `support::own_listener_files`, the fake sandbox's ports, wherever a port a test will listen on again is chosen). Say what you found; a block of ports per nextest slot (`NEXTEST_TEST_GLOBAL_SLOT`), or a small lock file per port under the target directory, are the obvious shapes; take the simplest that also holds between tiers on one builder. Read `/proc/sys/net/ipv4/ip_local_port_range` rather than assuming it. A person's box is unchanged. A retained deployment's restart keeps its ports, as now. Check `temper/crates/temper/tests/fake_backends/` and `temper/tests/` for other tests choosing ports by binding to 0 and then releasing them.

**Keep `test-threads = 1`:** raising it is a later, separate change, after temper/forge's look at the second failure. The fixed callback port's `temper-sign-in-callback` group stays.

## Proof

Focused tests of the choice: outside the ephemeral range, disjoint across two slots, a retained restart keeps its port. `cargo check --tests` for what changes in the process tier. Nothing in the slow tiers runs in this plan: after-landing proves it.

## Size

One task. One cut is at most 15 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/domain/testing.md`

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
