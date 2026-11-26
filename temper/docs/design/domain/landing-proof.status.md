# landing-proof: status

2026-11-26. Where the tree's code lags or departs from [landing-proof.md](landing-proof.md).

- **Not yet:** 1.2, *Only a refusal is a failed receipt*, 2.3 and 2.4: a report naming no failed test, or one not read, makes a failed receipt and asks a repair, a missing one is the runner's whatever the run showed, and a retained failed receipt naming nothing is reused [in flight: change 18910].
- **Not yet:** 1.2, *A failed receipt*: a project whose checks can't flake still reruns known flakes [in flight: change 18911].

- **Not yet:** 1.1: the project's
  prepare and key commands, as jobs (`worker/jobs.md`). 1.2: the baseline
  at main's tip.

- **Not yet:** 2.1, by plan `forge-polls-quiet`: a landing polls its
  runs in flight every `poll`, not at the client's `poll_at`.
