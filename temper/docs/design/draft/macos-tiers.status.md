# macos-tiers: status

2026-11-19. Where the design lags or departs from [macos-tiers.md](macos-tiers.md).

- **Design lags:**
  - 7: smith and jig have no list of their Linux-only tests.
- **Design departs:**
  - 4.3: every profile of `.config/nextest.toml` gains the overrides that double its slow timeout on macOS, not only those the parts run (`../testing/macos.md`, section 7): nextest applies `default`'s overrides to every profile after its own, so a profile with none of its own would take `default`'s doubled timeouts on macOS in place of its own. The goal's designer chose it, 2026-11-19; the draft's intent, a slow timeout doubled on macOS and no profile of macOS's own, holds.
  - 4.3: the browser part's Chrome for Testing is 147.0.7727.117, not the 147.0.7727.101 that Linux's browser tiers run, because Chrome for Testing has no build 147.0.7727.101. The pin is the nearest build on the same branch, a later patch of the same Chromium (`../testing/macos.md`, section 6). The requester chose it, with its digest, 2026-11-19.
  - 4.3: the browser part also takes a C compiler for `wasm32-unknown-unknown`, Homebrew's LLVM, which the draft's list doesn't name (`../testing/macos.md`, section 6): ring's build script compiles C for that target, and the runner's `clang`, Apple's, has none, so the part's build failed before any test ran (GitHub run 44517790115). draft-13 asked for it, 2026-11-19; the draft's intent, the browser part building and running its pairs, holds.
- **Open:** 10's three questions.
