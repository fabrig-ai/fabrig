# survey-answer: the run's binding calls

The design is `smith/docs/design/protocol/limits.md`, 3.10 and 6, and
`smith/docs/design/domain/run.md`, 5.3. These are the shapes the two tasks build to, so
neither needs the other's branch.

## 1. A sub-agent's answer has the outcome's room (`answer-room`)

- `smith_domain_run::Limits::answer_bytes` stays a `u32` with its name. `derive` sets it
  to the outcome's room, the larger of 4,096 and `tool_payload`, the same value it gives
  `outcome_bytes` (which stays `u64`). No new `Derivation` field: `tool_payload` is
  already there.
- Nothing else of the derivation moves. With the product's values the child's feedback
  stays below the outcome's four-fold term in `smith_domain::feedback_bound`, so
  `session.delegated_result_bytes` and T don't move.
- The child is told the new figure by the share text already
  (`smith-domain-run/src/prompt.rs`, `render_share`); its words stay.

## 2. The cut falls at a character's boundary (`answer-room`)

- `smith-domain-run/src/agent.rs`, `yielded`: an answer longer than `answer_bytes`
  keeps its head up to the cap, ending at the last UTF-8 character boundary at or below
  it, so the kept bytes never end inside a character. `cut` counts every byte left out:
  the answer's length less the kept length.
- An answer whose bytes before the cap are not UTF-8 is kept at the cap, as today: the
  boundary rule moves the cut back over at most the three continuation bytes of one
  character that straddles the cap, never further.
- `Returned::Answered { text, cut, stop, used, left }` keeps its fields.

## 3. A cut answer's words (`cut-words`)

`smith-domain/src/feedback.rs`, `render_child`, writes, for an answer `A`, its cut `N`
and its stop's name `S` (`end-turn`, `budget`, `max-tokens`, `refusal`, `no-calls`, as
today):

- `N == 0` and `S` is `end-turn`: `A`, as today.
- `N == 0` and `S` is any other: `A` + `\n[child-result cut=0 stop=S]`, as today.
- `N > 0`, any stop:

  ```text
  A
  [child-result cut=N stop=S]
  The answer above is cut: its last N bytes were left out. The sub-agent has closed; to read the rest, ask a new one for it with a narrower brief.
  ```

  that is, `A` + `\n[child-result cut=N stop=S]\nThe answer above is cut: its last N
  bytes were left out. The sub-agent has closed; to read the rest, ask a new one for it
  with a narrower brief.`, `N` in decimal both times, ASCII only, no trailing newline.

The usage line follows as today (`\n\nChild usage: ...`).

## 4. The feedback bound holds the words (`cut-words`)

`smith_domain::feedback_bound`'s child term is the answer cap, plus the marker measured
at its maxima (`cut` at `u64::MAX`, the longest stop name, the words of section 3), plus
the usage suffix measured as today. The measure replaces the fixed 128 bytes, through the
same measuring `Text` the usage suffix uses, so the bound follows the words if they
change. It stays `None` on overflow.
