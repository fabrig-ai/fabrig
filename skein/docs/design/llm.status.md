# llm: status

2026-10-29. Where skein's code lags or departs from [llm.md](llm.md).

- **Built:** 4: edge and derived-capacity checks through `client::valid`, edge bounds and scenario ceilings without a client heap sum. 2.2, 4.7, 4.8: requests ask for one call where the caller declares one. 2.5: cut HTTP exchanges report `Unavailable`; 4.8: Codex requests sent gzip.
- **Not yet:** 4.6, 4.7: withdrawn routes still present. 6, 7: Codex routing state, WebSocket and remaining transport/capture questions.
