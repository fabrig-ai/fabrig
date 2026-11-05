# request-dump: standing rules

Every task of this plan holds to these.

- **What this is:** a temporary diagnostic, off by default, that comes out whole after v0.1.0 (`temper/docs/design/draft/dogfood-hacks.md`, 2.13). Add nothing it doesn't need: no settings, no protocol, store, schema or client change, no `docs/help` page.
- **Off costs one branch.** With the dump not asked for, no body is copied, no queue fills and no file is touched.
- **Never a credential.** Nothing dumped comes from a `Credential`, an `Endpoint`, a header, a `GrantValue`, a program's environment, `mcp.json` or the configuration directory. A line in a log holds a path and an error's words, never a body.
- **Never fails a run.** A failure of the dump is one log line. No `expect` or `unwrap` on what the disk or an environment value does.
- **No boundary names:** skein and smith code, comments and tests name neither temper nor `TEMPER_DUMP_LLM`. Their comments cite their own status file: `skein/docs/design/llm.status.md`, `skein/docs/design/llm-connection.status.md`, `smith/docs/design/protocol/agent.status.md` or `smith/docs/design/protocol/programs.status.md`. temper's cite `temper/docs/design/draft/dogfood-hacks.md`, 2.13.
- **Leave alone:** every `temper-client-*` crate and every UI file; the design docs and drafts, which this plan's cut has already written; `smith/crates/smith-agent-shell/README.md` and every other Markdown file, some of which tests read (`.config/gate/read-by-tests.txt`).
- **Fakes:** skein's fake LLM peer and the fake Claude program, in the agent-process worlds. No real host, no real provider, no live probe.
- **Budgets and timeouts are the user's.** A new test here takes milliseconds of simulated work and a few small files in a scratch directory.
