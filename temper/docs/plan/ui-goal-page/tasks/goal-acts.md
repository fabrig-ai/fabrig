+++
id = "goal-acts"
part = "goal-acts"
kind = "feature"
needs = ["goal-worlds", "goal-gallery"]
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/task.rs",
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/acts.rs",
  "temper/crates/temper-client-domain/src/tests/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/task.rs",
  "temper/crates/temper-client-view/src/confirm.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
  "temper/crates/temper-client-view/src/tests/toast.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tests/client/tests/suite/task.rs",
]
effort = "high"
minutes = 90
+++

# A goal's acts

## Goal

Amend, Pause or Resume, and Stop on a goal's page, as `temper/docs/design/client/README.md`, 5.11, says, with their outcomes in the toast.

Acceptance:

- **`Act::Pause`, `Act::Resume`, the toast's new kinds and `needs::Act::Dismiss`** as
  `temper/docs/plan/ui-goal-page/notes/contracts.md`, The goal's page's acts, gives them.
- **The acts:** `.acts` after "Draft" or "Chat T2": "Amend", "Pause" or "Resume", "Stop" (`.btn.quiet.small`), each by `act`'s admission.
  While live; "Resume" alone in state paused and offered Release; "Pause" otherwise when offered Stop. Ended, none of the three.
- **Pause:** `Ask::Stop` at once; toast "Paused: running tasks stop, nothing new starts." **Resume:** `Ask::Release` naming the hold's
  revision, at once; toast "Resumed."
- **Stop:** `Act::Close`, the confirmation, `Role::Dialog`: "Stop TITLE?", "What landed on its branch stays there.", "Confirm", "Cancel";
  `Ask::Cancel` with the reason "Stopped from its page". The `Effect::Cancel` Impact words are gone from a goal. A chat's close doesn't move.
- **Amend:** `amendment` between the header and the bands, in `GOAL_FIELD`, `GOAL_TEXT`, `GOAL_BTN`; "Specification", "Tracked", "Priority",
  "Send amendment", "Discard" and what each does unchanged, its outcome in its region.
- **Refusals:** in the toast, in jig's words, "Look again" after a moved one, "Unknown" past retention. Nothing done before its answer; a double
  press joins the pending request.
- **Gallery:** `goal-amending`, `goal-stop-dialog`, `goal-toast`.
- The tests below pass, each having bitten; `temper-client-domain`, `temper-client-view` and `temper-client-gallery` are green whole.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.6, 3.7, 5.11; 3.5, *Needs you*, The toast.
- `pages/task.rs` (domain): `send`, `Sending`, `hold_revision`; `pages.rs`: `task_confirmation`, `confirmation`; `acts.rs`:212 for `Ask::Release`.
- `confirm.rs`, `main`; `frame.rs`, `toast`.

## Owns and may change

Owns, as `files` lists them.

Options, each pre-approved:

- **`amendment`'s look:** a parameter, or a small struct of tokens: the agent's page keeps `TASK_AMEND` and what it draws.
- **The reason's meaning:** "Cancelled from its page" had one reader. Change it; a test holding it follows.
- **Resume's revision:** the chip's hold's revision, `chip_hold`, as an own held card's Release named it. If the document has moved, the engine
  refuses as moved and the toast says so.
- **The toast in the terminal:** as `frame.rs`' `toast` prints the others. If it's `Shell::Browser` only, leave it: the page's state shows the end, and
  say so.
- A world's assert that `goal-worlds` left on Stop's dialog is yours now: `temper/tests/client/tests/suite/task.rs`, its words alone.

## Steps

1. The domain: acts, `Sending::Resume`, routing, the outcome to the toast. A change of what a `Toasted` holds: the Needs page's and Home's arms keep what they
   say.
2. The view: `.acts`, the dialog's words, `amendment`'s look, the toast's arms.
3. **Tests:**
   - domain, `tests/pages/goal.rs`: `pause_sends_stop_at_once`, `resume_names_the_holds_revision`, `a_double_press_joins_the_pending_request`,
     `a_done_pause_toasts_on_its_answer_not_before`, `a_refused_stop_toasts_jigs_words`, `stop_opens_its_confirmation_and_sends_its_reason`, `resume_is_refused_unless_paused`;
   - view, `tests/goal.rs`: `the_acts_in_order_by_offer`, `not_offered_not_drawn`, `ended_has_none`, `paused_says_resume`, `held_otherwise_has_no_resume`,
     `the_stop_dialogs_words`, `the_editor_sits_between_header_and_bands`;
   - `tests/toast.rs`: `paused`, `resumed`, `a_moved_refusal_says_look_again_and_dismisses`;
   - the agent's page: `an_agents_stop_still_confirms_as_before`.
4. **A change of meaning:** `Toasted` and `confirm.rs` have other readers. `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p
   temper-client-gallery -p temper-client-shell-terminal`; `-p temper-client-world -E 'test(task)'`.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.

## Rebase (2026-11-05, step merge, foreman ui-goal-page/goal-acts/foreman-1)

rebase words.rs by combining both segment-count increases
The merge conflicts in `temper/crates/temper-client-view/src/words.rs`.
Main commit `789b2ee91d` (“client: preserve the charter in check-reserve refusals”), incorporated through integration merge `1e39271882`, adds two segments for `AgentChecks`.
The task adds eight segments for its six goal-action phrases.
Resolve `SEGMENTS` to `2593` (base `2583` + main’s `2` + task’s `8`).
Keep the task’s `PHRASES = 1492`, `ALL` length `896`, six new variants, phrase IDs `3432..3437`, and English phrases.
Preserve main’s revised `AgentChecks` wording, its `Number` hole, and the `+ 53` text-byte allowance in `Words::english`.
Run the spec’s required checks through heavy after rebasing; the refusal occurred during merge and provides no test results.
