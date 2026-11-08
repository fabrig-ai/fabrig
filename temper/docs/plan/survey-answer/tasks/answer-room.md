+++
id = "answer-room"
part = "answer-room"
kind = "feature"
needs = []
files = ["smith/crates/smith-domain-run/src/limits.rs", "smith/crates/smith-domain-run/src/agent.rs", "smith/crates/smith-domain-run/src/tests/agents.rs", "smith/crates/smith-agent-service/src/profile/tests.rs", "smith/tests/agent-process/src/world/peer.rs", "smith/tests/agent-process/src/world.rs", "smith/tests/agent-process/tests/suite/fact_drain.rs"]
effort = "high"
minutes = 100
+++

# survey-answer: a sub-agent's answer has the outcome's room

## Goal

A goal's read-only survey through `sub_agent` came back as 1,024 bytes and
`[child-result cut=23 stop=end-turn]` on a deployment whose `tool_payload` is 63,740. The
cap is `smith-domain-run`'s `answer_bytes`, fixed at 1,024 in `limits::derive` whatever
the payload; no profile of smith, jig or temper reaches it, and the child is told it in
its share, so it writes to it. A child's answer is its run's result to its asker, so it
now has the outcome's room, the larger of 4,096 and `tool_payload`
(`smith/docs/design/protocol/limits.md`, 3.10), and a cut falls at a character's
boundary (`smith/docs/design/domain/run.md`, 5.3).

Acceptance:

- `notes/contracts.md`, sections 1 and 2, hold.
- **The failing story first:** in the agent's process world, whose limits are derived
  from its profile (`smith_agent_process_world::limits`, payload 256), a real delegated
  child that answers 3,000 bytes reaches main whole: the result main's next request
  carries for the `sub_agent` call starts with the whole answer, holds no
  `[child-result`, and the call's `ToolCompleted` has `omitted_bytes` 0. Before the
  change it fails, kept at 1,024 with `[child-result cut=1976 stop=end-turn]`; quote it.
- A row of the profile table: for the stock's pair (payload 10,662) and the reference's
  (63,740), `run.answer_bytes` equals `run.outcome_bytes`. It fails before (1,024).
- A row of the child-answer test: a child's answer whose cap falls inside a two-byte
  character keeps the bytes before that character and counts the rest as cut.
- Each bites: restore `answer_bytes: 1024` and see the story and the table row fail;
  drop the boundary step and see the character row fail; restore.
- Delete, as stated departures from the implementer's text (`notes/common.md`), with no
  other change there: the bullet `**Not yet:** 3.10: a sub-agent's answer stops at 1,024
  bytes ...` from `smith/docs/design/protocol/limits.status.md`; the first bullet
  `**Not yet:** 5.3: a sub-agent's answer cap is 1,024 bytes ...` from
  `smith/docs/design/domain/run.status.md`; and from `smith/docs/design/STATUS.md` the
  words `a sub-agent's answer cap of 1,024 bytes and its cut at a byte (`domain/run.md`,
  `protocol/limits.md`), ` in its first paragraph, rewrapping only those lines. The
  task `cut-words` deletes the bullets and paragraph about the marker's words beside
  them: leave those.

## Inputs

- `temper/docs/plan/survey-answer/notes/common.md`; `notes/contracts.md`, sections 1
  and 2.
- `smith/docs/design/protocol/limits.md`, 3.10 (the sub-agent's answer), 4
  (`render-in-result`), 6 (the row for an answer past `answer_bytes`), 11 ("No pinned
  values").
- `smith/docs/design/domain/run.md`, 5.3 ("The result." and "A cut answer").
- `smith/AGENTS.md`: production domains are step code.

## Owns and may change

- Owns the files in the header, and the three status files' lines named above.
- `smith-domain-run/src/limits.rs`: `derive` sets `answer_bytes` as section 1 says; the
  field's comment says it follows `tool_payload` as the outcome does. Nothing else of
  the derivation changes.
- `smith-domain-run/src/agent.rs`: `yielded`'s cut, as section 2 says, in step code (no
  `while`, `loop`, closure or indexing). Matching on `core::str::from_utf8` of the kept
  head and its `Utf8Error` (`error_len() == None` is a character cut short at the end;
  `valid_up_to()` is where it starts) is one way; a bounded `for` over the last three
  bytes is another. Yours.
- `smith-domain-run/src/tests/agents.rs`:
  `a_sub_agent_answers_with_its_last_message_once_it_has_ended` becomes a table over
  `(yielded, kept, cut)`, its rows `(b"The parser is in src/parse.rs.", b"The parser is
  in", 14)` and `("The parser is aé in src.".as_bytes(), b"The parser is a", 10)` under
  the harness's 16-byte cap; its other assertions hold for each row.
- `smith-agent-service/src/profile/tests.rs`, `forty_eight_calls_per_response_derive_with_two_models`:
  each `Ok` row also asserts `u64::from(limits.domain.run.answer_bytes) ==
  limits.domain.run.outcome_bytes`. The pinned T figures stay; if one moves, that is a
  red to explain, not a figure to rewrite.
- The agent's process world:
  - `src/world/peer.rs`: a `TraceCase::LongChildAnswered` variant, "a real delegated
    child answers 3,000 bytes before main's accepted finish", whose child peer says
    that text where `ChildAnswered`'s says `child answer`; `child_peer` takes the text.
    The world exports the text (a `pub const` or `pub fn` beside `TraceCase`, re-exported
    from `lib.rs` with a one-line `pub use`), so the test reads it and holds no second
    copy. ASCII, ending in a mark of its own (such as `END-OF-SURVEY`), so a cut shows.
  - `src/world.rs`: the checks that select `ChildAnswered`'s charter and query count
    (`observe_facts`, `maximum_queries`) select `LongChildAnswered` too.
  - `tests/suite/fact_drain.rs`: the story
    `a_childs_answer_of_three_thousand_bytes_reaches_main_whole`, beside
    `a_child_usage_line_tells_the_budget_record_naming_its_call` and shaped like it:
    `World::new(19, &charter())`, `capture(Capture::Everything)`,
    `observe_facts(TraceCase::LongChildAnswered, 4)`, `settle()`, an accepted answer;
    the child's `ConversationOpened` and its call; `child_usage(&world, parent, call)`
    for the result text; the assertions of the acceptance. No `check_memory` call.
- Pre-approved:
  - If 3,000 bytes meets another of the world's bounds before the answer cap (one
    completion's text is 4,096 bytes at its 512-token output), use a length between 2,048
    and 3,500 and name the bound in the report.
  - If the variant forces arms or `==` checks elsewhere in the world crate, add the
    variant there; if it needs more than two further sites, a `World` setter for the
    child's text in its place is yours, said in the report.
  - A reader's test that asserted a child answer cut at 1,024 now sees it whole: make
    the row's answer pass its world's cap again, in the reader's test file only, and name
    it. None is known.
  - A measured memory ceiling, as `notes/common.md` says.
- Not yours: `smith-domain/src/feedback.rs` and `smith/tests/agent/tests/suite/feedback.rs`
  (the task `cut-words`), `smith-domain-run/src/prompt.rs` (the share's words stay), the
  hand-written limits of `smith/tests/agent/src/limits.rs` and the lowering in
  `smith/tests/agent-process/src/program_world.rs` and
  `jig/testing/jig-channel-kit/src/program_runs.rs`, which keep their figures.

## Steps

1. The story, the table row and the character row; run each by name and see it fail for
   the rule's reason.
2. The derivation and the cut; the three pass.
3. Delete the status lines.
4. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p
   smith-domain-run -p smith-agent-service -p smith-agent-process-world --all-targets --
   -D warnings`.
5. This changes what a shared limit means, so its readers' focused tests run: `heavy
   cargo nextest run -p smith-domain-run -p smith-domain -p smith-agent-service -p
   smith-agent-shell -p smith-local-service -p smith-local-shell -p
   smith-agent-process-world -p smith-protocol-llm-world -p smith-agent-world`; then
   `heavy cargo nextest run -p jig-channel-world --test suite -E 'test(/program_runs/)'`;
   then the derivation's property test, `heavy cargo nextest run -p
   smith-agent-process-world --profile fuzzy -E 'test(/fuzzy::profile/)'`. Check each
   count.
6. `python3 -I -B tools/citations/citations.py --base origin/main`.
7. Commit (why: a survey a run asked through `sub_agent` was cut at a fixed 1,024 bytes
   on a 63,740-byte deployment); push your branch.

## Report

As the implementer's text says, with the failures quoted from step 1 and the CPU time of
the new story beside `a_child_usage_line_tells_the_budget_record_naming_its_call`'s,
measured in one run.
