# baseline: status

2026-11-11. Where temper's code lags or departs from [baseline.md](baseline.md).

- **Not yet:** 3: no owner crate keeps a text: no `schema/kept/`, no generated `patches/mod.rs`; smith-transcript's `src/patches.rs` is hand-written and empty.
- **Not yet:** 4: no real set: `temper-service`'s store hands none, so jig's store holds `PatchSets::NONE`, and `temper-engine-shell`'s `versions` prints every level none.
- **Not yet:** 5: jig's open refuses a store at the baseline as `Unleveled` wherever the binary's level is set, and compares the layouts at equal levels.
- **Not yet:** 6: no golden store, and no test that starts on one.
- **Not yet:** 7: no gate check of patches, and `AGENTS.md` does not name the policy.
- **Not yet:** 8: no fixture patch, and no test seam for the sets in `temper-service`.
