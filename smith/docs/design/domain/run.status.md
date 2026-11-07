# run: status

2026-11-07. Where smith's code lags or departs from [run.md](run.md).

Section 8.1 has no known gap in failed-check feedback: it carries the
repository, convention path, exit, given milliseconds, cut count and
output tail. The given span is retained when the check starts, bounded
by both `check_timeout` and the call's remaining time, and is printed
for every exit. The machine's deadline table keeps output read before
and during stopping, through pipe closure. No timeout figure changes;
a silent check still has no output to report (section 15).

- **Not yet:** 5.2: a host call asked again waits `host_backoff` before
  every ask (`smith-domain-run/src/run/relay.rs`, `host_recover`), so a
  host that stays busy is asked twenty times a second until the caller's
  deadline.
- **Not yet:** 6, 9.2: messages mid-work and threshold notes. The run
  offers queued messages only at main's opening and when main yields
  (`smith-domain-run/src/run.rs`, `continue_messages`), and no note
  marks the budget's thresholds.
- **Departs:** 3.3: the workspace's section renders as `## Checkout`, and
  the system text says "the checkout" (`smith-domain-run/src/prompt.rs`,
  `render_checkout`). The user's request of 2026-11-04 asks that the word
  stay only where git's checkout is meant, and that behaviour not change;
  this text is what the LLM reads, so it stays (15).
