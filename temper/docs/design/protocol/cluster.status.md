# cluster: status

2026-11-10. Where the tree's code lags or departs from [cluster.md](cluster.md).

- **Not yet:** 6: the retained case 4.6 process story is ignored until
  its fixture stops expecting models in the startup configuration
  (`../testing/process-tier.status.md`); jig's fleet takes a draining
  worker out of placement before its runs end (jig's
  `domain/engine/fleet.md`), and the real-loop story supplies no
  proof of that process boundary. No reference-case story runs in this tier; the task-per-worker,
  lost-worker, hung-worker and late-worker boundary stories remain.

- **Departs:**  a group's file holds its kind alone, which the box writes at `start FORGE owned|fake`, and `temper-box forge fetch`, `prepare` and `run` compose the setup entry from `application.machine` by that kind (`temper-engine-shell/src/forge.rs`, `entry`; `temper-forge-setup/src/machine.rs`, `group_file`); the code records no reason (`forge/engine.status.md`, 8).  `temper-worker-domain/src/translate.rs` rebuilds smith's `SourceName` with `.expect("same source name grammar")`, so a stored name smith refuses aborts the process, where `jig/crates/jig-protocol-worker-agents/src/sources.rs` answers its command `Unconfigured`; no reason is recorded. 5, 6: the cluster's description keeps the lab's format, cost label, file name, `{experiment}` binding and empty fields, and the ignored case 4.6 story uses it (`tests/support/case.rs`); their purposes, the lab and that case, are retired, and the code awaits removal with this design.

`binaries.md`, section 2's `temper-engine` is present, with `binaries.md`, 1's rules: its
table of `help`, `serve`, `versions`, the forge's six verbs and
`internal forge-child`, its `help` and forge rows the constants the
box's table holds; `--version` from `temper-version`; `versions`' lines;
the unknown command's refusal; and a worker alone's and a standalone
document refused before any other check, naming `temper-worker serve`
and the box's `temper-box serve`, by `serve` and by each forge command that
reads the document. Its step test runs the built binary over a table of
those cases (`crates/temper-engine/tests/suite/main.rs`). `serve` and
the forge's commands are `temper-engine-shell`'s, which the box's
`temper-box` runs too, under its own name and with its worker's half
(`crates/temper-box-shell/src/worker_half.rs`). `temper-service`'s `worker`
feature is on in the box's build and the worlds' and off in
`temper-engine`'s, where a start of any shape but an engine alone is
refused. `tests/boundary/tests/binaries.rs` holds `temper-engine`'s
row: the worker's domain and smith's agent excluded, but its four named
exceptions (`binaries.md`, 1). It also holds the box's machine, `temper-box-machine`, and the
fakes' crates, `temper-fake` with skein's fake forge, LLM and OAuth
beneath it, which `temper-engine-shell`'s check of the document reads
for its `box` limits and `demo` member, as the box's does (`components.md`;
`../draft/deployment-binaries.status.md`). The gate builds the
workspace whole, so its run of the step test runs a binary built with
`worker` on; the feature-off service is compiled by `temper-engine`'s
own build alone, as `binaries.md`, 1 accepts.
