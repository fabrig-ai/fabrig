# worker: status

2026-10-29. Where temper's code lags or departs from [worker.md](worker.md).

- **Built:** 3.1: admission by watermark: the queues between the worker's components uncapped, no output reservation; the engine's records wait in a capped entrance in order, new work at the watermark (10,198 entries, `temper_worker_domain::WATERMARK`), the engine's answers about admitted turns and calls passing a held assignment; each stage drains what it held when it began; the orders W1 to W5 tested under load. 4.1, 5: a retried producer's fresh hold fetches and freezes its landing tip, and delivers an existing unlanded head only through a verified guarded push. 4.3: configured model bounds classified as Exceeded or Model with their value. Open run behavior and on-demand host reads are integrated. 3.1: the held entrance is the link's capped edge.
- **Not yet:** 4.1, 4.2, 5: expected heads, saved work and checkout that never creates a missing base branch. 9: containment.
