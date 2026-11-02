# programs: status

2026-11-02. Where smith's code lags or departs from [programs.md](programs.md).

- **Not yet:** 5.1, 5.5: what the program offers is not read from
  `system/init`'s `capabilities`: `checked_init`
  (`smith-protocol-program/src/backend/stream.rs`) checks its tools and
  MCP servers only. 8: no live run on Claude Code 2.1.312 has passed: the
  two agent-process probes were red in the after-landing run at main
  `da9027dbf5ae`, the fake and the decoder now take 2.1.312's init and
  thinking shapes, and the summary session's probe is new; whether they
  pass live is unknown (testing.status.md, 2.3).
