# forge-setup: status

2026-11-11. Where skein's code lags or departs from [README.md](README.md).

- **Not yet:**  the starter-workflow observation remains `forgejo.status.md`, 2. 8, and `skein/docs/design/forge/README.md`, 4.3: the migration observations remain `forgejo.status.md`, 2.
  4.1, 4.3: the crate gives the checks and an `Add` one
  refuses is ignored, the owner having checked first, but
  `skein-forge-setup/src/host/limits.rs`'s `check_forge` is `pub(super)`:
  an owner reaches it only by building a `Host`, and
  `temper/crates/temper-service/src/setup/runtime.rs` records an `Add` as
  served whether the host took it or not.
   the site-admin path-read observation remains `forgejo.status.md`, 2.
