# bugs-skein: the plan

The skein bugs the debt backfill routed (`inputs/bugs-skein.md`): small independent fixes, each with
a test that fails on the old code. Released in batches.

## Batch 1: the bugs seen on the tree

| Bug | Task | Part |
|---|---|---|
| 2 | `oauth-start-refused` | `oauth-client` |
| 5 | `browser-profile` | `scripts` |
| 9 | `writer-width` | `forge-client` |
| 10 | `walker-depth` | `forge-dialects` |
| 14, seen half | `github-streams` | `fake-forges` |
| 18, seen half | `golden-directories` | `codec` |
| 19, seen half | `browser-contains` | `browser-kit` |

No task needs another: the critical path is any one task. Each owns its crate.

## Dropped

- **Bug 3:** already fixed. `skein-io`'s `file::Request` and `file::Event` print `len`, never the
  bytes, by hand-written `Debug` (`702facd67f`, with tests in `src/tests/file.rs`, `src/tests/store.rs`).
- **Bug 17:** already fixed. `CryptoRng` derives nothing (`078b2ddbb7`, with a test in
  `skein-lib/src/tests/crypto_rng.rs`).
- **Bug 13:** out, by the draft: debt-4 takes it.

## Outstanding

Bugs 1, 16 and the surveyed "not re-checked" bugs follow in later batches.

## Overlaps with the drafts beside this one

None in this batch. debt-4 and fabrig-scale change nothing under `skein/`; playground-browser
changes `skein-tls/Cargo.toml`, `skein-ui-web/src/platform/worker*` and `ui-web.md`, none of which
this batch touches (`ui-web.md`, 14.1 is only read).
