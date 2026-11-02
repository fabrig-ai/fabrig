# programs: status

2026-11-02. Where smith's code lags or departs from [programs.md](programs.md).

- **Built:** 1 to 9: Claude Code backend, its fake, sign-in, tools, stream, notices, budgets and sessions; live checks exist. 5.9: Open runs on a program backend (replies, listening, parking, wind-down; the domain's Yield request; the stream's JSON token bound from its line ceiling); the fake scripted per conversation by cue, reading by line, and its built-ins acting on the workspace. The real program's stream (system/init without the prompt tool, system notices passed over, a later init checked as the first, a thinking turn's records under one message id as one completion, its final usage), a result contract's StructuredOutput, the program's own call; and a turn interrupted by the control request, SIGINT a cancel's alone; an interrupt of the agent's says so in the next message; the live suite last passed against Claude Code 2.1.311. Every MCP tool schema an object with no top-level combinator, alternatives under `outcome`; one outcome path per run (StructuredOutput under a contract, finish without).
- **Built:** 5.3 and 5.4: modify's `remove` in the agent's server; native file snapshots supply its read-before-remove knowledge.
- **Built:** 8: the fake follows Claude Code 2.1.312: init's server entry carries `"source":"dynamic"` and `claude_code_version` is `2.1.312`; a thinking turn sends a coalesced `thinking_delta` with `estimated_tokens: null` beside the number-bearing one. The two agent-process probes expect both, and the decoder's unit tests take both (`stream_tests.rs`, `turns_tests.rs`), no production change needed.
- **Built:** 5.8, 8: a fresh program run reports `run.started.resumed` false and a carried-over one true (the agent-process world's `program_events.rs`, `program_resume.rs`); `compact::checked_init` is unit-tested (`compact_tests.rs`); `a_live_summary_session_offers_no_tools` probes the summary session's init live.
- **Not yet:** 8: the live suite on 2.1.312. Its two probes were red at main da9027dbf5ae, and the summary probe is new; whether they pass in the after-landing pair on a main holding the fixes is unknown (testing.status.md).

- **Built:** 5.7: reserve host-tool grants use the same finishing choice as direct completions.

- **Built:** 5.7: Finite wind-down admits the granted `wait` to park, using the same run policy as direct completions.
