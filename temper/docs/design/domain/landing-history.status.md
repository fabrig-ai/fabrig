# landing-history: status

2026-11-17. Where temper's code lags or departs from [landing-history.md](landing-history.md).

- **Not yet:** 2.2, 5.1: `update` and `updated` have no `forge_base`, and `update`'s base is the tip its step read, not the base it asks for (`temper-engine-domain-forge/src/domain.rs`, `emit_change_effect`) [in flight: change 6069].
