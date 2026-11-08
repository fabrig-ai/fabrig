# survey-answer: the plan

A read-only survey a goal's run asked through `sub_agent` came back as exactly 1,024
bytes, then `[child-result cut=23 stop=end-turn]`, on a deployment whose `tool_payload`
is 63,740.

## What cut it

`smith-domain-run`'s `Limits::answer_bytes`, "the most bytes of a sub-agent's last
message its asker is given", is fixed at 1,024 in `limits::derive`
(`smith/crates/smith-domain-run/src/limits.rs`), whatever the payload. No profile reaches
it: jig's and temper's profiles feed smith's `declared.tool_payload` (63,740 on the
reference), which sizes `run_bytes` and `outcome_bytes` but never `answer_bytes`. The
bound the request named first, `session.delegated_result_bytes`, is
`smith_domain::feedback_bound`, at least four times the larger of `run_bytes` and
`outcome_bytes` (509,920 bytes or more on the reference), and cut nothing. The child is
told the 1,024 in its `## Your share` and writes to it, which is why `cut` was only 23:
it answered in about the room it was given, far short of the 1,500 words its lead asked.

The design didn't size it: `smith/docs/design/domain/run.md`, 5.3 said only "up to the
answer cap it was told", and `smith/docs/design/protocol/limits.md` named no answer cap.
This cut writes it: a child's answer is its run's result to its asker, so it has the
outcome's room, the larger of 4,096 and `tool_payload` (limits.md, 3.10): 63,740 bytes on
the reference, 10,662 on the stock. `smith check` prints it as `run.answer_bytes`, and the
startup relationship `render-in-result` names it (limits.md, 4). It holds by
construction, since the child's feedback stays below the outcome's four-fold term, so T
and every pinned figure stay. No new startup relationship: one would compare the
derivation with itself.

## Parts and tasks

| Part | Task | Needs | Crates | Design |
|---|---|---|---|---|
| `answer-room` | `answer-room` | none | `smith-domain-run`; tests in `smith-agent-service`, `smith-agent-process-world` | limits.md 3.10, 6; run.md 5.3 |
| `cut-words` | `cut-words` | none | `smith-domain`; tests in `smith-agent-world` | run.md 5.3; limits.md 6 |

- **`answer-room`** (request items 1 and 2): the failing story first, in the agent's
  process world, whose limits are derived from its profile: a real child's 3,000-byte
  answer reaches main whole, where today it is cut at 1,024. A row of the profile table
  holds `run.answer_bytes` equal to `run.outcome_bytes` for the stock's and the
  reference's pairs; a row of the child-answer test cuts at a character's boundary.
- **`cut-words`** (item 3): the marker gains its words, `The answer above is cut: its
  last N bytes were left out. The sub-agent has closed; to read the rest, ask a new one
  for it with a narrower brief.`, and `feedback_bound` measures the marker at its maxima
  in place of a fixed 128 bytes. A continuation is left as an open question (run.md, 15):
  the child closes when it answers.
- **The design** (item 4) is this cut's own, on the trunk with the specs: smith's
  limits.md 3.10, 4 and 6, run.md 5.3 and 15, and temper's
  `temper/docs/design/protocol/configuration.md`, 13.8 (the payload's list), with
  **Not yet** entries in `limits.status.md`, `run.status.md` and smith's `STATUS.md`
  that each task deletes for its half.

Nothing was dropped: four items, two tasks.

## Order and critical path

Both tasks start at once; neither needs the other. The contracts note
(`notes/contracts.md`) fixes the cap, the cut's place, the words and the bound, so
`cut-words` builds without the cap's branch. Critical path: `answer-room`, about 100
implementer-minutes, most of it in the readers' focused tests (the derived cap changes
meaning for every world that derives smith's limits).

## Overlaps

- The two tasks own different crates. Their status lines sit in the same three files,
  placed apart (each task's lines separated by another entry or paragraph), so the
  deletions merge cleanly.
- Drafts beside this one: no open task of `tui-recorded` or `ui-ask-m1` touches these
  files. `ui-ask-m1/mechanics` changed `smith-domain-run/src/prompt.rs` and its tests,
  merged; this plan leaves `prompt.rs` alone.
- temper's `configuration.md`, 13.8 gains one bullet in this cut; no task edits it.
