# Request: a test killed at cancellation before its record is unobserved, not a coverage error

From fabrig/dogfood via agents/loop, 2026-11-03 ~00:30Z. The design is `tools/tiers/design.md` 7.3
("The summary verifies exactly one record for every JUnit attempt"; "Unstarted tests are never
claimed as executed"; "Starvation alone returns 0"), and fabrig's
`skein/docs/foundation/testing-strategy.md` 8.1 (starved is green; a test not run because the run
stopped is unobserved, never passed).

## Today

`configuring-p1/integration-1`'s end check (hetzner6, slot 1, tip b631709ea2, ended 00:01Z) went
red with no failing code: the default suite hit nextest's 120 s global timeout under load
(`run.json`: `starved: true`, `wall_s` 119.96, returncode 100), and four
`temper-forge-setup` tests started at 00:00:49.4Z were aborted by it (JUnit: `process aborted with
signal 15 (SIGTERM)`). Their wrappers were killed before writing the pre-exec record, so
`tools/tiers/cpu.py` `summary` raised `record coverage: missing [...4 tests...]; unexpected []`
(exit 2), and fab's precheck read the step red. The tests pass on main. A repair task spent on it
is spent on infrastructure.

## Build (one task)

- `tools/tiers/cpu.py` `summary`: in a starved run (`run.json` `starved: true`), a JUnit attempt
  with no record whose outcome is an abort by SIGTERM or SIGKILL (nextest's "process aborted with
  signal 15"/"9", the cancellation's signals) is **unobserved**: neither a coverage error nor a
  failure nor an executed attempt; the summary counts it with the starved/unobserved outcomes, a
  green warning. Any other missing record, and any missing record in a run that is not starved,
  stays exit 2 as today. Unexpected records keep today's rule.
- `tools/tiers/design.md` 7.3 says so, beside "Unstarted tests are never claimed as executed".
- Tests (`tools/tiers`' own tests, fakes): a starved run with a JUnit SIGTERM abort and no record
  sums green with the test unobserved; the same run not starved stays exit 2; a starved run
  missing the record of a test JUnit shows as passed or failed by assertion stays exit 2.
- Whatever fab reads from the summary (the agents repo's `fab/cpu.py`, `fab/queue_cpu.py`) already
  takes starved and unobserved counts; if the summary's JSON gains a field, name it so fab's
  readers need no change, or say what they need.
