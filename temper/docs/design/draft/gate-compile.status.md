# gate-compile: status

2026-11-13. Where the design lags or departs from [gate-compile.md](gate-compile.md).

Section 4 is designed in skein's `data/codec/README.md`, section 5, and `data/codec/command.md`, section 1, and in `temper/docs/design/protocol/store.md`, section 4: a family a build stores, sends or embeds keeps its current version's text and generated code alone, an older record being read as a generic value of its kept text. Section 7's one-version check is designed in `tools/crates/design.md`, 3.2 (milestone C2 of [the execution file](gate-compile.execution.md)). Section 3 is designed in skein's `data/codec/README.md`, sections 4 and 6, and `data/codec/command.md`, section 1, with the exception in `skein/docs/foundation/testing-strategy.md`, 2.1; section 7's generated-tests check in `tools/crates/design.md`, 3.4 (milestone C1 of [the execution file](gate-compile.execution.md)). Nothing else of this draft is designed or built yet.

- **Design lags:**
  - 5: `tools/crates/design.md`, 3.1, names the five crates' splits as not yet planned.
- **Design departs:**
  - 3.2: `skein-codec-tests`, skein's codec fixtures, keeps its library tests beside those section 3.2 names: its patch module's golden tests are in its library, as `temper-protocol-records`' are (`tools/crates/design.md`, 3.4). C1's designer's reading of the code, 2026-11-13.
  - 4.1, 4.2, 7: `smith-channel`'s `schema/` held version 6's text beside version 7's, which section 4.1 doesn't name. Version 6 is before the channel's baseline, so it has no kept text, and nothing reads the text, so it goes as `transcript-v6.schema` does. The two crates' version-6 golden bytes stay, as bytes: jig's fixtures and tests read the transcript's (smith's `protocol/channel.md`, 2). Since main's channel version 8 (9959c93145), `schema/` also held version 7's text, kept in `schema/kept/`, and `src/generated/v7.rs`, which nothing compiles: both go too, and version 7's goldens stay. C2's designer's reading, 2026-11-13.
  - 7: the one-version check's exceptions are skein's codec fixtures and jig's two patch fixtures, `jig-test-patches` and `jig-test-patches-jig`, which section 7 doesn't name: their families are fixtures no build stores, sends or embeds, which section 4.2's rule doesn't bind, and their `schema/` holds two versions of a family for their stories (skein's `data/codec/command.md`, section 1; `tools/crates/design.md`, 3.2). C2's designer's reading, 2026-11-13.
- **Open:** none.
