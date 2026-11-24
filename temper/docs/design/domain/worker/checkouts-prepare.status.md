# checkouts-prepare: status

2026-11-24. Where the tree's code lags or departs from [checkouts-prepare.md](checkouts-prepare.md).

- **Not yet:** 1: a missing base branch fails the
  preparation; the checkout creates it from the default branch for a
  repository that may be written
  (`temper-worker-domain-checkout/src/hold.rs`, `PrepareStep::Create`).
- **Not yet:** the whole document and `README.md`, 8: git's invocations in contained process trees;
  `temper-worker-protocol-git` runs them as plain child processes.
- **Not yet:** 8: a base with heads merged in has no real run on record: `temper-fake-git`
  holds a two-head start's commands and the `merge-in-progress` transcript git's
  refusal of a second merge, and the request `parallel-faults` of 2026-11-06 has
  its proof in a rerun of its replay, which no record shows. It asks whether this
  is the fault of the early start in the request `landing-fixes` of 2026-11-06,
  item 3, an uncommitted merge at `7913a87678`. Both deployments ran before
  `f6b4c408a7` had each merge it commits checked out, so it may be, by reading
  alone: neither was reproduced on a real run, and `landing-fixes` has its proof in
  the next landing plan on a real deployment, which no record shows either.
- **Not yet:** 9: a producer's start that finds its base contained stands
  on `temper-fake-git` alone (`a_resumed_producer_holding_the_tip_pushes_no_merge`,
  `temper/tests/worker/checkout/tests/fakes/merges.rs`), with that same proof.
- **Not yet:** 9: a parked producer's delivered and saved edits survive a moved landing
  tip, repeated questions and a save refused for markers in the composed system
  world's two `parked` stories. Those use fake git and a fake disk, and their
  first committed edit is a delivery, not a run's own `git commit`. The
  `resume-told` request asks for proof in a real checkout; none is on record,
  nor its next deployment replay. The clean-merge preparation and the conflict
  advice, including the help page and stock charters, have scripted coverage
  alone; whether real producers and their answerers now read the tree rightly
  awaits that replay, as the request's `walk = false` leaves it.
