# gate-compile: status

2026-11-13. Where the design lags or departs from [gate-compile.md](gate-compile.md).

Section 4.2's rule for product families is designed: skein's `data/codec/README.md`, section 5, and `temper/docs/design/protocol/store.md`, section 4, keep a family's current version's generated code alone. The code departs: `temper-protocol-store-codec`, `jig-protocol-store-people` and `smith-transcript` hold older versions' code, which milestone C2 removes. Nothing else of this draft is designed or built yet.

- **Design lags:**
  - 3.2: skein's `data/codec/README.md`, section 6, has each generated file carry its own golden and bound tests, and its `data/codec/command.md`, section 1, has the owner's drift test write the generated code alone. `skein/docs/foundation/testing-strategy.md`, 2.1, keeps step tests in the crate they test, and names no exception for a codec crate's generated tests.
  - 4.2: `data/codec/command.md`, section 1, lets `schema/` hold several texts of a family, its greatest the current.
  - 5: `tools/crates/design.md`, 3.1, names the five crates' splits as not yet planned.
  - 7: `tests/boundary/` has neither check.
- **Design departs:** none yet.
- **Open:** none.
