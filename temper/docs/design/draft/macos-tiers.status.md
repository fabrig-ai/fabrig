# macos-tiers: status

2026-11-19. Where the design lags or departs from [macos-tiers.md](macos-tiers.md).

- **Design lags:**
  - 7: smith and jig have no list of their Linux-only tests.
- **Design departs:**
  - 4.3: every profile of `.config/nextest.toml` gains the overrides that double its slow timeout on macOS, not only those the parts run (`../testing/macos.md`, section 7): nextest applies `default`'s overrides to every profile after its own, so a profile with none of its own would take `default`'s doubled timeouts on macOS in place of its own. The goal's designer chose it, 2026-11-19; the draft's intent, a slow timeout doubled on macOS and no profile of macOS's own, holds.
- **Open:** 10's three questions.
