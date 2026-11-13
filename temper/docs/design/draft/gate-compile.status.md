# gate-compile: status

2026-11-13. Where the design lags or departs from [gate-compile.md](gate-compile.md).

Section 4 is designed in skein's `data/codec/README.md`, section 5, and `data/codec/command.md`, section 1, and in `temper/docs/design/protocol/store.md`, section 4: a family a build stores, sends or embeds keeps its current version's text and generated code alone, an older record being read as a generic value of its kept text. Section 7's one-version check is designed in `tools/crates/design.md`, 3.2 (milestone C2 of [the execution file](gate-compile.execution.md)). Nothing else of this draft is designed or built yet.

- **Design lags:**
  - 3.2: skein's `data/codec/README.md`, section 6, has each generated file carry its own golden and bound tests, and its `data/codec/command.md`, section 1, has the owner's drift test write the generated code alone. `skein/docs/foundation/testing-strategy.md`, 2.1, keeps step tests in the crate they test, and names no exception for a codec crate's generated tests.
  - 5: `tools/crates/design.md`, 3.1, names the five crates' splits as not yet planned.
  - 7: `tests/boundary/` has no check of generated tests outside the library.
- **Design departs:**
  - 4.1, 4.2, 7: `smith-channel`'s `schema/` held version 6's text beside version 7's, which section 4.1 doesn't name. Version 6 is before the channel's baseline, so it has no kept text, and nothing reads the text, so it goes as `transcript-v6.schema` does. The two crates' version-6 golden bytes stay, as bytes: jig's fixtures and tests read the transcript's (smith's `protocol/channel.md`, 2). C2's designer's reading, 2026-11-13.
  - 7: the one-version check's exceptions are skein's codec fixtures and jig's two patch fixtures, `jig-test-patches` and `jig-test-patches-jig`, which section 7 doesn't name: their families are fixtures no build stores, sends or embeds, which section 4.2's rule doesn't bind, and their `schema/` holds two versions of a family for their stories (skein's `data/codec/command.md`, section 1; `tools/crates/design.md`, 3.2). C2's designer's reading, 2026-11-13.
- **Open:** none.
