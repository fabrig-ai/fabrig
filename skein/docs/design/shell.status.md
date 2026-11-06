# shell: status

2026-11-06. Where skein's code lags or departs from [shell.md](shell.md).

- **Not yet:** 6: the configured peer names resolved at startup (step 5).
  6.4: the helper's dispatch, and the service's own cgroup opened at
  startup. 6.6: that macOS answers `ENOPROTOOPT` to `SO_ACCEPTCONN` is
  read from published XNU, whose `main` may lag `macos-26`'s kernel, and
  no recorded run shows it: a Linux test holds `listening`
  (`skein-shell/src/listener_calls.rs`) on both answers, and only the
  constant that selects macOS's goes unrun. 7.5: `linux` and `darwin` each
  still write eight functions that differ at most in a comment, a panic's
  words or a path: `usage`, byte for byte, because the `resources` it calls
  differs, and `close`, `count`, `effective_user`, `fd`, `nonblocking`,
  `status_flags` and `truncate`, which the `debt-27` plan's cut, scanning for
  what was the same byte for byte, took as differing or did not name. That
  plan's request has such code live once, and its item is deleted. Seen by
  reading. 9: temper's one story has not been seen green on the macOS workflow
  (`temper/docs/design/domain/testing.md`, 2.8), and no macOS runner runs
  skein's suites. No build of `darwin` over `posix` is on record: no builder
  is known to hold `aarch64-apple-darwin`, so the first is that workflow's
  compile step, where a red blocks nothing, and whether it has run is unknown.
- **Departs:** 6.2: `read_trust_roots` gives its caller the counts, and only
  `smith-agent-shell`'s `trust_sources` says them. temper's own two readers
  (`temper/crates/temper/src/lib.rs`, `trust`, and
  `temper-forge-setup-shell/src/fetch.rs`) say nothing of a malformed block, where
  the section has startup say the counts: the request `first-usable-model` of
  2026-11-06 asked only that smith's line go quiet. Seen by reading.
  6.7: the user's request of 2026-11-04 asked each part of its
  round to end with fewer lines than it started; the part that built `store_file`
  and `bytes::concat` ended 190 lines up, 215 in skein against 25 down
  in smith, since a host and its first tests replaced two copies.
  7.5: the `debt-28` request of 2026-11-06 has each of its items end with fewer
  lines than it started; the keeper's fence ended 48 lines of Rust up, 9 in code
  and 39 in tests, since a change of path removes no body.
