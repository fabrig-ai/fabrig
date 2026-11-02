# programs: status

2026-11-02. Where smith's code lags or departs from [programs.md](programs.md).

- **Not yet:** 5.1, 5.5: what the program offers is not read from
  `system/init`'s `capabilities`: `checked_init`
  (`smith-protocol-program/src/backend/stream.rs`) checks its tools and
  MCP servers only.
