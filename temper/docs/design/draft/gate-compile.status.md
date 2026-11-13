# gate-compile: status

2026-11-13. Where the design lags or departs from [gate-compile.md](gate-compile.md).

Section 4.2's rule for product families is designed: skein's `data/codec/README.md`, section 5, and `temper/docs/design/protocol/store.md`, section 4, keep a family's current version's generated code alone. The code departs: `temper-protocol-store-codec`, `jig-protocol-store-people` and `smith-transcript` hold older versions' code, and so do two wires (`smith-channel`'s `v7`, `jig-protocol-worker-wire`'s `v12`, found by C1's generated-tests check, 2026-11-13); milestone C2 removes them all. Section 3 is designed in skein's `data/codec/README.md`, sections 4 and 6, and `data/codec/command.md`, section 1, with the exception in `skein/docs/foundation/testing-strategy.md`, 2.1; section 7's generated-tests check in `tools/crates/design.md`, 3.4 (milestone C1 of [the execution file](gate-compile.execution.md)). Nothing else of this draft is designed or built yet.

- **Design lags:**
  - 4.2: `data/codec/command.md`, section 1, lets `schema/` hold several texts of a family, its greatest the current.
  - 5: `tools/crates/design.md`, 3.1, names the five crates' splits as not yet planned.
  - 7: `tests/boundary/` has no one-version check.
- **Design departs:**
  - 3.2: `skein-codec-tests`, skein's codec fixtures, keeps its library tests beside those section 3.2 names: its patch module's golden tests are in its library, as `temper-protocol-records`' are (`tools/crates/design.md`, 3.4). C1's designer's reading of the code, 2026-11-13.
  - 3.2, 4.1: the older versions' generated record files C2 removes keep their test modules until it does, and their crates' libraries build no test target, so those tests don't run meanwhile; the patches' goldens still check their layouts. The generated-tests check excepts each such file until C2 (`tools/crates/design.md`, 3.4). C1's designer's choice, 2026-11-13: C2 deletes the files, so moving their tests would only conflict with it. For the same reason, skein-codegen's `Output` keeps `rust`, the records with their tests inside, because only those files' drift tests read it. C2 removes `rust` with the files (`skein/docs/design/data/codec/README.status.md`).
- **Open:** none.
