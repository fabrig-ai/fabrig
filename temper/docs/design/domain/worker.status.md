# worker: status

2026-11-03. Where temper's code lags or departs from [worker.md](worker.md).

- **Not yet:** 4.1, 5: a base with heads merged in, and its
  preparation's conflict naming the head: the checkout's starts are a
  base branch, a branch, a commit, saved work and a merge in progress
  (`temper-worker-domain-checkout/src/boundary.rs`, `Start`). 4.4: the prepare and key kinds: no kind's entry is built,
  so the worker's hello declares none (`temper/crates/temper-service/src/worker.rs`
  gives its root no kind), and with prepare the `Rewrite` delivery
  (`../protocol/git.md`, 4.10). 5: a missing base branch fails the
  preparation; the checkout creates it from the default branch for a
  repository that may be written
  (`temper-worker-domain-checkout/src/hold.rs`, `PrepareStep::Create`).
  5, 9: git's invocations in contained process trees;
  `temper-worker-protocol-git` runs them as plain child processes.
