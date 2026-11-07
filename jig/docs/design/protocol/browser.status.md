# browser: status

2026-11-07. Where jig's code lags or departs from [browser.md](browser.md).

- **Departs:** 15: the browser tier runs through `tools/tiers/run`, but
  `jig/scripts/browser.sh` stays, and ends in `--profile browser`, which
  `.config/nextest.toml` does not define, so it fails at its last line;
  it retires in jig's own branch (`tools/tiers/design.md`, 8).
