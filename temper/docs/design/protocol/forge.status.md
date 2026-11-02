# forge: status

2026-11-02. Where temper's code lags or departs from [forge.md](forge.md).

- **Not yet:** 2.2, 2.6: a connected Forgejo: `Connect` with its pasted tokens, the `Check` plan and a token's `Replace` (no connect carries tokens, and `temper-service/src/setup/lifecycle.rs` never stages `Check`). 2.4: `Fact::Gap` is dropped, neither logged nor shown (`temper-service/src/engine.rs`). 10: the idle cost check, an idle span's calls held to a bound computed from the keep's `poll`, backoff and `slow`; the worlds compare history sizes alone.
- **Departs:** 2, 2.1, 2.6, 7: a forge's records, its number, mode, place, bot and git program come from its store record made by `SetUp` or `Connect` and numbered at runtime, not from configuration's `forges`, because forges are store records and the playground is reset (`../draft/playground-reset.md`, 3, 4, 6). 2.8, 8.1: the box's link also carries a group's start by kind (`start FORGE owned|fake`) and `remove FORGE`, and the stop of the demo's forge is the playground's reset, for the same reason.
