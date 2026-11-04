# bugs-skein: the skein bugs the debt backfill routed

From `coord/debt/for-director.md` (fabrig/debt, 2026-11-03; the user approved routing these). Paths starting with a crate's name are under `skein/crates/`; line numbers are at `a6a8155d35` or older, so locate each again.

Rules for this plan:

- Keep to the code's bugs below; nothing else from the debt list. Low priority: small tasks, no reshaping.
- Bug 13 (skein-kv deep copies) is out: fabrig/debt takes it into D017 in debt-4, which owns `skein-kv/src/store.rs`.
- Re-check every bug marked "not re-checked" against main first, by survey. One that no longer holds is dropped and named in the report, with what was looked at.
- Every fix carries a test that fails on the old code.
- Keep every part in the plan. Wait for a designer before reporting `Done:`.

### skein (owner: skein)

1. **`Io` and `FileIo` operation tokens can be equal on one kernel.** Seen. `skein-io/src/file_layer.rs`, `next_op` counts from 1. Two callers tag tokens by hand. (Debt D044.)
2. **The oauth client hangs a direct caller.** Seen. `skein-oauth/src/machine.rs`: `Client::step` returns early outside `Idle` for `SignIn`, `Refresh` and `Revoke` (`if self.phase != Phase::Idle { return; }`), with no terminal.
3. **A trace prints a stored token record in full.** Seen. skein-io's `file::Request` derives `Debug`, so a trace of `Store { bytes }` shows the bytes.
4. **`issuer_endpoint` admits `https://` alone and `https://a@b`.** Not re-checked. `skein-oauth`.
5. **The browser script names a profile that does not exist.** Seen. `skein/scripts/browser.sh` runs `--profile browser`, but `.config/nextest.toml` has only `browser-skein`, `-jig` and `-temper`.
6. **llm-connection's `valid()` accepts limits that `Component::new` refuses.** Not re-checked.
7. **forge-connection:** a `Call` dropped while Closing gets no terminal, and a duplicate runtime `Add` panics only in tests. Not re-checked.
8. **forge-setup:** an invalid `Add` is dropped; `Remove` fabricates `Stage::Admin`/`"remove"`; an unusable io or TLS limit is reported as `Unusable::Zero`. Not re-checked.
9. **forge-client admits a `Config` by `size_of::<Writer>()`.** Seen. `skein-forge-client`, `identity.rs`. Admission depends on the platform.
10. **The GitHub walker's stack ignores the configured depth.** Seen. It is sized by `github::limits::DEPTH` (10) whatever `limits.json.depth` says.
11. **Forgejo paths are capped at 12 segments,** so deep files are refused as `Malformed`. Not re-checked.
12. **skein-json's collector pops the caller's `below` queue** to take back a demand. Not re-checked.
13. **skein-kv deep-copies queued operations on every commit and scans in O(n²).** Not re-checked (debt D017 mentions it).
14. **The fake GitHub seeds five RNGs with the same `seed`.** Seen. `skein-fake-forge-github`: `delivery.rs`, `installations.rs`, `logs.rs`, `pages.rs`, `service.rs`. Not re-checked: the fake Forgejo closing on half the timeouts.
15. **skein-io:** a `Window` on a gone stream submits `Fd(-1)`, and `Io::new` panics at `sockets ≥ 2^29`. Not re-checked.
16. **skein-shell's `open_child_pidfd` off Linux returns `Error::Other(45)`** (Darwin's ENOTSUP as a bare number). Seen. Not re-checked: the readiness slot table's wrapping generation.
17. **skein-lib's `CryptoRng` derives `Clone`,** so a clone repeats the stream (against `skein/docs/design/lib.md`, 8). Seen.
18. **Two codec families share one golden directory.** Seen. `skein/tests/codec/tests/suite/drift.rs` checks both `every-v1` and `secrets-v1` against `golden/v1`, while the CLI writes `golden/v<N>`. Not re-checked: `patch_check`'s leftover temp crates, and the `Stack` price in `run/heap.rs`.
19. **Panics on odd input.** Seen: skein-browser's `contains` calls `windows(needle.len())`, which panics on an empty needle. Not re-checked: ui-web's `pixels` on a NaN size, and fake-peers' `panic!` for impossible io cells.
20. **skein-world does not build off unix** despite its `cfg(unix)` fences. Not re-checked.

