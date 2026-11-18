# macos-tiers: status

2026-11-18. Where the design lags or departs from [macos-tiers.md](macos-tiers.md).

- **Design lags:**
  - 3 to 6: `../testing/README.md`, 2.8 has `macos.yml` run on main four times a day by a schedule, compile every target and run the one story; it says nothing of dispatch, sets, parts, the report, the relay or live on macOS, and no tier of macOS's is in `.forgejo/workflows/`.
  - 4.3: `.config/nextest.toml` has no override for `cfg(target_os = "macos")`.
  - 7: smith and jig have no list of their Linux-only tests.
- **Design departs:** none.
- **Open:** 10's three questions.
