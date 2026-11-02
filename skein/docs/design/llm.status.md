# llm: status

2026-11-02. Where skein's code lags or departs from [llm.md](llm.md).

- **Not yet:** 4.6, 4.7: removal of the withdrawn routes, the Codex route and Anthropic's bearer route with its identity profile. smith still uses them (`smith-protocol-llm/src/endpoints.rs`, `smith-agent-shell/src/config/endpoints.rs`), and jig does not refuse the withdrawn dialect: `jig-protocol-config/src/llm.rs` reads an endpoint's `dialect` as any count, and jig's fixtures write 1.
