# run: status

2026-11-11. Where smith's code lags or departs from [run.md](run.md).

- **Not yet:** 8.1, *The checks' bound* and *Out of time, once more*: every check is given `check_timeout` (`smith-domain-run/src/land.rs`, `next`), the charter names no bound, and a check stopped at its deadline refuses its delivery at once, with no retry [in flight: change 73].

Section 3.3 has no known gap in omitting the workspace tool instructions
and Checkout section when no workspace is given. The remaining Checkout
wording departure below concerns runs that have a workspace.

Section 5.3 has no known gap in the child's outcome-sized answer cap,
UTF-8 boundary cut or cut guidance. The domain table checks a 14-byte
omission and a cap inside a two-byte character. The agent's simulated
process-world story checks a real child's 3,000-byte answer reaching
main whole, without a cut marker or omitted tool bytes. The canonical
feedback table checks every stop, zero and maximum cut counts, and the
usage suffix. Retaining a cut answer's rest for a continuation remains
section 13's open question; the run keeps no rest beyond the cap.

Section 5.3's room has ten scripted program-world rows: two to six
calls at a room of five, with program and credential
children. They send all MCP requests before reading replies, mix an
immediate child with children that read then answer, and put credential
children on another provider. Each checks the admitted children's
answers and closure, each excess call's `refused reason=too-many`, main's
accepted answer and removal of private directories. The reported
deployment abort on such a burst was no bound's but a file answer's
routing (protocol/programs.status.md, 5.1); no production bound changed.

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
a silent check still has no output to report (section 13).

## Gaps and departures

- **Not yet:** `ui-ask-m1`'s `proposal-read` part was never designed,
  according to the retirement brief. Its landed run proposal read needs
  design coverage reconciliation against `jig/docs/design/domain/engine/executors.md`,
  3; this is a design gap, not an absent implementation.

- **Departs:** 3.3: the workspace's section renders as `## Checkout`, and
  the system text says "the checkout" (`smith-domain-run/src/prompt.rs`,
  `render_checkout`). The user's request of 2026-11-04 asks that the word
  stay only where git's checkout is meant, and that behaviour not change;
  this text is what the LLM reads, so it stays (15).
