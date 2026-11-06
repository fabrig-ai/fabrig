+++
id = "journey-goes-on"
part = "tab-root"
kind = "feature"
needs = ["sign-in-holds"]
files = ["temper/crates/temper-client-shell-web/tests/suite/host.rs", "temper/crates/temper-client-shell-web/tests/suite/host/walk.rs", "temper/crates/temper-client-shell-web/src/journey/tests.rs"]
effort = "high"
minutes = 40
+++

## Goal

A recording goes on through a sign-in and its end, keeping what the shell did with the tab, and the replay checks it. The host's walk, which ends on a sign-in's departure today, walks through instead.

Acceptance:

- `tests/suite/host/walk.rs`'s walk presses the sign-in, takes the answer, sees the tab's lines, goes on to the sign-in's end through the `accounts` watch, and ends on a mark or a save, with no `departs()`.
- A recording of that walk replays. `a_departure_not_recorded_is_stale` is rewritten: `a_tab_not_recorded_is_stale`, one for `tab-open` and one for `tab-point`, each naming its turn and line.
- A tab refused, as `tab-refused`, replays, with no `tab-point` after it.
- A departure still ends a recording: a test on a forge's or an authorization's `navigate` or `replace`, keeping `Duties::departs()`'s rule alive. If `src/journey/tests.rs` holds that already, name it in the report and don't add one.
- `a_save_is_whole_and_the_recording_goes_on`, `a_recorded_walk_proves_and_replays_its_marks`, `a_proved_recording_keeps_no_html`, `a_changed_ask_is_stale_at_its_line`, `a_changed_deadline_is_stale_at_its_line`, `an_event_no_control_gives_is_stale_at_its_act`, `the_proof_refuses_a_mark_whose_html_differs`, `a_version_1_recording_is_unreadable`, `a_renamed_control_holds`, `an_act_with_no_event_keeps_nothing` and `a_saves_control_is_found_by_its_event` pass on the new walk.
- Remove the **Not yet** line from `temper/docs/design/client/journeys.status.md`. This task may make that one edit, against the implementer's text.

## Inputs

- `temper/docs/plan/sign-in-new-tab/notes/contracts.md`, *The recording*.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.
- `temper/docs/design/client/journeys.md`, 3.1 and 4.2.

## Owns and may change

- Owns the files in the header. The draft `real-journeys` is changing `tests/suite/host.rs` and `host/walk.rs`, and adds `host/redacted.rs` with `real-walk`: if your base has them, take its shape and keep its tests passing. If `real-walk` hasn't landed, your hunks go on the files as they stand, and the merge is fab's.
- If a real-journeys `redact` test now holds a recording with a sign-in's address in `tab-point`, that address is among what it replaces, as `navigate`'s is: `fields.rs` holds it by the `tab-surface` task's arm.
- No recording under `temper/docs/design/client/journeys/` holds a sign-in, so none moves, and none of them is yours.

## Steps

1. The walk in `host/walk.rs`: at the answer, drop the `Return` exchange and its answer, and go on as the sketch in the acceptance has it.
2. The stale tests, failing first; break `asked` by hand once (skip `Duties::tab`), see `a_tab_not_recorded_is_stale` fail, restore.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-web --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-shell-web`; `heavy cargo nextest run -p temper-client-journeys`. Commit and push.

## Report

As the implementer's text asks.
