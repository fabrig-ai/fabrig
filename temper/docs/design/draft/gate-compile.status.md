# gate-compile: status

2026-11-13. Where the design lags or departs from [gate-compile.md](gate-compile.md).

Nothing of this draft is designed or built yet.

- **Design lags:**
  - 3.2: skein's `data/codec/README.md`, section 6, has each generated file carry its own golden and bound tests, and its `data/codec/command.md`, section 1, has the owner's drift test write the generated code alone.
  - 4.2: `data/codec/command.md`, section 1, lets `schema/` hold several versions of a family, its greatest the current. The code departs from `data/codec/README.md`, section 5: `temper-protocol-store-codec` and `jig-protocol-store-people` hold older versions' generated code.
  - 5: `tools/crates/design.md`, 3.1, names the five crates' splits as not yet planned.
  - 7: `tests/boundary/` has neither check.
- **Design departs:** none yet.
- **Open:** none.
