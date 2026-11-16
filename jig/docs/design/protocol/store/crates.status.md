# crates: status

2026-11-16. Where jig's code lags or departs from [crates.md](crates.md).

- **Not yet:** 3: the translation's `catalogue`, `component::wires`, `component::floor`, `component::kept_wires` and `openings` name the component's `Store`, `Wire`, `FLOOR`, `KeptWire` and openings' types, and the component extends `component::Levels` with an inherent `impl`.
- **Not yet:** 4.3: the translation's tests, the component's and the domain fixtures they share are one tree, `src/tests/`, unstaged.
- **Not yet:** 2, 4, 5: `jig-protocol-store` is one crate, its records' translation its `codec` module and that module's tests among its tests; no `jig-protocol-store-kit`; the size check excepts it.
