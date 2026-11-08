# tui-record-keys: plan

A small repair from the user's own recording, 2026-11-08. Their terminal dropped the Shift from Alt+Shift+M and Alt+Shift+S, so the TUI's `record` build got `alt "m"` and `alt "s"`, and nothing was marked or saved until `q`. The record build now also takes Alt+m as the mark key and Alt+s as the save key. The shipped TUI, the walk's steps and the committed journey stay as they are (`temper/docs/plan/tui-record-keys/inputs/request.md`).

## Parts and tasks

- **keys** (direct, slice 1, no walk): task `keys` (`tasks/keys.md`), high effort, about 45 minutes, needing nothing. It changes two arms of `Run::record_key`'s match (`temper/crates/temper-client-shell-tui/src/run.rs`) and adds test rows:
  - the run suite's recorder key test, `run::recording_keys_save_whole_and_closing_saves_again`, runs its walk once with the Shift forms and once with Alt+m and Alt+s;
  - the run's unit test `run::tests::recording_keys_pass_to_the_host_without_a_root` gains `Alt('m')` and `Alt('s')`: a run with no recording, the shipped binary's, gives them to the host;
  - it removes the status entry the plan opened.

## Critical path

`keys` alone. It starts at the first push, and the plan lands with it.

## What was decided

- **One task, as the request asks.** The change is two match arms and their test rows.
- **The design, written by the lead:** `temper/docs/design/client/tui/journeys.md`.
  - 3.1 gains the bullet *Each key in two forms*: Alt+m and Alt+s, why they exist, and that a run with no recording gives them to the host.
  - 3.3 says the keys never reach the host in either form.
  - 8.1's table says `mark` and `end` press the Shift forms.
  - `journeys.status.md` has a **Not yet** entry for 3.1 and 3.3, which the task removes. The spec states that departure from the implementer's text.
  - The request also cites section 2. Section 2 names no key, so the edit is in 3.1, where the keys are named.
- **The shipped build's row is the run's unit test with `record: None`.** The shipped binary gives the run `record: None` (`temper/crates/temper/src/tui.rs`). In that build `record_key` isn't compiled at all. So the place the shipped path can be checked is the run with no recording. A key the host has no event for changes no frame, so nothing else in the run's world could show that Alt+m reached the host.
- **The record build's rows are one row of key forms each.** The existing test isn't a table. It becomes one with two rows, `(\x1bM, \x1bS)` and `(\x1bm, \x1bs)`, each on a fresh world with the walk unchanged. One extra run of a simulated world is the whole cost.
- **No `notes/common.md` or `notes/contracts.md`.** One task, and nothing crosses a part. The request's `For every task of this plan` section serves as the common note.
- **What stays:** `again` (`temper/tools/journeys`) keeps pressing the Shift forms. The committed `sign-in-approved` holds no `key alt` record, so nothing is recorded again. The browser's journeys keep their keys. A walk's `key alt "m"` step would now be taken as a mark in a recording, and `again`'s check would refuse it. No walk has one, and the design doesn't add a rule for it.

## Overlaps

- **`tui-link-colours`** is being cut beside this plan. Its trunk holds only its inputs so far. Its request has it:
  - keep the scheme among a recording's start facts, and keep an OSC 11 answer from being read as a key. That probably touches `temper/crates/temper-client-shell-tui/src/run.rs` and `tests/suite/run.rs`;
  - extend `journeys.md` for the colours' prints;
  - record `sign-in-approved` again.

  These overlaps couldn't be avoided: the keys live in `run.rs`, and the design names them in `journeys.md`. To keep merges small, this plan's edits are only:
  - `record_key`'s two arms and the unit test's key list;
  - the suite's `replay` refusal and its one key test;
  - three bullets and two table rows of `journeys.md` (3.1, 3.3 and 8.1), with its date;
  - the status file's date line. The task leaves the status paragraph as it is on `main`, line for line.
- The other drafts beside this one name no file this task owns.
