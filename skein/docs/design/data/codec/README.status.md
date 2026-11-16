# codec: status

2026-11-16. Where skein's code lags or departs from [README.md](README.md).

- **Departs:** 4: `Output` also exposes `rust`, the records' module with its tests inside. The generator's own tests in `skein/testing/skein-codegen/src/tests.rs` read it to check the emitted code. Consumer drift tests use `records` and `tests` apart.

