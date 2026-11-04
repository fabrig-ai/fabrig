# programs: status

2026-11-04. Where smith's code lags or departs from [programs.md](programs.md).

- **Not yet:** 5.1, 5.5: what the program offers is not read from
  `system/init`'s `capabilities`: `checked_init`
  (`smith-protocol-program/src/backend/stream.rs`) checks its tools and
  MCP servers only. 8: whether the live suite passes on Claude Code
  2.1.312 is unknown; 5.8's explicit current-activation refresh is
  checked through the fake program, with its live model treatment unknown.
  Its two agent-process probes and the summary
  session's probe passed in the after-landing run at main
  `df41529d3ee1`, whose Claude Code release is unknown. No after-landing
  run read holds the 5.10 probes of two programs answering at once or
  children answering in a later activation on one configuration directory
  (testing.status.md, 2.3).
  8: no live walk establishes that the stream rejection seen in the advisor
  and the earlier read-only/review sightings is resolved. The step regression
  exercises a reachable `StructuredOutput` boundary; the rejected live record
  is unknown.
