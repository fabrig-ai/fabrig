# programs: status

2026-11-02. Where smith's code lags or departs from [programs.md](programs.md).

- **Not yet:** 5.1, 5.5: what the program offers is not read from
  `system/init`'s `capabilities`: `checked_init`
  (`smith-protocol-program/src/backend/stream.rs`) checks its tools and
  MCP servers only. 8: whether the live suite passes on Claude Code
  2.1.312, its two agent-process probes, the probe of two programs
  answering at once on one configuration directory (5.10) and the
  summary session's probe among them, is unknown (testing.status.md,
  2.3).
