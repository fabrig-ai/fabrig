# README: status

2026-11-07. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 15.6: the retained case 4.6 process story is ignored until
  jig's fleet removes a draining worker from placement before its runs
  end. No reference-case story runs in this tier; the task-per-worker,
  lost-worker, hung-worker and late-worker boundary stories remain.

- **Not yet:** 8: a stop spends no try of a run's task: jig's core
  settles nothing at the stop (jig's `domain/engine.status.md`, 6.1), so
  the next start settles a run that committed a turn as lost, and no
  test holds the stop for a coding run. The plan `ui-stop-coding-row`
  builds it.

- **Not yet:** 4: `temper versions`, `temper upgrade --check` and `--back`, and a patching start's lines: the binary's table has no row for them (`crates/temper/src/commands.rs`). 6: the setup adapter's close kept as requested apart from `Close` sent, and sent once the host's room permits: `close()` sends `Close` at once (`temper-service/src/setup.rs`). 9: the baseline's set and its rehearsal: no owner's kept texts (`--baseline`), no real patch sets in place of jig's `PatchSets::NONE`, no gate check of patches, no golden store.
- **Departs:** 4: a group's file holds its kind alone, which the box writes at `start FORGE owned|fake`, and `temper forge fetch`, `prepare` and `run` compose the setup entry from `application.machine` by that kind (`temper/src/forge.rs`, `entry`; `temper-forge-setup/src/machine.rs`, `group_file`); the code records no reason (`forge.status.md`, 2.8). 6, 7: `temper-worker-domain/src/translate.rs` rebuilds smith's `SourceName` with `.expect("same source name grammar")`, so a stored name smith refuses aborts the process, where `jig/crates/jig-protocol-worker-agents/src/sources.rs` answers its command `Unconfigured`; no reason is recorded. 15.5, 15.6: the cluster's description keeps the lab's format, cost label, file name, `{experiment}` binding and empty fields, and the ignored case 4.6 story uses it (`tests/support/case.rs`); their purposes, the lab and that case, are retired, and the code awaits removal with this design.
