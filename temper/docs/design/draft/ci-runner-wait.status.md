# ci-runner-wait: status

2026-11-16. Where the design lags or departs from [ci-runner-wait.md](ci-runner-wait.md).

The design carries section 1 in `../domain/forge/change/stalls.md`, 2, with the turn a waiting change keeps in `../domain/forge/queues.md`, and section 2 in `../domain/landing.md`, 4.2, and `../domain/landing-proof.md`, 2.1. The stall's text, `../domain/forge/change/repairs.md`'s *Stalls* and the bullets after it, moved to `stalls.md` whole, since `repairs.md` had no room under the size limit; `ci-runner-wait.md`, section 4 there is the line that kept a job's state diagnostic only.

- **Design departs:**
  - 1: a wait for a runner has a bound of its own, `forge.change_policy.runner_wait_ms` (4 hours on the stock), where the draft counts it nothing and bounds it not: a job whose labels no runner has would otherwise wait unheld for ever, keeping its change's landing turn. A landing's gate run takes the same bound. Decided by the user, 2026-11-16, with the unknown-state grace's default, `forge.change_policy.unknown_grace_ms`, 30 minutes on the stock.
  - 1: the execution is read by run (`Read::Run`, one read a linked run, at most 8 a step), not by job: a run's read says waiting and blocked alike, which count the same, and a job's state is read only to explain a hold, as before.
