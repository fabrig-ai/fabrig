# run: status

2026-11-08. Where smith's code lags or departs from [run.md](run.md).

Section 5.3 has no known gap in the child's outcome-sized answer cap,
UTF-8 boundary cut or cut guidance. The domain table checks a 14-byte
omission and a cap inside a two-byte character. The agent's simulated
process-world story checks a real child's 3,000-byte answer reaching
main whole, without a cut marker or omitted tool bytes. The canonical
feedback table checks every stop, zero and maximum cut counts, and the
usage suffix. Retaining a cut answer's rest for a continuation remains
section 15's open question; the run keeps no rest beyond the cap.

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

- **Not yet:** 6, 9.2: messages mid-work and threshold notes. The run
  offers queued messages only at main's opening and when main yields
  (`smith-domain-run/src/run.rs`, `continue_messages`), and no note
  marks the budget's thresholds.
- **Departs:** 3.3: the workspace's section renders as `## Checkout`, and
  the system text says "the checkout" (`smith-domain-run/src/prompt.rs`,
  `render_checkout`). The user's request of 2026-11-04 asks that the word
  stay only where git's checkout is meant, and that behaviour not change;
  this text is what the LLM reads, so it stays (15).
