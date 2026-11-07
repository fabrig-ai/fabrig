# browser: status

2026-11-07. Where jig's code lags or departs from [browser.md](browser.md).

Section 11 has no known gap in taking a malformed or wrong-length
`secret` as an empty secret, removing it from the address and sending the
link exchange. Without `secret`, or at a different route, no link is taken.
The shell's table covers these cases; cookie fallback remains deferred in
section 17.

- **Not yet:** 15, 18, 19: the engine end telling the component a browser's exchange by its cookie alone, and its step tests; the approval's take answered to a browser with the cookie.
- **Departs:** 15: the browser tier runs through `tools/tiers/run`, but
  `jig/scripts/browser.sh` stays, and ends in `--profile browser`, which
  `.config/nextest.toml` does not define, so it fails at its last line;
  it retires in jig's own branch (`tools/tiers/design.md`, 8).
