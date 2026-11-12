# README: status

2026-11-12. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 5: agents spawned from the pinned copies [in flight: change 1749].
- **Not yet:** 5: the ending's drain, git's saves while stopping and the abort at the stop bound: git refuses every operation once the worker stops [in flight: change 1767].
- **Not yet:** 5: the worker's unit under a service manager in `temper help server` [in flight: change 1757].

`binaries.md`, section 3's feature-free `temper-agent` is present, with its shared
help, run, list, relay, program-child and argv0 Claude launcher
dispatch, the launcher and the substitution `temper-launcher`'s, and
`--version` from `temper-version`, with no build script of its own
(`crates/temper-agent-shell`'s tests, and its package's suite at
`bin/temper-agent`). Its parser
table and twenty-row suite cover its forms, refusals and modes. The engine's and the
agent's rows of `binaries.md`, 4's closure test hold, and `components.md`'s
crates hold them: the engine's, since
`temper-fetch-shell` links no smith crate and the forge's setup links
no launcher; and the agent's, its row's temper
crates, no jig crate and no exception, since `temper-shell` links
skein's crates and `temper-docs` alone and jig's form of the three
directories is `temper-service-shell`'s `jig_directories`, which the
box's `temper_box_shell::directories` re-exports. `README.md`, section 5's model-list launch uses the worker's named
agent program, with the box's `internal agent-list AGENT.json` or the
deployment's `AGENT.json` arguments, and smith's `--list` before them
(`temper-service/src/worker.rs`, `list_launch`). The service's
`a_one_argument_agent_lists_with_its_configuration_alone` and
`list_reads_launch_temper_without_credentials_or_a_run_slot` hold both
forms; other argument forms and unusable grants fail without IO.

Section 6.2's wait and recovery notes, ordered gate reasons, one-second
threshold and suppression of repeats are present. Service tests step
supplied times through refused startup inputs and a held watermark
input; the journal-stop test checks the engine's end and reason. The
binary's line tables cover the notes and journal exits. These checks
establish the diagnostics below the real loop, not the cause of the
incident's unexplained 503 (`../domain/root.md`, section 12).

- **Not yet:** 6.2: the accepted `core-after-project` decision calls for
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

Section 6 has no known gap in orderly worker shutdown: serve cancels the
runs and waits for every agent process, including a spawn in flight,
before aborting the remaining worker resources. The real-loop held-turn
test checks Claude's interrupt and end before serve ends drained. The
live proof and forced-stop containment remain in `setup/README.md`, section 9.

Section 6 also has no known gap in the task's accounting and saved work
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
stall deadline of the close (jig's `protocol/channel/README.md`, 9.2), but no
stop test of temper's shows the whole stop within a bound, and the
engine has no stop deadline of its own (`README.md`, section 11).

- **Not yet:**  the box upgrade commands remain `binaries.status.md` (Not yet, the whole document). The restored-forge link gap remains `../domain/root.status.md`, 8.1. 4: the setup adapter's close kept as requested apart from `Close` sent, and sent once the host's room permits: `close()` sends `Close` at once (`temper-service/src/setup.rs`).
- **Departs:**  the group-file departure remains `forge/engine.status.md`, 2 and 8. 4, 5: `temper-worker-domain/src/translate.rs` rebuilds smith's `SourceName` with `.expect("same source name grammar")`, so a stored name smith refuses aborts the process, where `jig/crates/jig-protocol-worker-agents/src/sources.rs` answers its command `Unconfigured`; no reason is recorded.  the retired cluster-description departure remains `cluster.status.md`, 5 and 6.

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
