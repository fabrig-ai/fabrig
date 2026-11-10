# components: status

2026-11-10. Where the tree's code lags or departs from [components.md](components.md).

- **Departs:** the whole document: `temper-client`'s COMMIT is its own build script's
  (`crates/temper-client/build/main.rs`, `TEMPER_CLIENT_COMMIT`), a second copy
  of `temper-version`'s, which it doesn't link: M1 built it while `temper-version`
  was being built beside it, and no plan has switched it yet.
