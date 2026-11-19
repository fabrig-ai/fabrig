# macos-tiers: status

2026-11-19. Where the design lags or departs from [macos-tiers.md](macos-tiers.md).

- **Design lags:**
  - 3 to 6: `../testing/macos.md`, sections 1 and 2 have `macos.yml` run on main four times a day by a schedule, compile every target and run the one story; it says nothing of dispatch, sets, parts, the report, the relay or live on macOS, and no tier of macOS's is in `.forgejo/workflows/`. Its section 4 has the suites under a `macos` nextest profile and no reader of the results.
  - 4.3: `.config/nextest.toml` has no override for `cfg(target_os = "macos")`.
  - 7: smith and jig have no list of their Linux-only tests.
- **Design departs:** none.
- **Open:** 10's three questions.
