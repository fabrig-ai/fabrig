# run-start: status

2026-11-27. Where the tree's code lags or departs from [run-start.md](run-start.md).

- **Not yet:** 1.1: the conventions hold a guide and one checks path alone, with no hooks directory. [in flight: change 20339]
- **Departs:** 1.3: the workspace's section renders as `## Checkout`, and
  the system text says "the checkout" (`smith-domain-run/src/prompt.rs`,
  `render_checkout`). The user's request of 2026-11-04 asks that the word
  stay only where git's checkout is meant, and that behaviour not change;
  this text is what the LLM reads, so it stays (15).
