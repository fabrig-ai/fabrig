+++
id = "cut-words"
part = "cut-words"
kind = "feature"
needs = []
files = ["smith/crates/smith-domain/src/feedback.rs", "smith/tests/agent/tests/suite/feedback.rs"]
effort = "high"
minutes = 60
+++

# survey-answer: a cut sub-agent answer says how to read the rest

## Goal

A lead that got a sub-agent's answer cut short read `[child-result cut=23
stop=end-turn]` and nothing more, so it re-read the files itself and paid for the
survey twice. A child has closed when it answers, so nothing continues it; a cut answer's
marker now says how many bytes were left out and that a new sub-agent, asked narrower,
reads them (`smith/docs/design/domain/run.md`, 5.3; `smith/docs/design/protocol/limits.md`,
6). The feedback bound holds the longer marker. The cap itself is the task
`answer-room`'s; this task doesn't need it.

Acceptance:

- `notes/contracts.md`, sections 3 and 4, hold: `render_child` writes the words exactly
  as section 3 gives them, and `feedback_bound`'s child term measures the marker at its
  maxima in place of the fixed 128 bytes.
- Rows of the existing tests in `smith/tests/agent/tests/suite/feedback.rs`
  (`smith-agent-world`, binary `suite`):
  - `child_cut_and_every_stop_survive_canonical_feedback`: for `cut` `u64::MAX` and
    every stop, the expected text is the bracket then the words with
    `18446744073709551615`; for `cut` 0 every row stays as today.
  - `child_usage_has_exact_scalar_values_flooring_and_complete_cap`: the `cut: 14`
    answer's expected text gains the words with `14`; its exact cap passes and one byte
    less is refused, as today.
  - `largest_usage_suffix_and_answer_fit_the_declared_feedback_bound`: its answer cap
    rises from 8,192 to 1,048,576 bytes (`1 << 20`), so that the child's term is the
    bound's largest; at 8,192 the outcome's four-fold term, past 262,144 under `TIGHT`,
    hides any marker. The largest answer (that many bytes, `cut` `u64::MAX`, stop
    `max-tokens`) still fits the bound, and one byte less than its measure is refused.
    With the words, this row fails while the bound keeps 128 bytes, and passes once
    section 4 holds.
  The first two fail before the words change, the third once they have changed and
  before the bound does; quote each.
- It bites: write the words without the cut's count and see the first two rows fail;
  put 128 back in the bound and see the third fail; restore.
- Delete, as stated departures from the implementer's text (`notes/common.md`), with no
  other change there: the bullet `**Not yet:** 6: a sub-agent's cut answer is marked
  ...` from `smith/docs/design/protocol/limits.status.md`; the second bullet `**Not
  yet:** 5.3: a cut answer's marker is the bracket alone ...` from
  `smith/docs/design/domain/run.status.md`; and from `smith/docs/design/STATUS.md` the
  paragraph `A sub-agent's cut answer is marked without words on how to read the rest
  (...).` with the blank line before it. The task `answer-room` deletes the bullets and
  the words about the cap beside them: leave those.

## Inputs

- `temper/docs/plan/survey-answer/notes/common.md`; `notes/contracts.md`, sections 3
  and 4.
- `smith/docs/design/domain/run.md`, 5.3 ("A cut answer": the words as the design gives
  them); `smith/docs/design/protocol/limits.md`, 3.10 (the answer's feedback within the
  sub-agent's feedback) and 6 (the row for an answer past `answer_bytes`).
- `smith/AGENTS.md`: production domains are step code.

## Owns and may change

- Owns the files in the header, and the three status files' lines named above.
- `smith-domain/src/feedback.rs`: `render_child` and `feedback_bound`'s child term. A
  helper that writes the marker, called by both `render_child` and the bound's measure so
  the two can't drift, is yours. Nothing else of the file changes: every other result's
  words stay byte for byte.
- Pre-approved:
  - If a test elsewhere in smith compares a cut child's whole feedback text and now
    fails on the words, update its expected text to section 3's in that test file only,
    and name it. None is known (`[child-result` is spelled only in these two files).
  - If `feedback_bound` moves for a world's hand-written limits and a figure pinned from
    it fails, that test reads the bound by its function rather than a number, or it is a
    red to report: say which.
  - A measured memory ceiling, as `notes/common.md` says.
- Not yours: `smith-domain-run` (the task `answer-room` owns it this wave: the cap, the
  cut's place and the share's words), the program backend's MCP text
  (`smith-protocol-program`), which carries the feedback as it is given.

## Steps

1. The rows; run `heavy cargo nextest run -p smith-agent-world --test suite -E
   'test(/feedback::/)'` and see the first two fail for the words.
2. The words; the first two pass and the third fails on the bound. Then the bound; the
   three pass.
3. Delete the status lines.
4. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p
   smith-domain -p smith-agent-world --all-targets -- -D warnings`.
5. The words reach every delegation's feedback, so its readers' focused tests run: `heavy
   cargo nextest run -p smith-domain -p smith-agent-world -p smith-agent-process-world
   -p smith-local-service`. Check the count.
6. `python3 -I -B tools/citations/citations.py --base origin/main`.
7. Commit (why: a lead given a cut sub-agent answer heard only `cut=23` and read the
   files again itself); push your branch.

## Report

As the implementer's text says, with the failures quoted from step 1.
