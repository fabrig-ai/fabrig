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

## Batch 2: what the surveys held, and bugs 1 and 16

| Bug | Task | Part | Needs |
|---|---|---|---|
| 1 | `file-op-lane` | `io` | `window-gone` |
| 4 | `issuer-endpoint` | `oauth-endpoints` | `oauth-start-refused` |
| 6 | `llm-valid-check` | `llm-connection` | |
| 7 | `connection-entrances` | `forge-connection` | |
| 8, third half; 15's forge-setup line | `setup-unusable` | `forge-setup` | |
| 8, second half | `setup-remove` | `forge-setup` | `setup-unusable` |
| 11 | `forgejo-path-segments` | `forgejo-paths` | `walker-depth` |
| 12 | `collector-demand` | `json` | |
| 14, other half | `forgejo-timeouts` | `fake-forgejo` | |
| 15, second half | `io-new-valid` | `io` | |
| 15, first half | `window-gone` | `io` | `io-new-valid` |
| 16, seen half | `pidfd-unsupported` | `shell` | |
| 18, patch_check | `patch-check-scratch` | `codegen` | |
| 19, fake-peers | `peers-unreachable` | `fake-peers` | |
| 19, ui-web | `pixels-total` | `ui-web` | |

`needs` here only keeps one writer to a crate, but for `file-op-lane`, whose ceiling needs `Io::new`
asking `valid`. The critical path is skein-io's: `io-new-valid`, `window-gone`, `file-op-lane`, about
145 implementer-minutes.

**Bug 1's rule.** `FileIo` mints with bit 59 and `valid` keeps `Io`'s slots below it, because the
top four bits are taken: `Token::tagged` (jig's harness) refuses a value past 2^60, and smith tags
bits 61 to 63. `skein-ui-text`'s `TAG` tells two `Io`s apart, so it stays, and D044 stays open for it.

**Stand-ins for a test that fails on the old code,** each stated in its spec: `writer-width` (the
old figure equals the new on every target built here, so it fails by not compiling),
`pidfd-unsupported` (the gate compiles no off-Linux arm; the test runs on the arm's function),
`peers-unreachable` (no entrance reaches the cells; clippy's `panic` lint fails on the old code).

## Dropped

- **Bug 8, first half (an invalid `Add` is dropped):** the behaviour stands, but it is the contract
  now. `skein/docs/design/forge-setup.md`, 6.3 (`373fca6fa8`, after the review) says it is ignored, as
  `skein-forge-setup/src/host/boundary.rs` does, and `an_added_forge_with_invalid_settings_is_ignored` pins it.
  A gap stays for a designer: 6.1 says the crate gives the checks, but `limits::check_forge` is
  private, and temper's `setup/runtime.rs` records an `Add` served either way.
- **Bug 16, other half (the readiness table's wrapping generation):** no bug. Looked at:
  `skein-shell/src/readiness.rs`, `finish`, `reap`, `start`, `cancel`, `resume`, `attempt_with`, the
  waiters, `readiness/interrupted.rs`. The generation is in no token, and its only reader is an
  `assert_eq!`. A slot frees in `reap`, after its one `ready` entry, so no stale entry can exist.
- **Bug 18, the `Stack` price in `run/heap.rs`:** no effect. It prices `Stack` where `Build` holds
  `List`, but `Stack::price` and `List::price` are one formula, and
  `patcher_stays_within_its_heap_bound_after_every_call` meters the heap. A rename could carry no
  failing test.
- **Bug 20 (skein-world off unix):** true, but the design rules that target out.
  `skein/docs/design/testing.md`, 5 and 10 have the crate native, on Linux and macOS. Making it build
  would go against them; deleting the six misleading `cfg(unix)` fences in its `lib.rs` could carry no
  failing test.
- **Bug 3:** already fixed. `skein-io`'s `file::Request` and `file::Event` print `len`, never the
  bytes, by hand-written `Debug` (`702facd67f`, with tests in `src/tests/file.rs`, `src/tests/store.rs`).
- **Bug 17:** already fixed. `CryptoRng` derives nothing (`078b2ddbb7`, with a test in
  `skein-lib/src/tests/crypto_rng.rs`).
- **Bug 13:** out, by the draft: debt-4 takes it.

## Overlaps with the drafts beside this one

None that could not be avoided. debt-4 and fabrig-scale change nothing under `skein/`; playground-browser
changes `skein-tls/Cargo.toml`, `skein-ui-web/src/platform/worker*` and `ui-web.md`. `pixels-total` stays in
`src/page/`, and `ui-web.md` is only read. `setup-unusable` changes two temper-service files fabrig-scale
does not list, for the variants it adds.

## Found beside the bugs, left

`jig/scripts/browser.sh` names the same missing profile (bug 5). Two more for a later plan:
`skein-forge-connection/src/limits.rs` has `setup-unusable`'s shape, and `skein-oauth/src/redirect.rs`,
near 64, has `issuer-endpoint`'s `starts_with` on a confidential redirect.
