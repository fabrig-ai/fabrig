# tooling: status

2026-11-27. Where temper's code lags or departs from [tooling.md](tooling.md).

- **Not yet:** 2: no worker sets `TEMPER_CLOSE_OUTDATED=3` [in flight: change 21256].
- **Not yet:** 4.2, *`result.json`, exit 3, outdated*: fabrig's checked close writes no outdated finding [in flight: change 21255].
- **Not yet:** 4.2, *Checked against its attempt*: the engine reads an outdated finding as the command's failure [in flight: change 21257].


- **Not yet:** 1, 3 to 6: the `prepare` and `key` commands and their
  contract (`../domain/worker/jobs.md`): no project names one, and the
  worker's prepare kind is temper's own alone, with no key kind,
  though the job capability that would run them is built
  (`temper-worker-domain-tooling`). 2: the project's `prepare_timeout`
  bounds no command: a command's time is its job's deadline. 6:
  `conflict_files` is no limit, on `files`, `settled` or `left`.
- **Departs:** 2: every command's `PATH` is `/usr/local/bin:/usr/bin:/bin`, and 6:
  `tooling_file_bytes` and `tooling_log_bytes` are 1 MiB and 64 KiB, the
  small profile's constants with no override
  (`temper/crates/temper-service/src/limits.rs`, `tooling`), because the
  worker's configuration has no member for them (`configuration/worker.md`, section 1 and `configuration/limits.md`
  ), which `tooling.md`, section 7 leaves open: the merge kind, the
  first built, keeps the profile's constants. 3: a job's
  directory is its checkout's, each repository at `<name>/`, the home
  the repository its items name `home`, and none under `repos/`
  (`temper/crates/temper-worker-domain/src/jobs.rs`), because the
  checkout lays a workspace's repositories out by name
  (`../domain/worker/checkouts.md`) and no kind built needs `repos/`.
