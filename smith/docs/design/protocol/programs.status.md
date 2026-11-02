# programs: status

2026-11-02. Where smith's code lags or departs from [programs.md](programs.md).

- **Not yet:** 2, 5.10: sub-agents on a program backend.
  `smith-domain/src/translate.rs` withholds the sub-agents family from a
  program's conversation; `smith-protocol-program` lists no `sub_agent`
  and refuses its call (`backend/files.rs`, `backend/conversation.rs`);
  `smith-agent-service/src/service/program.rs` runs main's conversation
  alone on the program, a child's completions going to the direct
  backend, and nothing ends a child's program when its conversation
  does. 5.1, 5.5: what the program offers is not read from
  `system/init`'s `capabilities`: `checked_init`
  (`smith-protocol-program/src/backend/stream.rs`) checks its tools and
  MCP servers only. 8: the fake's stories have no children, and no live
  story runs them; whether the live suite passes on Claude Code
  2.1.312, its two agent-process probes and the summary session's probe
  among them, is unknown (testing.status.md, 2.3).
