+++
id = "relay-policy"
part = "relay"
kind = "feature"
needs = ["relay-types"]
files = ["temper/crates/temper-engine-domain-forge-change/src/relay.rs", "temper/crates/temper-engine-domain-forge-change/src/lib.rs", "temper/crates/temper-engine-domain-forge-change/src/tests/relay.rs", "temper/crates/temper-engine-domain-forge-change/src/tests.rs"]
effort = "high"
minutes = 70
+++

# relay-policy: the relay's step, decided purely

## Goal

`temper_engine_domain_forge_change::relay` decides, from a change's `Questions`, what one
step heard and the change's bound, what the change does with its open questions (contract
C7): take new ones, hold past the bound, delegate a foreman, answer the asker, pass a
question up, hold an answer for dependencies, drop what an ended asker left. It is pure, in
the crate beside `step`, and tested alone; task `relay-up` applies it.

Acceptance, `relay(questions, heard, limits, room)`, the rules in order:

1. **Ended askers.** Each open question whose asker is in `heard.ended` leaves `open`; one
   `Foreman { task: Some(f) }` adds `Cancel { task: f }`. An `Asked` one stays (the change's
   own question is still out) until the requester's answer, which is then dropped with no
   act.
2. **Taken.** Each of `heard.asked`, oldest first, joins the end of `open` as `Taken`, its
   `line` the words' first line (up to the first `\n`) fitted to `QUESTION_LINE` bytes on a
   UTF-8 boundary, cut and marked with `…` within that bound.
3. **The head.** The first open question is worked, then the next once it leaves, until one
   waits or `room` runs out:
   - `Taken`: when `taken` equals `limits.questions` and the change was not just released
     (`heard.released` false), act `Hold` once and stop; otherwise `taken` counts one more
     (released: `taken` restarts at 1), and route `Foreman` (and not released past the
     bound) acts `Foreman { asker, number }`, the head becoming `Foreman { task: None }`;
     route `Up`, or a release past the bound, acts `Ask { asker, number, reasons: [] }`, the
     head becoming `Asked`.
   - `Foreman { .. }` with `heard.foreman` set: `Answer(t)` acts `Answer { words: t }` and
     the head leaves; `Finish(t)` acts `Answer` with `Finish: the work done holds; list the
     rest under Left.\n` then `t`, and the head leaves; `Escalate(t)` or `Failed(t)` acts
     `Ask { reasons: "Foreman:\n" then t }`, the head becoming `Asked`. Without
     `heard.foreman`, it waits.
   - `Asked` with `heard.answer` set: `heard.kept` makes it `Held { answer }` and it waits;
     otherwise `Answer { words: answer }` and the head leaves. Without an answer, it waits.
   - `Held { answer }`: `heard.kept` false acts `Answer { words: answer }` and the head
     leaves; otherwise it waits.
   - Each `Answer` and `Ask` takes one of `room`; with none left, `more` is set and the step
     stops where it stands. `Foreman`, `Cancel` and `Hold` take no room.
4. `route` `None`: `relay` returns `questions` unchanged and no act (the root never takes a
   question for such a change).
5. `open` never passes `QUESTIONS_OPEN`; a `heard.asked` past it is left to the root
   (contract C7).

## Inputs

- `temper/docs/design/domain/forge.md`, 8.11 (whole) and 8.4 ("A release lifts what it
  releases").
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C1, C2 and C7.
- This task edits no design file.

## Owns and may change

Owns the files in the header: the new module `relay.rs`, its export in `lib.rs` (C7's
types and `relay`), and its tests in `src/tests/relay.rs`, declared from `src/tests.rs` as
the crate's other test modules are (`turn.rs`, `verifying.rs`).

- `relay` takes and returns owned values as C7 has them; helpers inside the module are
  yours. If C7's types need a derive to be compared in tests (`PartialEq`, `Debug`), add it.
- No `domain.rs` change: `relay` stands beside `step` and reads none of its state.

## Steps

1. `relay.rs` and its export.
2. Tests in `src/tests/relay.rs`, one per rule, each named for what it shows:
   `a_new_question_waits_its_turn_behind_the_head`;
   `the_head_gets_a_foreman_under_foreman`; `the_head_goes_up_under_up`;
   `a_foremans_answer_answers_the_asker_and_the_next_is_worked`;
   `a_foremans_finish_opens_with_the_finish_line`;
   `a_foremans_escalation_goes_up_with_its_reasons`; `a_failed_foreman_goes_up_too`;
   `the_requesters_answer_comes_down`; `an_answer_while_kept_is_held_then_relayed`;
   `past_the_bound_the_head_holds_once`; `released_past_the_bound_it_goes_up_without_a_foreman`;
   `an_ended_asker_cancels_its_foreman_and_drops_its_questions`;
   `an_asked_question_of_an_ended_asker_drops_its_answer`;
   `no_room_leaves_more_for_the_next_step`; `a_long_first_line_is_cut_and_marked`;
   `route_none_relays_nothing`.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge-change -E
   'test(/relay/)'` (check the count), then the crate's whole suite.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge-change --all-targets -- -D warnings`. Commit ("Decide a
   change's relay of questions") and push.

## Report

As the implementer's text says.
