# forge-setup: status

2026-11-04. Where skein's code lags or departs from [forge-setup.md](forge-setup.md).

- **Not yet:** 8: the `setup-migrate-*` facts.
  6.1, 6.3: the crate gives the checks and an `Add` one
  refuses is ignored, the owner having checked first, but
  `skein-forge-setup/src/host/limits.rs`'s `check_forge` is `pub(super)`:
  an owner reaches it only by building a `Host`, and
  `temper/crates/temper-service/src/setup/runtime.rs` records an `Add` as
  served whether the host took it or not.
- **Departs:** 7: the setup world plays the host component through HTTP
  bytes, but `skein/tests/forge/setup/src/world.rs` and `host.rs` write
  two worlds: Forgejo's reconcile stories in `tests/suite/reconcile.rs`
  run on the plans-only `World`, while `HostWorld` plays the host; no
  reason is recorded (the code's side is
  `skein/docs/development/debt/D031-forge-setup-two-worlds.md`).
