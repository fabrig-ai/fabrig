# browser: status

2026-11-07. Where jig's code lags or departs from [browser.md](browser.md).

Section 11 has no known gap in taking a malformed or wrong-length
`secret` as an empty secret, removing it from the address and sending the
link exchange. Without `secret`, or at a different route, no link is taken.
The shell's table covers these cases; cookie fallback remains deferred in
section 17.

- **Departs:** 15: the browser tier runs through `tools/tiers/run`, but
  `jig/scripts/browser.sh` stays, and ends in `--profile browser`, which
  `.config/nextest.toml` does not define, so it fails at its last line;
  it retires in jig's own branch (`tools/tiers/design.md`, 8).

Sections 15, 18 and 19 have the HTTP engine end's cookie-only browser
mark, refusal of a bearer even beside the browser header, and an
approval's take answered with a cookie and no credential in its body.
The mapping's tables cover the routes, guards and credential carriers.
