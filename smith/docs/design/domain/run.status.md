# run: status

2026-11-07. Where smith's code lags or departs from [run.md](run.md).

Section 5.2 has no known gap in a busy or unanswered host call's retry
backoff: it doubles from `host_backoff` to 64 times that first wait,
preserving the call's name. The eight-busy-answer table checks waits
of 1, 2, 4, 8, 16, 32, 64 and 64 units. A next ask at or past the
caller's deadline is suppressed; an unanswered ask's uncertainty remains
until the terminal answer or caller expiry.

Section 8.1 has no known gap in failed-check feedback: it carries the
repository, convention path, exit, given milliseconds, cut count and
output tail. The given span is retained when the check starts, bounded
by both `check_timeout` and the call's remaining time, and is printed
for every exit. The machine's deadline table keeps output read before
and during stopping, through pipe closure. No timeout figure changes;
a silent check still has no output to report (section 15).

- **Not yet:** 5.3: a sub-agent's answer cap is 1,024 bytes rather than
  the outcome's room, and a cut falls at a byte rather than a character's
  boundary (`smith-domain-run/src/limits.rs`, `derive`; `src/agent.rs`,
  `yielded`).
- **Not yet:** 6, 9.2: messages mid-work and threshold notes. The run
  offers queued messages only at main's opening and when main yields
  (`smith-domain-run/src/run.rs`, `continue_messages`), and no note
  marks the budget's thresholds.
- **Not yet:** 5.3: a cut answer's marker is the bracket alone, with no
  words on how much was left out or how to read it
  (`smith-domain/src/feedback.rs`, `render_child`).
- **Departs:** 3.3: the workspace's section renders as `## Checkout`, and
  the system text says "the checkout" (`smith-domain-run/src/prompt.rs`,
  `render_checkout`). The user's request of 2026-11-04 asks that the word
  stay only where git's checkout is meant, and that behaviour not change;
  this text is what the LLM reads, so it stays (15).
