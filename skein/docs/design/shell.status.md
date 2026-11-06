# shell: status

2026-11-06. Where skein's code lags or departs from [shell.md](shell.md).

- **Not yet:** 6: the configured peer names resolved at startup (step 5).
  6.4: the helper's dispatch, and the service's own cgroup opened at
  startup. 6.6: that macOS answers `ENOPROTOOPT` to `SO_ACCEPTCONN` is
  read from published XNU, whose `main` may lag `macos-26`'s kernel, and
  no recorded run shows it: a Linux test holds `listening`
  (`skein-shell/src/listener_calls.rs`) on both answers, and only the
  constant that selects macOS's goes unrun. 9: temper's one story has not
  been seen green on the macOS workflow
  (`temper/docs/design/domain/testing.md`, 2.8), and no macOS runner runs
  skein's suites.
- **Departs:** 6.2: `read_trust_roots` gives its caller the counts, and only
  `smith-agent-shell`'s `trust_sources` says them. temper's own two readers
  (`temper/crates/temper/src/lib.rs`, `trust`, and
  `temper-forge-setup-shell/src/fetch.rs`) say nothing of a malformed block, where
  the section has startup say the counts: the request `first-usable-model` of
  2026-11-06 asked only that smith's line go quiet. Seen by reading.
  7.5: the system parts have no design for the keeper's
  `poll_child`, `close_keeper_fd`, `abandon_binary` or `wait_readable`,
  but `skein-shell/src/process.rs`, `linux.rs` and `darwin.rs` expose
  them in the crate's public API through `lib.rs`; no reason is recorded
  (the code's side is
  `skein/docs/development/debt/D045-shell-keeper-syscalls.md`).
  6.7: the user's request of 2026-11-04 asked each part of its round to
  end with fewer lines than it started; the part that built `store_file`
  and `bytes::concat` ended 190 lines up, 215 in skein against 25 down
  in smith, since a host and its first tests replaced two copies.
