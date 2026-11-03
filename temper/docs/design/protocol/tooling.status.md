# tooling: status

2026-11-03. Where temper's code lags or departs from [tooling.md](tooling.md).

- **Not yet:** 1, 3 to 6: the `prepare` and `key` commands and their
  contract, since neither kind is built (`../domain/worker.md`, 4.4): no
  project names a command, and no worker declares a kind, though the
  job capability that would run them is built
  (`temper-worker-domain-tooling`). 4.1: the `merge` command, its kind
  and the project's `merge_job`. 2: a command's time is the job's
  deadline, never yet the project's `prepare_timeout`. 6:
  `conflict_files` is no limit.
- **Departs:** 2: every command's `PATH` is `/usr/bin:/bin`, and 6:
  `tooling_file_bytes` and `tooling_log_bytes` are 1 MiB and 64 KiB, the
  small profile's constants with no override
  (`temper/crates/temper-service/src/limits.rs`, `tooling`), because the
  worker's configuration has no member for them (`configuration.md`,
  sections 4 and 7) until a kind is built (section 7). 3: a job's
  directory is its checkout's, each repository at `<name>/`, the home
  the repository its items name `home`, and none under `repos/`
  (`temper/crates/temper-worker-domain/src/jobs.rs`), because the
  checkout lays a workspace's repositories out by name
  (`../domain/worker.md`, section 5) and no kind built needs `repos/`.
