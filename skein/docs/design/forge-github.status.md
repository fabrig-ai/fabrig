# forge-github: status

2026-10-29. Where skein's code lags or departs from [forge-github.md](forge-github.md).

- **Built:** 2 to 12: dialect foundation, paged repository/issue/pull/status reads, protection joins, JWT and installation startup, signed deliveries and log, job-log redirect, manifest and App conversion documents.
- **Not yet:** Other read plans and write waits; remaining 4 to 14 behavior. 4.3, 15: batched GraphQL reads and conditional reads beyond the delivery log.

- **Built:** 4.1, 4.4: failed Actions checks resolve the linked job id
  independently of the check-run id, validating repository, head, run
  and positive attempt before pinning. Missing App metadata can resolve
  through the same validated Actions link; an explicit other App stays
  unpinned. Step and fake-GitHub machine regressions compose Checks and
  Job for two failed heads and retain the gate's timestamp in the log tail.

- **Built:** 5.2, 7: ambiguous update stale refusals get one bounded pull
  verification. A moved head stays `Stale`; an unchanged open head yields
  `Conflict` even with an optimistic mergeability hint. Step regressions
  cover verification failures; the fake GitHub protocol world varies the
  refusal over the wire and repeats the attempt. The finding's raw
  exchange is unavailable, so the scenario is not a conformance claim
  about GitHub's exact conflict message.
