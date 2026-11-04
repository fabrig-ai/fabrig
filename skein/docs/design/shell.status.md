# shell: status

2026-11-04. Where skein's code lags or departs from [shell.md](shell.md).

- **Not yet:** 6: the configured peer names resolved at startup (step 5).
  6.4: the helper's dispatch, and the service's own cgroup opened at
  startup. 9: the macOS workflow
  (`temper/docs/design/domain/testing.md`, 2.8) compiles the members that
  reach rustls and runs temper's one story, neither yet seen green, and
  no macOS runner runs skein's suites.
- **Departs:** 7.5: the system parts have no design for the keeper's
  `poll_child`, `close_keeper_fd`, `abandon_binary` or `wait_readable`,
  but `skein-shell/src/process.rs`, `linux.rs` and `darwin.rs` expose
  them in the crate's public API through `lib.rs`; no reason is recorded
  (the code's side is
  `skein/docs/development/debt/D045-shell-keeper-syscalls.md`).
  6.7: the user's request of 2026-11-04 asked each part of its round to
  end with fewer lines than it started; the part that built `store_file`
  and `bytes::concat` ended 190 lines up, 215 in skein against 25 down
  in smith, since a host and its first tests replaced two copies.
