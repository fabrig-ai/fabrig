# service-process: status

2026-11-22. Where the tree's code lags or departs from [service-process.md](service-process.md).

- **Not yet:** 1.2: the accepted `core-after-project` decision calls for
  a distinct `WatchRefusal::NotReady`, in place on jig's client wire 19,
  and a not-ready ctl deadline line pointing to serve's log. The core's
  refusal vocabulary has no such variant, and
  `temper-service/src/clients/translation.rs`, `not_ready`, still refuses
  watches as `Busy`. The sign-in line is present, but other calls can
  still claim all watches are in use while the gate is shut.

`binaries.md`, section 5's `temper-worker` is present on `temper-worker-shell` and
`temper-service-shell`, with its step test (`bin/temper-worker/tests/suite/main.rs`).
`cluster.md`, 6's story on the deployment's binaries is written and compiled, and runs
after landing (`bin/temper-box/tests/process.rs`,
`a_task_runs_on_each_worker_of_the_deployments_binaries`).

`binaries.md`'s `temper [ADDRESS]` command is present, including bare
`temper` choosing the TUI only when stdin and stdout are terminals.
The TUI's later passes and post-landing walk are the client's section
11 work (`../client/README.md`).

`binaries.md`'s smith's agent modes are present: `main` gives
`--mcp-relay` and `--program-child` to smith's agent shell after its
`argv[0]` dispatches and before the table (`crates/temper-box-shell/src/binary.rs`).
The `suite` binary's table runs the built binary in each, the program
child over a scripted Claude Code's `auth status`, read `Usable` and
not signed in by smith's status check
(`bin/temper-box/tests/suite/agent_modes.rs`).

Section 1 has no known gap in orderly worker shutdown: serve cancels the
runs and waits for every agent process, including a spawn in flight,
before aborting the remaining worker resources. The real-loop held-turn
test checks Claude's interrupt and end before serve ends drained.
`bin/temper-box/tests/cluster_turn/unit_stop.rs` holds the process
boundary (`a_unit_stop_settles_a_workers_held_turn` in `temper-box::process`).
The live proof and forced-stop containment remain in `setup/README.md`, section 9.

Section 1 also has no known gap in the task's accounting and saved work
at an orderly stop. The one-process drain table holds chat and coding
agents mid-turn, drains and restarts the same disk and store, and checks
one interrupted terminal, no claim settled at restart, unchanged task
tries and saved work, and a new attempt. Its coding row checks unchanged
change counts and saved branch head, a writer slot owned by the new
attempt, and a prepared checkout at that head with the saved edit.
This is simulated proof; the live no-survivor proof above remains
separate. A worker stopped apart from its engine retains `cluster.md`, section 7's
deferred failure accounting.

`cluster.md`, section 3 has no known gap in a link's loss lines: the worker prints
`lost` once each linked link is lost, the engine `worker lost` for each
channel that said hello and `worker opening ended` for an opening, once
until that worker's reason changes or it links, and a failed workers'
listener prints its line when it fails (`temper-service/src/worker.rs`,
`link`; `workers.rs`, `route`; `temper-service-shell/src/host.rs`).
The binary's line tables
`worker_loss_lines_name_each_cause_and_refusal_exactly`,
`engine_worker_loss_lines_name_each_cause_and_refusal_exactly` and
`worker_listener_failures_print_the_exit_lines_words_at_once`, and
`temper-service`'s
`a_linked_loss_is_noted_once_and_a_refused_dial_has_no_lost_line`,
`worker_opening_ends_are_suppressed_by_name_and_reason_until_hello` and
`a_workers_listener_failure_is_noted_before_the_service_finishes` hold
it. Nor in `worker unsent`: the engine writes it once for each record its
link did not send, naming the record, its frame's bytes and the room or
the channel's reason (`workers.rs`, `route`, `Note::WorkerUnsent`;
`temper-service-shell/src/host.rs`, `note_line`), as
`worker_unsent_lines_name_the_record_bytes_and_bound` and
`each_unsent_record_reaches_the_operator_once` hold. The request `worker-relinks` of 2026-11-08 also asked that the
engine stop on SIGTERM within its stop grace, where a service manager
killed one after 90 seconds: the workers' part closes within the
stall deadline of the close (jig's `protocol/channel/worker-link.md`, 1.2), but no
stop test of temper's shows the whole stop within a bound, and the
engine has no stop deadline of its own (`README.md`, section 11).
