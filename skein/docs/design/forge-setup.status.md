# forge-setup: status

2026-11-04. Where skein's code lags or departs from [forge-setup.md](forge-setup.md).

- **Departs:** 7: the setup world plays the host component through HTTP
  bytes, but `skein/tests/forge/setup/src/world.rs` and `host.rs` write
  two worlds: Forgejo's reconcile stories in `tests/suite/reconcile.rs`
  run on the plans-only `World`, while `HostWorld` plays the host; no
  reason is recorded (the code's side is
  `skein/docs/development/debt/D031-forge-setup-two-worlds.md`).
