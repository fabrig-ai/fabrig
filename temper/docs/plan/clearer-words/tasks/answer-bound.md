+++
id = "answer-bound"
part = "answer-bound"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-tools/src/jig/render/outcome.rs", "temper/crates/temper-client-tools/src/jig/runner/acts.rs", "temper/crates/temper-client-tools/src/jig/rows.rs", "temper/crates/temper-client-tools/src/tests/jig/acts.rs", "temper/crates/temper-client-tools/src/tests/help.rs", "temper/docs/help/reference/ctl-task.md", "temper/tests/client/tests/suite/person_task_tools.rs", "temper/docs/design/client/tools.status.md"]
effort = "high"
minutes = 60
+++

# `task answer`'s refusal names the bound the person task asks for

## Goal

`temper ctl task answer` refuses a reason past a person task's contract with a bare `Limit reached`, while its help says TEXT is at most 1048576 bytes. The refusal comes to name the task's bound, and the help comes to say a person task may ask for less. The engine, the wire and the exit status do not change: `task answer`'s form with `--rev` already reads the person task before it asks, so the contract is in hand when the refusal comes.

Acceptance:

- A choice's reason past that choice's `words`, refused `Limit { Admission { Contract } }`, prints exactly `task answer: Limit reached: task 41's reason for choice 1 is at most 4 bytes`, status 2.
- Words past a report's `words`, refused the same way, print exactly `task answer: Limit reached: task 41's words are at most 4 bytes`, status 2.
- With `--json`, either carries `"bound":4` beside the refusal, as `read_again` sits beside one.
- The same refusal prints the bare `task answer: Limit reached` wherever the text is not past a bound the read contract gives: text within the bound, a choice the contract lacks or that it cut, an answer of another form than its contract, `--cannot`, `--approve`, and a restored request. So does any other `Limit`.
- `temper ctl help task answer` and `temper/docs/help/reference/ctl-task.md` say a person task may ask for less.
- Against the real engine, a reason one byte past the choice is refused with its bound, status 2, and the task stays unanswered.
- The **Not yet** entry on `task answer`'s refusal in `temper/docs/design/client/tools.status.md` is gone.

## Inputs

- `temper/docs/plan/clearer-words/notes/common.md`.
- `temper/docs/design/client/tools.md`, 5.5: area 3's last paragraph on a person answer, which this cut wrote; 6.3, exit 2.
- Why `Contract` alone is not enough: `jig/crates/jig-domain-tasks/src/run.rs`, `invalid_result`, which reports a wrong form, an unknown choice and text past the bound alike; `jig/crates/jig-domain-core/src/routing/refusals.rs` maps them all to `LimitKind::Contract`. Read only.

## Owns and may change

Owns: `temper-client-tools/src/jig/render/outcome.rs`, `src/jig/runner/acts.rs`; `ANSWER`'s `notes` in `src/jig/rows.rs`, nothing else there; the new tests in `src/tests/jig/acts.rs` and `src/tests/help.rs`; `temper/tests/client/tests/suite/person_task_tools.rs`, one new story; `temper/docs/help/reference/ctl-task.md`, regenerated; the one entry in `tools.status.md`.

May also change, and how:

- `temper-client-tools/src/tests/fixtures/acts.rs`: a helper that serves a person document with a given contract, if the test needs one.
- `temper-client-tools/src/tests/golden/*`: only if the help's golden test shows `ANSWER`'s notes in one, regenerated.

Options, each pre-approved:

- `context` in `acts.rs` returns a tuple that grows to four: a small named struct, or a scoped `#[expect(clippy::type_complexity, reason = "...")]`, is your choice.
- If the printed prefix differs from `task answer: ` for a reason of the existing `refused_text`, follow what `refused_text` prints for every other tool, and say so in the report. The words after `Limit reached` stand.
- If the joined story's call must take the task first or read a new revision, do as the file's `answer_choice` does.
- Anything that needs jig, the wire or the store is outside this task: stop and ask.

## Steps

1. **`render/outcome.rs`.**

   ```rust
   /// What a person task's contract asks of an answer past it (temper/docs/design/client/tools.md, 5.5).
   #[derive(Clone, Copy, Debug)]
   pub(crate) struct Asked {
       pub bound: u32,
       /// The choice the reason is for; absent for a report's words.
       pub choice: Option<u32>,
   }
   ```

   `Operation` gains `pub asked: Option<Asked>`, `None` from `Operation::of`, so that a restored request has none. A private helper gives the `Asked` only for `c::Refusal::Limit { which: c::Limit::Admission { which: c::AdmissionLimit::Contract } }`. `refused_text`'s general path, after its `acts::refusal(...)`, adds `: task T's reason for choice C is at most B bytes` or `: task T's words are at most B bytes`, T from `operation.task`. `refused_json` adds the key `bound`.
2. **`runner/acts.rs`.** `Request::Act` gains `asked: Option<render::outcome::Asked>`, set in `build` and `None` in `close_request`. `context` carries it, and `send` sets `operation.asked`, beside `operation.ended_task`. `build` gets it from a new private function:

   ```rust
   /// Past what the read contract asks, the refusal names the bound (client/tools.md, 5.5).
   fn asked(call: &Call, document: &c::ReadValue) -> Option<render::outcome::Asked>
   ```

   It answers `None` unless `call.tool == rows::ANSWER_AT` and the document is `c::ReadValue::PersonTask`. Form 1 finds the choice whose `code` is the call's CHOICE in `c::TaskContractDocument::Verdict`, its TEXT at index 3. Form 2 takes `Report { words }`, its TEXT at index 2. Other forms and contracts answer `None`. It answers `Some` only where the text's length is past the bound, with `skein_lib::length::usize_of`.
3. **`rows.rs`.** `ANSWER`'s `notes` gain, after their last line:

   ```
   TEXT's bound is the most any call takes; a person task may ask for less:
   task show prints what it asks, and a refusal past it names the bound.
   ```
4. **Regenerate** `ctl-task.md`, then run the test again without the variable:

   ```
   SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-tools --lib -E 'test(the_task_chat_and_inbox_references_are_what_the_table_renders)'
   heavy cargo nextest run -p temper-client-tools --lib -E 'test(the_task_chat_and_inbox_references_are_what_the_table_renders)'
   ```

   Its diff is those two lines.
5. **The step tests,** in `src/tests/jig/acts.rs`, on the shape of `dependency_refusals_print_jigs_words_and_integer_facts`: `fixtures::signed_in`, `root.submit`, `root.serve` of a `c::ReadValue::PersonTask` from `fixtures::person_document()` with its `contract` set, then `root.answered` with the refusal, then `root.answer`.

   ```rust
   #[test]
   fn a_person_answer_past_its_contract_names_the_bound() {
       // Verdict { choices: [code 1, words 4] }, ["task","answer","41","--rev","7","1","blue!"]: the choice's line, status 2.
       // Report { words: 4 }, ["task","answer","41","--rev","7","words"]: the words' line, status 2.
       // Each with --json: "bound":4.
   }
   #[test]
   fn a_refusal_with_no_bound_the_task_asks_for_stays_bare() {
       // Each prints exactly "task answer: Limit reached\n", no "bound" under --json:
       // text within the bound; choice 2 of a contract with only code 1; words against a Verdict;
       // a reason against a Report; --cannot and --approve past nothing the contract names;
       // text past the bound under another Limit, c::Limit::Inbox.
   }
   ```

   And in `src/tests/help.rs`, a test that `temper_docs::find(b"ctl-task")`'s text, white space collapsed, holds `a person task may ask for less`.
6. **The joined story,** in `person_task_tools.rs`, beside `a_person_answers_a_delegated_choice_and_wakes_its_goal`, since only the real engine shows that this refusal is `Contract`:

   ```rust
   #[test]
   fn a_reason_past_its_choice_names_the_bound() {
       // start_goal, delegate, take it as answer_choice does.
       // A reason of RESULT_WORDS + 1 bytes: status 2, stderr exactly
       //   "task answer: Limit reached: task {child}'s reason for choice 1 is at most 128 bytes\n",
       //   and person_answer stays None.
       // A reason of RESULT_WORDS bytes is then taken.
   }
   ```
7. **The status.** Delete the **Not yet** entry on `task answer`'s refusal from `tools.status.md`.
8. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p temper-client-tools -p temper-client-world --all-targets -- -D warnings`.
9. Tests, one command at a time: `heavy cargo nextest run -p temper-client-tools`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(person_task_tools)'`; `heavy cargo nextest run -p temper --lib -E 'test(every_ctl_group_page_is_what_the_table_renders)'`, since `temper` also holds the group references to their rendering. A departure from the implementer's text: nothing public changes, so the other dependents wait for the trunk's end check.
10. Commit, say why, and push.

## Report

As the implementer's text asks. Name the rule each new test was seen to catch, and the joined story's time beside its tier's budget if it nears it.
