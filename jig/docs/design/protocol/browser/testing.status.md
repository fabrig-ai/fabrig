# testing: status

2026-11-10. Where the tree's code lags or departs from [testing.md](testing.md).

- **Departs:** the whole document: the browser tier runs through `tools/tiers/run`, but
  `jig/scripts/browser.sh` stays, and ends in `--profile browser`, which
  `.config/nextest.toml` does not define, so it fails at its last line;
  it retires in jig's own branch (`tools/tiers/design.md`, 8).

`testing.md` and `README.md`, sections 17 and 18 have the HTTP engine end's cookie-only browser
mark, refusal of a bearer even beside the browser header, and an
approval's take answered with a cookie and no credential in its body.
The mapping's tables cover the routes, guards and credential carriers.
