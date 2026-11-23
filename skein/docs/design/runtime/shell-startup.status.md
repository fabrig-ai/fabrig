# shell-startup: status

2026-11-23. Where the tree's code lags or departs from [shell-startup.md](shell-startup.md).

- **Not yet:** 1: the configured peer names resolved at startup (step 5).
  1.4: the helper's dispatch and the startup check, `Kernel::can_restrict`.
  [in flight: change 9014]
  1.4: the service's own cgroup opened at startup.
  1.6: that macOS answers `ENOPROTOOPT` to `SO_ACCEPTCONN` is
  read from published XNU, whose `main` may lag `macos-26`'s kernel, and
  no recorded run shows it: a Linux test holds `listening`
  (`skein-shell/src/listener_calls.rs`) on both answers, and only the
  constant that selects macOS's goes unrun.
- **Departs:** 1.2: `read_trust_roots` gives its caller the counts, and only
  `smith-agent-document`'s `config/native.rs`, `trust_sources`, says them,
  through the shell's re-export too. temper's own two readers
  (`temper/crates/temper-engine-shell/src/serve.rs`, `trust`, and
  `temper-fetch-shell/src/fetch.rs`) say nothing of a malformed block, where
  the section has startup say the counts: the request `first-usable-model` of
  2026-11-06 asked only that smith's line go quiet. Seen by reading.
  1.7: the user's request of 2026-11-04 asked each part of its
  round to end with fewer lines than it started; the part that built `store_file`
  and `bytes::concat` ended 190 lines up, 215 in skein against 25 down
  in smith, since a host and its first tests replaced two copies.
   the `debt-28` request of 2026-11-06 has each of its items end with fewer
  lines than it started; the keeper's fence ended 48 lines of Rust up, 9 in code
  and 39 in tests, since a change of path removes no body.
   the shared-posix-call gap remains `readiness.status.md`, 5.
