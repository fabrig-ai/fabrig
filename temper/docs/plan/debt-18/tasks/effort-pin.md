+++
id = "effort-pin"
part = "smith-review"
kind = "feature"
needs = []
files = ["smith/crates/smith-local-domain/src/parent_tests.rs", "smith/crates/smith-local-protocol/src/terminal.rs", "smith/crates/smith-local-service/src/tests.rs", "smith/crates/smith-local-service/src/service.rs"]
effort = "high"
minutes = 25
+++

# smith: the effort rule gets back a test that can fail on it

## Goal

`temper/docs/plan/debt-18/inputs/smith-debt14-review.md`, finding 1. The rule that a run refused for its effort tells the
person the choices lives in `smith-local-domain/src/person.rs` (the `host::Refusal::Invalid(host::RunInvalid::Effort)`
arm of `finish_answered`). No test names `RunInvalid::Effort`: the one test near it, in the service, injects a ready
`Failure::EffortRefused` through a `#[cfg(test)]` seam. The cut re-read this: the finding holds.

Acceptance:

- A step test in `parent_tests.rs` asserts `Failure::EffortRefused(ChoiceNotice::Effort { .. })` for a host refusal of
  `RunInvalid::Effort`, with the effort and the choices the model declares, and goes red when the arm in `person.rs` is
  removed. `rg -c 'RunInvalid::Effort' smith/crates/smith-local-domain/src/parent_tests.rs` reads at least 1.
- A case in `smith-local-protocol/src/terminal.rs`'s tests pins that the rendered refusal ends with no newline, and goes
  red when the `local::Failure::EffortRefused` arm of `notice_text` stops returning `choice_text` directly.
- `a_refused_run_effort_writes_the_choice_without_a_newline` and `Service::render_notice` are gone:
  `rg -c 'render_notice' smith -g '*.rs'` finds nothing.
- The branch's net lines don't rise (the review's estimate is -5). Every other test passes unchanged.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/inputs/smith-debt14-review.md`, finding 1

## Owns and may change

- **Owns:** the new test in `smith-local-domain/src/parent_tests.rs`, the new case in the `tests` module of
  `smith-local-protocol/src/terminal.rs`; the removal of the test from `smith-local-service/src/tests.rs` and of
  `render_notice` from `smith-local-service/src/service.rs`, with any import that leaves unused. Nothing else in these
  files: a later task of this plan changes them further.
- If the step test can't reach the arm through `parent_request`, drive it as `gone_after_a_late_turn_save_shows_the_answer_once`
  does (`Answered`, then `crate::fixture::gone(..)`, and any save it waits for).

## Steps

1. The step test, in the form of its neighbours (`running`, `parent_request`):

   ```rust
   #[test]
   fn a_run_refused_for_its_effort_shows_the_effort_and_the_choices() {
       let mut config = crate::tests::config();
       config.models[0].effort = Some(b"high".as_slice().into());
       config.efforts[0] = Box::new([b"high".as_slice().into()]);
       let (mut domain, env, mut out) = running(config);
       parent_request(&mut domain, &env, parent::Request::Answered {
           client: Token::new(1),
           answer: host::Answer { read: None, turns: 0, spent: 0, result: host::RunResult::Refused {
               refusal: host::Refusal::Invalid(host::RunInvalid::Effort),
           } },
           detail: Box::default(),
       }, &mut out);
       // ...then pop to the notice, and assert the whole value:
       // Notice::Failure(Failure::EffortRefused(ChoiceNotice::Effort { effort: b"high", choices: [b"high"], dropped: false }))
   }
   ```

2. The terminal case: `notice_text(&local::Notice::Failure(local::Failure::EffortRefused(effort)), b"main", &[], 1024)`
   equals `b"Effort mystery is not declared; accepted: low, high"`, whole, next to
   `local_choice_words_keep_their_terminal_byte_contract`.

3. Delete the service's test and the seam.

4. Break each rule by hand and restore: remove the `person.rs` arm (the step test goes red); make the terminal's arm append
   a newline (the terminal case goes red).

5. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-local-domain -p smith-local-protocol
   -p smith-local-service --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-local-domain -p smith-local-protocol
   -p smith-local-service`; the citations check. Commit and push.

## Report

The common note's items, what you broke by hand and what failed.
