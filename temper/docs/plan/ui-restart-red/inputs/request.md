# ui-restart-red: `ui-repo-link` broke restarts and two process stories

After-landing has been red since `ui-repo-link` landed, and it's my plan's red. At `378c289eaf02` these fail, alone as well as in the tier, and they passed before it:

| Story, `temper::process` | Where | Message |
|---|---|---|
| `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart` | `tests/support/first_repository.rs:134` | init engine settings: a problem at `clients.provider` |
| `playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge` | the same | the same |
| `a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart` | `tests/support/forge_land.rs:767` | `second: no forge-1/admin_token` |
| `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` | see the log | |
| `startover::owned_setup_cuts_start_over_and_a_removal_cut_finishes_on_restart` | `tests/suite/startover.rs:142` | |

`temper::live`'s two owned-forge landing stories were red in the same run. Don't run them: read whether the same cause explains them, and say so. Live logins aren't this plan's.

## What a foreman read, unproved

1. Commit `8a8a1268fd` added `"provider": "$provider"` to the init template. `first_repository::run_spend` parses the template without expanding that placeholder, so the two setup stories die at once. That may only be the test's helper, or other readers of the template may have the same problem: check.
2. Commit `239cef0884` made `temper serve` start before a restored owned group when a box signs its person in. The design it brought (`temper/docs/design/protocol/setup.md`, the restart lines) still says preparation makes a new handover and serve takes it over. **With serve already running, the handover may never be taken,** leaving `forge-1/admin_token` on disk after a restart, and possibly the forge not reconnecting. `Prepare::later_token` is where it's made. **This is the important one: a secret left on disk, and a restarted box that may not work, on the default path.** Prove what really happens first.

## The ruling

All five stories pass, with the order of startup whatever makes both true: a restarted box signs its person in through the source, which fetches discovery from temper while Forgejo starts; **and** the handover is taken and no token file stays. Don't weaken any of the stories' assertions, least of all that no token file remains. If the two orders truly can't both hold, a discovery fetch that retries, or making the source's fetch not block Forgejo's start, may be better than the order: pick the smallest that's sound, and bring `setup.md` and `sign-in.md`, 9.9, with it. A restart that loses one-click sign-in for a moment, with the password still working, is better than a token left on disk.

## How

**This overrides the common block's first bullet for these five stories alone:** it's a red's repair, so run each by name, alone, with `heavy`, and report each one's result. Never the whole tier. Also a focused test for each cause, shown red without the fix.

`ui-repo-link-2` is touching the box's lifetime nearby: if it has landed, take it. One or two tasks. No `Look-request` trailer.

## Sources

- `temper/crates/temper/tests/support/first_repository.rs`
- `temper/crates/temper/tests/support/forge_land.rs`

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
