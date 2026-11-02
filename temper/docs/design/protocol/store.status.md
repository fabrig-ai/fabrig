# store: status

2026-11-02. Where temper's code lags or departs from [store.md](store.md).

- **Not yet:** 2: family 20 `Setting` (the schema reserves its byte); family 22 `Delivery` (no `Key` or `Stored` variant); and the families of after landing and findings, `Tier`, `Verification`, `LandingLog`, `Finding`, `Mark` and `QaPass`; a tier receipt's failing tests; the `Change` family's `Red`, `Cleared` and `Uncleared`.
- **Departs:** 2, 4: no `Demo` (17) or `DemoAct` (19) family, their bytes reserved and not reused; bytes 25 to 27 hold `Forge`, `ForgeNumber` and `Playground` (`temper-protocol-store/src/keys.rs`), because forges are store records and the playground is reset (`../draft/playground-reset.md`, 3, 4, 6).
