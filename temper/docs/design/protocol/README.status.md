# README: status

2026-11-09. Where temper's code lags or departs from [README.md](README.md).

Section 4.3's feature-free `temper-agent` is present, with its shared
help, run, list, relay, program-child and argv0 Claude launcher
dispatch. Its parser table and twenty-row suite cover its forms,
refusals and modes. 4.1's two closure tests are present, each with its
row: `binaries.rs`'s `temper-engine` and `closure.rs`'s `temper-agent`,
the agent's with no exception. The client's and the worker's rows
remain M5's. The box's modes and the worker's current program are
unchanged; naming `temper-agent` is M3's
(`../draft/deployment-binaries.execution.md`, 1).


Section 8.2's wait and recovery notes, ordered gate reasons, one-second
threshold and suppression of repeats are present. Service tests step
supplied times through refused startup inputs and a held watermark
input; the journal-stop test checks the engine's end and reason. The
binary's line tables cover the notes and journal exits. These checks
establish the diagnostics below the real loop, not the cause of the
incident's unexplained 503 (`../domain/root.md`, section 13).

- **Not yet:** 8.2: the accepted `core-after-project` decision calls for
  a distinct `WatchRefusal::NotReady`, in place on jig's client wire 19,
  and a not-ready ctl deadline line pointing to serve's log. The core's
  refusal vocabulary has no such variant, and
  `temper-service/src/clients/translation.rs`, `not_ready`, still refuses
  watches as `Busy`. The sign-in line is present, but other calls can
  still claim all watches are in use while the gate is shut.

Section 4's `temper [ADDRESS]` command is present, including bare
`temper` choosing the TUI only when stdin and stdout are terminals.
The TUI's later passes and post-landing walk are the client's section
11 work (`../client/README.md`).

Section 4's smith's agent modes are present: `main` gives
`--mcp-relay` and `--program-child` to smith's agent shell after its
`argv[0]` dispatches and before the table (`crates/temper/src/main.rs`).
The `suite` binary's table runs the built binary in each, the program
child over a scripted Claude Code's `auth status`, read `Usable` and
not signed in by smith's status check
(`crates/temper/tests/suite/agent_modes.rs`).

Section 8 has no known gap in orderly worker shutdown: serve cancels the
runs and waits for every agent process, including a spawn in flight,
before aborting the remaining worker resources. The real-loop held-turn
test checks Claude's interrupt and end before serve ends drained. The
live proof and forced-stop containment remain in `setup.md`, section 15.

Section 8 also has no known gap in the task's accounting and saved work
at an orderly stop. The one-process drain table holds chat and coding
agents mid-turn, drains and restarts the same disk and store, and checks
one interrupted terminal, no claim settled at restart, unchanged task
tries and saved work, and a new attempt. Its coding row checks unchanged
change counts and saved branch head, a writer slot owned by the new
attempt, and a prepared checkout at that head with the saved edit.
This is simulated proof; the live no-survivor proof above remains
separate. A worker stopped apart from its engine retains section 15.7's
deferred failure accounting.

Section 15.3 has no known gap in a link's loss lines: the worker prints
`lost` once each linked link is lost, the engine `worker lost` for each
channel that said hello and `worker opening ended` for an opening, once
until that worker's reason changes or it links, and a failed workers'
listener prints its line when it fails (`temper-service/src/worker.rs`,
`link`; `workers.rs`, `route`; `temper-engine-shell/src/serve/host.rs`).
The binary's line tables
`worker_loss_lines_name_each_cause_and_refusal_exactly`,
`engine_worker_loss_lines_name_each_cause_and_refusal_exactly` and
`worker_listener_failures_print_the_exit_lines_words_at_once`, and
`temper-service`'s
`a_linked_loss_is_noted_once_and_a_refused_dial_has_no_lost_line`,
`worker_opening_ends_are_suppressed_by_name_and_reason_until_hello` and
`a_workers_listener_failure_is_noted_before_the_service_finishes` hold
it. The request `worker-relinks` of 2026-11-08 also asked that the
engine stop on SIGTERM within its stop grace, where a service manager
killed one after 90 seconds: the workers' part closes within the
stall deadline of the close (jig's `protocol/channel.md`, 10.2), but no
stop test of temper's shows the whole stop within a bound, and the
engine has no stop deadline of its own (section 14).

- **Not yet:** 15.6: the retained case 4.6 process story is ignored until
  its fixture stops expecting models in the startup configuration
  (`../domain/testing.status.md`, 2.6); jig's fleet takes a draining
  worker out of placement before its runs end (jig's
  `domain/engine.md`, section 8), and the real-loop story supplies no
  proof of that process boundary. No reference-case story runs in this tier; the task-per-worker,
  lost-worker, hung-worker and late-worker boundary stories remain.

- **Not yet:** 4: `temper versions`, `temper upgrade --check` and `--back`, and a patching start's lines: the binary's table has no row for them (`crates/temper/src/commands.rs`). A link the forge's restore cannot make still panics, with serve's fail-stop report and its backtrace, where jig's start's are refusals (`../domain/root.status.md`, 9.1). 6: the setup adapter's close kept as requested apart from `Close` sent, and sent once the host's room permits: `close()` sends `Close` at once (`temper-service/src/setup.rs`). 9: the baseline's set and its rehearsal: no owner's kept texts (`--baseline`), no real patch sets in place of jig's `PatchSets::NONE`, no gate check of patches, no golden store.
- **Departs:** 4: a group's file holds its kind alone, which the box writes at `start FORGE owned|fake`, and `temper forge fetch`, `prepare` and `run` compose the setup entry from `application.machine` by that kind (`temper-engine-shell/src/forge.rs`, `entry`; `temper-forge-setup/src/machine.rs`, `group_file`); the code records no reason (`forge.status.md`, 2.8). 6, 7: `temper-worker-domain/src/translate.rs` rebuilds smith's `SourceName` with `.expect("same source name grammar")`, so a stored name smith refuses aborts the process, where `jig/crates/jig-protocol-worker-agents/src/sources.rs` answers its command `Unconfigured`; no reason is recorded. 15.5, 15.6: the cluster's description keeps the lab's format, cost label, file name, `{experiment}` binding and empty fields, and the ignored case 4.6 story uses it (`tests/support/case.rs`); their purposes, the lab and that case, are retired, and the code awaits removal with this design.

Section 4.2's `temper-engine` is present, with 4.1's rules: its
table of `help`, `serve`, `versions`, the forge's six verbs and
`internal forge-child`, its `help` and forge rows the constants the
box's table holds; `--version` from `temper-version`; `versions`' lines;
the unknown command's refusal; and a worker alone's and a standalone
document refused before any other check, naming `temper-worker serve`
and the box's `temper serve`, by `serve` and by each forge command that
reads the document. Its step test runs the built binary over a table of
those cases (`crates/temper-engine/tests/suite/main.rs`). `serve` and
the forge's commands are `temper-engine-shell`'s, which the box's
`temper` runs too, under its own name and with its worker's half
(`crates/temper/src/worker_half.rs`). `temper-service`'s `worker`
feature is on in the box's build and the worlds' and off in
`temper-engine`'s, where a start of any shape but an engine alone is
refused. `tests/boundary/tests/binaries.rs` holds `temper-engine`'s
row: the worker's domain and smith's agent excluded, but its four named
exceptions (4.1). It also holds the box's machine, `temper-box`, and the
fakes' crates, `temper-fake` with skein's fake forge, LLM and OAuth
beneath it, which `temper-engine-shell`'s check of the document reads
for its `box` limits and `demo` member, as the box's does (section 5;
`../draft/deployment-binaries.status.md`). The gate builds the
workspace whole, so its run of the step test runs a binary built with
`worker` on; the feature-off service is compiled by `temper-engine`'s
own build alone, as 4.1 accepts.

- **Not yet:** 4.2: `temper-engine upgrade --check` and `--back`: jig's
  rehearsal and going back (jig's `protocol/store.md`, 10.5) are not
  built, so neither binary's table has the row. 4.1: `binaries.rs` has
  no row for `temper-client`, which is M5's (below).

Section 4's deployment client, `temper-client`, is present: its table
of `help`, `[ADDRESS]` and `ctl`, the box's rows run by the box's code,
its help and refusals naming `temper-client`, and `--version`, in step
tests of its package. Section 5's
`temper-shell` holds the table's types and printing, the shared rows,
`help`'s answers, the three directories and the engine's link file, and
the box's `temper` prints from it. Built alone, the client links none of
the engine's, the worker's, smith's agent, the box or the forge's setup,
and `temper-protocol-records` without `forge` (`cargo tree -p
temper-client`). No test holds that closure yet: its row in
`binaries.rs` is M5's (`../draft/deployment-binaries.execution.md`). Section 4's client
story is written and compiled in the process tier, and runs after
landing.

- **Departs:** 4: `temper-client`'s story runs the box's `temper serve` as its engine
  (`crates/temper/tests/suite/client_binary.rs`), written before `temper-engine`
  was built; M5 switches it to `temper-engine`
  (`../draft/deployment-binaries.execution.md`, M5).
- **Departs:** 4, 5: `temper-client`'s COMMIT is its own build script's
  (`crates/temper-client/build/main.rs`, `TEMPER_CLIENT_COMMIT`), a second copy
  of `temper-version`'s, which it doesn't link: M1 built it while `temper-version`
  was being built beside it, and no plan has switched it yet.
- **Departs:** 4: beyond the crates section 4 says it links, `temper-client` links:
  - `jig-protocol-accounts`, an engine crate of jig's (section 5), and through it
    `jig-protocol-config`, `jig-protocol-engine-clients`,
    `jig-protocol-engine-clients-http`, `jig-protocol-store` and jig's
    `jig-domain-*` crates, with skein's OAuth, TLS, link and kv crates beneath
    them.
    `temper-client-shell-cli`'s manifest names it, though no code of that crate
    uses it; no reason is recorded.
  - `temper-settings-form`, through `temper-client-tools`, which section 5 has
    the client and the engine share; and `temper-protocol-records`, through
    `temper-client-domain`, for the forges' pasted bundle
    (`../client/README.status.md`, 2).
  - `jig-protocol-transcript` and `smith-transcript`, below `jig-client`
    (`../client/README.md`, section 2).

  M5's boundary test names whichever stay as the client's exceptions, with their
  reasons (`../draft/deployment-binaries.md`, section 4).
