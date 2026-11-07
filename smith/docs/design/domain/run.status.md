# run: status

2026-11-07. Where smith's code lags or departs from [run.md](run.md).

- **Not yet:** 6, 9.2: messages mid-work and threshold notes. The run
  offers queued messages only at main's opening and when main yields
  (`smith-domain-run/src/run.rs`, `continue_messages`), and no note
  marks the budget's thresholds.
- **Departs:** 3.3: the workspace's section renders as `## Checkout`, and
  the system text says "the checkout" (`smith-domain-run/src/prompt.rs`,
  `render_checkout`). The user's request of 2026-11-04 asks that the word
  stay only where git's checkout is meant, and that behaviour not change;
  this text is what the LLM reads, so it stays (15).
