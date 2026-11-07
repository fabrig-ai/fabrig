# Slow tier runner

Provisional, 2026-10-29. Run one tree's slow tier with its prerequisites.

## 1. In one page

- **One entry point:** from a fabrig checkout root,
  `tools/tiers/run TREE TIER [-- NEXTEST_ARGS...]` builds prerequisites and
  runs `cargo nextest run -p PKG ... --profile TIER-TREE --no-fail-fast`,
  using the pair's sorted package names from `tools/tiers/packages.toml`,
  followed by the extra arguments unchanged. The map is generated from the
  packages with selected tests in each profile's workspace JSON listing.
- **The config owns the pairs:** the root `.config/nextest.toml` profile
  tables whose names contain a hyphen supply the pairs, split at the last
  hyphen into tier and tree. Unknown pairs and pairs missing a valid package
  set fail before any command runs.
- **Plain commands by default:** every build and nextest command runs as it
  stands, or behind the wrapper when `TIERS_WRAPPER` sets one (section 3).
- **Builds are sequential:** commands run one at a time; where the runner
  runs the built bundle tool itself, it runs directly, past the wrapper.
  Cargo metadata builds nothing and runs directly to discover the target
  directory.
- **Disk before each of those commands:** with `TIERS_DISK_BYTES` set
  (section 3), wait for that many bytes available on `/`, checking once per
  minute and printing each wait.
- **Visible evidence:** print each step's command; nextest prints its
  summary, and what a wrapper prints stays visible. Prerequisite failures
  name their step.

## 2. Prerequisites

Paths to source assets are relative to the checkout root. Build artifacts
and bundle output directories are beneath `cargo metadata`'s
`target_directory`, including when Cargo config relocates it. All builds
use Cargo's default dev profile, as the trees' browser scripts do, but
temper's playground: `bundle.sh --playground` builds its page and its
world in their own profiles (temper's `docs/design/playground.md`, 7), so
that the tier walks and measures what a visitor downloads.

| Pair | Prerequisites in order | Bundle environment |
| --- | --- | --- |
| browser-skein | wasm `skein-ui-example-shell-web`; `skein-ui-bundle`; run the tool; `skein-ui-example-server` | `SKEIN_UI_EXAMPLE_BUNDLE`, target's `ui-example-bundle` |
| browser-jig | wasm `jig-client-test-shell-web`; `skein-ui-bundle`; run the tool; `jig-test` | `JIG_TEST_BUNDLE`, target's `jig-test-bundle` |
| browser-temper | wasm `temper-client-shell-web` with `loopback-navigation`; `skein-ui-bundle`; run the tool; `temper/scripts/bundle.sh --playground`; `temper` | `TEMPER_TEST_BUNDLE`, target's `temper-bundle`; `TEMPER_TEST_PLAYGROUND`, target's `temper-playground` |

The wasm target is `wasm32-unknown-unknown`. Each tool invocation uses the
module's underscored package name, `/bundle/` prefix, and the old browser
script's stylesheets, pictures, files and noscript text. Temper matches
`temper/scripts/bundle.sh --loopback`, without embedding the bundle in the
native binary. Existing bundle outputs are rebuilt on every invocation.

### 2.1 Native binary lists

A tier's tests may execute binaries that its `-p` package selection does not
build. Each pair lists these in the `[binaries]` table of
`tools/tiers/packages.toml`: an array of `{ package = "...", bin = "..." }`
records, sorted and unique by package then binary. Every configured pair
has a list, including an explicit empty list when none is needed. Names
contain only ASCII letters, digits, underscores and hyphens; two packages
may not provide the same binary within a pair. Missing/unknown pair lists
and invalid records are refused before any command, including in verification.
An optional `env` names a test environment variable (uppercase ASCII with
`_TEST_` in its name). After building the binary, set that variable to its
absolute `target_directory/debug/BINARY` path, replacing any inherited value.
Variable names are unique within a pair. `process-temper` sets
`TEMPER_TEST_CLAUDE` this way for its two program-login tests, with
`smith-fake-claude`'s `fake-claude`, the one adapter both trees run.

After any bundle step, build each listed binary in order with
`cargo build -p PACKAGE --bin BINARY`, then run nextest. The browser
servers in the table above are native list entries. `process-temper` also
lists `smith-fake-claude`'s `fake-claude`: temper's
fake-backend tests execute that adapter as the `temper` binary's sibling.
A failed binary build returns 2 and starts no later step.

All other configured pairs run nextest alone. Tier tests also use binaries
from their own packages (`harness_fixture`, `jig-test`,
`smith`, `temper`); Cargo/nextest builds those integration-test targets.
The browser servers cross package boundaries and require the explicit builds
above. Smith's selected live tests use the `smith` package's own binaries;
the separate `smith-fake-claude` package's binary tests are not selected by
any of these pairs. No other native binary requirement was found across
the configured pairs.

## 3. Environment and output

Every child inherits the caller's environment with `NEXTEST_DOUBLE_SPAWN=0`
(skein/docs/foundation/testing-strategy.md, 8). The chosen bundle variable
is set to the absolute output directory for nextest, replacing any inherited
value. `TEMPER_BUNDLE=none` stays as the caller's environment sets it: temper's
native binary must not embed a person's browser bundle. Chromium, fonts and
live sign-ins remain the caller's environment prerequisites; this tool
installs nothing and performs no sign-in.

Two settings come from that environment. Unset, each does nothing, and no
default names a tool:

- **`TIERS_WRAPPER`, the wrapper:** a command prefix, split on spaces and put
  before every Cargo build, `temper/scripts/bundle.sh --playground` as one
  command, every nextest run and listing, and the `cc` build of the CPU wrapper
  (sections 7 and 7.5). Unset or empty, those commands run plain. Cargo metadata
  and the built bundle tool the runner runs itself never go through it. Sections
  1 to 6 call this setting the wrapper; from section 7 on it is named
  `TIERS_WRAPPER`, and a bare "wrapper" is the CPU wrapper,
  `tools/tiers/cpu-wrapper.c`.
- **`TIERS_DISK_BYTES`:** the minimum free bytes on `/` before each of those
  commands, wrapper or no wrapper. Unset or empty, nothing waits. A value
  that is not a non-negative integer is a failed disk check (section 4).

The runner sets umask 077, closes child stdin, and leaves child output visible,
what a wrapper prints around a command included. Commands remain sequential,
and their inputs must stay still until completion. A low disk check sleeps 60
seconds, checks again, and prints another line only if disk remains low; there
is no disk timeout.

## 4. Exit codes

Return nextest's exit code, including a zero-selection or test-failure code.
Verification returns 0 for identical selections and 1 for differences,
naming every missing or added test by package, binary and test name.
Return 2 for a refused pair, invalid invocation/configuration, failed metadata,
failed disk check, command launch failure, or failed prerequisite; report the
failed step and do not start later steps. The runner does not retry failures.

## 5. Offline checks

`python3 -I -B tools/tiers/test_run.py` uses temporary fixture checkouts
inside this checkout, fake commands on PATH, and a mocked disk sleep.
It checks configuration-derived pairs, prerequisite order, arguments and
environment, plain commands with no wrapper and a fake wrapper's prefix on
each command, map refusal and sorted package arguments, verification with fake
JSON listings, the disk wait on and off and refusal/failure exit codes without
builds, network access, Chromium or credentials. It clears any inherited
`TIERS_WRAPPER` and `TIERS_DISK_BYTES` and needs no real wrapper on PATH.
The suite's limit is 60 seconds.

## 6. Package-map verification

Run `tools/tiers/run --verify-all` with every green baseline. It checks all
configured pairs, including live profiles and process-temper, without running
their tests or building bundles or prerequisite binaries. Both verification
forms validate the native binary lists; they do not execute them.
`tools/tiers/run --verify TREE TIER` checks one pair. Neither form accepts
extra nextest arguments.

For each pair, run `cargo nextest list --message-format json --profile TIER-TREE`
behind the wrapper, first over `--workspace`, then over the map's `-p` arguments.
Compare the selected package/binary/test identities, ignoring filtered tests
and build artifact paths. Print both counts for every pair, and continue
checking the remaining pairs after a difference. The listing's output is
captured whole, and what a wrapper prints around the JSON stays visible. A
wrapper that truncates its output reads its own setting from the caller's
environment; the runner sets none. A listing failure names its step and stops.

Verification follows section 3's disk wait before each listing. Workspace
listings build the workspace test
executables; ordinary tier runs build only their mapped packages' tests and
the prerequisites in section 2. When profiles gain or lose packages, regenerate
the sorted map from their workspace listings and verify it before the baseline.

## 7. Opt-in CPU accounting and budgets

`tools/tiers/run --cpu DIR TREE TIER [-- NEXTEST_ARGS...]` collects every
executed test attempt in a fresh, private DIR. Without `--cpu`, the runner
uses exactly section 1's command, prerequisites, environment and exit behavior.
CPU mode builds `tools/tiers/cpu-wrapper.c` with `cc -std=c11 -O2 -static
-Wall -Wextra -Werror`, behind `TIERS_WRAPPER`, beneath Cargo metadata's target
directory, before the nextest command. This Linux process supervisor uses C for
direct `wait4`, `prctl` and signal calls with no interpreter or dynamic-loader
startup. It requires a C compiler and static libc development files. It runs
in section 2.1's prerequisite phase, after the per-pair Cargo binary builds,
with their disk wait, sharing any metadata target already discovered.
It stays outside `[binaries]`: that schema names Cargo package/bin targets
and applies on ordinary runs, whereas this C target is conditional on `--cpu`.
Its CPU
overhead must have a median below 2 ms measured on 100 trivial attempts,
with every attempt below 10 ms as a gross-regression backstop. The measurement
subtracts recorded test-tree CPU from `wait4` usage of a `posix_spawn`ed wrapper,
including supervisor startup, publication and exit, and reports mean, median and max.
The writer caches test identity and a JSON buffer, emits fixed decimal seconds,
and reuses the reserved temporary file for initial publication. Every checkpoint
and final update still uses one write and atomic rename. Its escaped-identity
check covers large names, control characters and UTF-8.

### 7.1 Overlay and gate reuse

The collector uses nextest's experimental `wrapper-scripts`, supported by
nextest 0.9.137. `cpu.py:prepare` returns `--config-file DIR/repository.toml
--tool-config-file tiers-cpu:DIR/nextest.toml`. Nextest permits experimental
feature enabling only in repository configuration, so the first file is a
private copy of the parsed repository configuration with `wrapper-scripts`
enabled and the selected profile's JUnit path pointing into DIR. Other parsed
values round-trip unchanged. The tool overlay defines
`[scripts.wrapper."@tool:tiers-cpu:account"]`, with an absolute wrapper command
and record directory, `target-runner = "within-wrapper"`, and an `all()`
run-wrapper rule for the selected profile. Listing is not wrapped. Profile
slow-timeout settings stay unchanged. Repository wrapper rules have nextest's
normal precedence; missing coverage is an error, rather than an unmeasured
success. JUnit is the independent executed-attempt inventory, including retries.

A caller can use the same collector for `default` and `fuzzy`, keeping its
own package selection and build command, through section 10's interface: it
builds the wrapper once, loads `cpu.py` with `runpy.run_path`, calls
`prepare(DIR, PROFILE, WRAPPER)` for each fresh run directory, appends the
returned arguments to nextest, checks disk admission, and calls
`execute(COMMAND, ENV, DIR, PROFILE, LIMITS)`. `LIMITS` is `budgets(PATH)`'s
result; it may be empty. `execute` owns output forwarding, completion metadata,
wall supervision, evaluation and the resulting exit status. It runs the
command as given: `TIERS_WRAPPER` is the runner's, and a caller's own build
command stands in its place. Keep each DIR until its report is retained. Where and when a caller runs the collector is the
caller's.

The interfaces and precedence are documented in nextest's
[wrapper scripts](https://nexte.st/docs/configuration/wrapper-scripts/),
[tool configuration](https://nexte.st/docs/configuration/#tool-specific-configuration),
and [JUnit support](https://nexte.st/docs/machine-readable/junit/).
The version-pinned identifier and JUnit implementations under
`nextest-runner/src/config/core/identifier.rs` and
`nextest-runner/src/reporter/aggregator/junit.rs` at tag
`cargo-nextest-0.9.137` define tool script identities and retry ordering.

### 7.2 Attempt records and settlement

The wrapper makes itself a `PR_SET_CHILD_SUBREAPER`, runs the unmodified test
command as its child, and sums user and system CPU from `wait4` for the test
and every adopted descendant. Linux's child usage includes descendants the
test itself waited for; those are counted once. After the test exits, adopted
children have 100 ms to finish. The supervisor then repeatedly kills its own
direct adopted children, reaps them and their newly adopted children, and
counts reaped descendants killed with SIGKILL as `leaked`. Settlement ends at
ECHILD or after one second. Cleanup eligibility and the deadline are checked
on every iteration, including successful reaps, and during the owned-child
kill walk. A failure to prove settlement sets `complete=false`.
The supervisor opens `/proc/self/task/SELF/children` before launching the
payload and retains its close-on-exec descriptor. Cleanup rewinds and reads
that descriptor, parsing PID tokens across read boundaries without another
open or allocation. Only that children list is inspected; no foreign
process environment, credentials or cgroup state is read.

`complete_scope="waited-and-reaped-descendants"` defines completeness: the test,
its waited descendants included in its usage, and descendants reaped by the
supervisor. Descendants automatically reaped by the kernel because the test
uses `SIGCHLD=SIG_IGN` or `SA_NOCLDWAIT` can consume CPU that `wait4` does not
report. ECHILD does not prove their CPU was included. This supervisor cannot
reconstruct the test's signal disposition after reaping it, does not monitor
live signal dispositions, and does not claim to detect this exclusion. Thus
`complete=true` proves settlement and accounting only within the stated scope;
CPU for automatically reaped descendants may be absent. No cgroup accounting
is implied.

Each attempt has one file in `DIR/records/`, containing one JSON line:
`profile`, `binary_id`, `test_name`, one-based `attempt`, `user_s`, `system_s`,
`wall_s`, `start_s` (host monotonic seconds), `exit_code` or `signal`,
`complete`, `complete_scope`, `leaked`, `pending`, `settling`, `interrupted`
and `wrapper_cpu_s`. The initial record precedes spawning the test and has
`pending=true`, `settling=false`, `complete=false` and no claimed exit status.
Immediately after reaping the test, atomic rename checkpoints its exit status
and observed CPU with `pending=false`, `settling=true`, `complete=false`, before
any descendant settlement. A final rename updates accounting after settlement
with `settling=false`. Temporary files are ignored. SIGKILL can leave an
incomplete initial or checkpoint record; an observed exit status remains
available even when settlement is interrupted. Incomplete CPU is a lower bound.
`interrupted` is retained for reading older records and is zero in new records.
Normal wrapper records preserve the test's exit code or reproduce
its terminating signal after recording. The test's stdin, stdout and stderr
are inherited unchanged. Collector failures return 125 and are visible.

Normal nextest 0.9.148 execution creates a process group, chooses its group
identifier for termination and signals that group on timeout and cancellation.
The wrapper and test stay in that group. The supervisor sets SIGTERM, SIGINT,
SIGHUP and SIGQUIT to SIG_IGN and keeps waiting; `posix_spawn` restores their
default dispositions in the test. Signals reach the test directly once; the
supervisor never forwards them and has no forwarding counter or wakeup race.
CPU mode rejects debugger interception, which disables nextest's process group.
Source: nextest 0.9.148
[`executor.rs`, 819–880](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.148/nextest-runner/src/runner/executor.rs#L819-L880),
[`ChildPid::for_kill`](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.148/nextest-runner/src/runner/imp.rs#L243-L255),
and [`unix.rs`, 77–232](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.148/nextest-runner/src/runner/unix.rs#L77-L232).

### 7.3 Summaries, budgets and starvation

`tools/tiers/cpu-summary DIR [--budgets tools/tiers/budgets.toml --pair PAIR]
[--json]` reports per-test CPU totals across attempts and per-pair totals,
user/system CPU, wall time, attempts, starved, unobserved, leaked and incomplete counts.
Suite CPU excludes compilation, listing, prerequisites and the supervisor.
Suite wall spans the first test start through the final wrapper settlement;
individual wall times and their sum are also available. A cancelled suite's
wall spans its first test through cancellation completion.

The summary verifies exactly one record for every JUnit attempt, rejects
missing, duplicate, unexpected or malformed records, and rejects incomplete
accounting without starvation evidence. Skipped tests and setup scripts are
not executed test attempts. An interrupted hard-wall run may have no JUnit:
its pre-exec records enumerate started attempts and explicitly show incomplete
accounting. Failures are reconciled from JUnit and every record with a known
test exit status, including checkpoint and accepted extra records. A failed
attempt stays unresolved until a later completed attempt of that test exits
successfully; a pending retry cannot erase it. JUnit policy failures also
remain failures unless a later completed attempt beyond its inventory passes.
Known timeout terminations remain starvation outcomes. JUnit represents
`on-timeout="pass"` as success; a matching successful attempt whose process
ended by SIGTERM/SIGKILL is recognized as a timeout termination. Unstarted
tests are never claimed as executed.

A run is starved when `run.json`'s `starved` is true, the hard wall below, or
when nextest's own `global-timeout` cancelled it: `command-output.txt` holds
nextest's cancellation line, `Cancelling due to global timeout`, first on its
line after any padding and colour. The summary reads that from the output and
leaves `run.json` as `execute` wrote it.

In a starved run, a still-pending attempt is a known timeout termination, and
so is an attempt that has a record, unfinished or ended by SIGTERM or SIGKILL,
and whose JUnit outcome is an abort by one of those signals, the signals of
cancellation (nextest's `test abort` type with `process aborted with signal 15`
or `9` in its message or text): cancellation reached the test through nextest.
A test that died of those signals by itself in a starved run reads the same
way. A record with any other exit status keeps its failure. A JUnit attempt
with no record and such an abort is unobserved: cancellation stopped its wrapper
before the pre-exec record. It is
neither a coverage error, a failure nor an executed attempt. The summary lists
it in `unobserved` by `binary_id`, `test_name` and `attempt`, counts it in the
suite's `unobserved` and warns of it, a green warning beside `starved`. Any
other missing record, and any missing record in a run that is not starved,
fails coverage (exit 2).

`run.json` identifies the profile and completion;
`command-output.txt` retains the command's output. A nonempty DIR is
refused to prevent mixing separate runs. CPU mode
reserves profile and configuration selection, and rejects stress mode, `--no-run` and `--debugger`.
Pending tests and checkpointed tests still settling descendants both count as
active for hard-wall supervision.

`tools/tiers/budgets.toml` contains optional per-pair tables with finite positive
seconds: `cpu_suite_s`, `cpu_test_s`, `wall_hard_s`. The tables record the
limits, with comments distinguishing the user's approved limits from
provisional figures whose interpretation stays open; absent tables have
no limits set here. `[tools]`
reserves the suite namespace in section 7.5. `[gate.<tree>.<profile>]`
reserves a tree's share of a gate suite, such as `[gate.hearth.default]`,
with the same three keys and no `auto_reap`. It is no pair: no tier reads
it, and whatever measures that share reads its verdict there, recording
a run that goes over rather than refusing it.
Smith's `default` and `fuzzy` entries use `cpu_suite_s` alone, at 30 and
120 respectively; whether those are its CPU figures remains open (section 8).
`--verify-all` validates every budget key, value and configured
pair before any command. CPU mode validates it too. A test's CPU budget
covers all its attempts; a pair's covers every test attempt. The summary exits
0 within budget or when no CPU budget is set, 1 for exceeded CPU budgets
(naming every test and suite), and 2 for invalid input or unverified coverage.

`wall_hard_s` applies to test execution, starting with the first attempt, and
is checked while a test is active. Crossing it emits a green `starved` warning
and cancels the command, with no help from whatever wraps it. `execute` starts
every command as the leader of its own session and process group. Cancellation
sends SIGTERM to that group, never to the PID alone, since a wrapper may forward
nothing, and goes on forwarding the command's output. After a 15-second grace,
past nextest's default 10 seconds and the wrapper's one-second settlement so
that an orderly cancellation still writes JUnit, it sends SIGKILL to every
process left in the session and stops reading. Nextest gives each test a process group
of its own inside that session (section 7.2): SIGTERM reaches a test through
nextest, and SIGKILL reaches it through the session, nextest gone or not. A
process that leaves the session is out of reach here: the wrapper's settlement
kills what a test leaves behind, and a build command that moves its work out of
the session, `TIERS_WRAPPER` or a caller's own, must stop that work on SIGTERM.

An interruption of the process running `execute` and a fault in the collector
cancel the same way and are raised again, with the output no longer read; a
second interruption during the grace goes straight to SIGKILL. `tools/tiers/run --cpu`
takes SIGTERM and SIGHUP as interruptions, since neither reaches a command in
its own session.

Nextest slow-timeout terminations are also counted as starved using the JUnit
attempt outcomes, without changing configured timeout periods. Starvation
alone, unobserved attempts included, returns 0; observed assertion, exit or
protocol failures and exceeded CPU budgets keep their failure verdicts.
Incomplete CPU on cancelled attempts is reported explicitly alongside the
green warning.

Every summary includes the top-level `unobserved` list and the suite's
`unobserved` count, including a tools-suite summary. With no unobserved
attempt they are an empty list and zero. These additional fields leave
existing summary keys' names and meanings unchanged (2026-11-04).

#### Budgets and completeness

Budgets apply to recorded CPU: user plus system time of the test, its waited
descendants and descendants the supervisor reaps, as defined by
`complete_scope="waited-and-reaped-descendants"` (section 7.2). A descendant
the kernel automatically reaps because the test ignores SIGCHLD or uses
SA_NOCLDWAIT is not counted, and this exclusion is not detectable by the
collector. An optional per-pair `auto_reap = ["test_name", ...]` declares
known auto-reaping tests; it defaults to an empty list and is empty today.
A declaration produces a summary warning, never a refusal, and does not
waive CPU limits or completeness checks.

An attempt cut short by the wall hard limit may have incomplete accounting:
its `complete=false` record with starvation evidence stays accepted as
`starved`, a green warning; failures observed before it stay red. Every other
`complete=false` record is invalid (exit 2). Passing `wall_hard_s` means
starved, a warning. Exceeding `cpu_suite_s` or `cpu_test_s` is red. Repair it
by asking, in order: does the test belong in this tier; can it be made faster;
otherwise, what does the user decide from the measurements? Moving a test
to escape a budget raises it and is forbidden
(`skein/docs/foundation/testing-strategy.md`, 8.1).

### 7.4 Offline proof

Run `python3 -I -B tools/tiers/test_cpu.py` directly in a cold or warm checkout.
Once per process, the suite invokes `cc` with section 7's compiler flags to
build the wrapper and continuous-reap deadline fixture from source into its
own temporary directory, cleaned after the suite. It never reads `target/`;
a missing `cc` fails the suite with a clear error. The suite prints the user,
system and total CPU seconds spent building both fixtures. The second build
adds `tools/tiers/fixtures/cpu-settlement.c` and
`-Wl,--wrap=wait4,--wrap=clock_gettime,--wrap=open,--wrap=read,--wrap=lseek,--wrap=kill,--wrap=write` to the wrapper
build. It reaps one real child then models a stream
of successful reaps and an advancing clock, with owned-child cleanup calls
intercepted; it spawns no process storm and signals no synthetic PID. Its
optional record trace captures each write to verify checkpoint state.
The modeled children list splits PID tokens across reads. A separate kill
refusal mode checks diagnostics when cleanup cannot settle; the test cleans
up its owned surviving process independently.

The suite covers overlay arguments and parsed configuration,
record coverage and validation, retries and skipped tests, CPU budgets and
verdict exit codes, hard-wall cancellation of a command and its descendants,
an interruption, a nextest global timeout from a recorded cancellation line,
standard streams, exit codes,
single and repeated group signals, exit checkpoints, explicit auto-reap
accounting scope, cancellation with extra records or pending retries,
cancellation aborts without records, starved or not,
settlement deadlines under continuous reaping, waited-child CPU,
double-forked daemon cleanup, and 100 trivial
attempts' overhead. `test_run.py` also checks budget validation during
`--verify-all` and preserves the ordinary command-line assertions in section 5.
All fixtures are local and use no service, sign-in or credential, and no real
`TIERS_WRAPPER` on PATH.

### 7.5 One command as a tools suite

From the checkout root,
`tools/tiers/cpu-run --out DIR --suite ID [--wall-hard SECONDS] -- ARGV...`
runs ARGV once under the same static supervisor. Cargo metadata locates
`target_directory/tiers-cpu-wrapper`, the path used by `run --cpu`. An absent
wrapper, or one older than its source, is built with section 7's flags, behind
`TIERS_WRAPPER` and after section 3's disk wait. Build diagnostics
go to stderr; discovery is quiet on success. A failed discovery, build or
launcher invocation returns 2 without running the suite.

The launcher execs the native
`--suite RECORD ID [--wall-hard SECONDS] -- ARGV...` mode. The command inherits its environment and
all three standard streams. The supervisor returns the command's exit code
or reproduces its terminating signal; launch failures return 127 and collector
failures 125. Without `--wall-hard`, signals sent to the process group reach
the command directly; the supervisor ignores PID-only termination signals as
in section 7.2. `TIERS_WRAPPER` takes only that build: ARGV runs as given.
Before exec, the Python launcher restores default dispositions for SIGPIPE,
SIGXFSZ and SIGXFZ where available and ignored by Python, so a native payload
retains its signal behavior, including a writer terminated by a closed pipe.

With a finite positive `--wall-hard`, the payload starts in its own process
group, separate from the recorder. Its deadline starts at the initial record,
after discovery and any build. The native supervisor polls `wait4` without
blocking. The deadline remains armed through descendant settlement, including
after the main command is reaped. At expiry it checkpoints `starved=true`,
preserving an already known exit status and marking the record as settling,
and sends SIGTERM to the
payload's group. If processes remain after a one-second grace, it sends
SIGKILL to that group and repeatedly kills its own adopted children, including
descendants that escaped the group with `setsid`. It reaps and accounts for
the payload and descendants until ECHILD, allowing up to one further second
for settlement. Only proven settlement produces `complete=true`. The command's
exit code or terminating signal still passes through; a complete starved
record makes starvation a green summary warning, with terminated payloads
excluded from `failures`. Independently exceeded CPU budgets still return 1.

Every post-launch collector error uses one failure path before exiting 125:
signal the owned payload group with SIGTERM, allow a grace, escalate to
SIGKILL, and reap adopted descendants. Where nextest or the caller owns the
untimed group, cleanup signals only the recorder's direct children and those
subsequently adopted, avoiding signals to the caller's group. Error cleanup
uses up to 2,000 one-millisecond polling steps, escalating after 1,000, with
bounded reap batches so a stream cannot prevent escalation. It uses neither
the record writer nor the collector's clock, so those failures cannot abort
cleanup recursively. Diagnostics follow cleanup, and the last published record
remains incomplete; an interrupted write never claims completed accounting.
Failed record writes close their descriptor before entering cleanup, preserving
the write error for the diagnostic. The children-list descriptor is already
open, so sustained EMFILE/ENFILE during record creation cannot prevent cleanup
from finding escaped descendants after adoption. If bounded cleanup cannot
prove settlement, stderr includes `cpu-wrapper: surviving pids:` followed by
the remaining listed children; an enumeration error is explicit and includes
the known unreaped payload PID. Exit 125 and incomplete accounting remain.

The caller's own timeout **must exceed wall-hard + one-second grace + five
seconds**, allowing scheduling and settlement time, and must never signal
the recorder first. Discovery, disk admission, build queueing and compilation
also need time outside the payload deadline. Use the native deadline to stop
the payload while leaving its recorder alive to finish. A killed recorder,
even after its starvation checkpoint, leaves an incomplete record that returns
2 from the summary. Missing records remain invalid too.

Each suite writes `DIR/<ID-sanitized>.json` directly, with section 7.2's fields:
`profile="tools"`, `binary_id=ID`, `test_name="suite"`, `attempt=1`, plus the
required boolean `starved` (false unless the native deadline expires). The filename
uses UTF-8 percent encoding for characters outside ASCII letters, digits and
`-_.~`, including `/` and `%`, so different IDs retain distinct names. The
original ID stays in JSON and in budget keys. The initial pending record is
published atomically without replacing an existing record; checkpoints and
the settled record replace it atomically. Existing suite records are refused
before command launch. A single DIR may hold several distinct suites.
IDs must be nonempty and contain no NUL; they need not name an existing file
or be relative paths. Encoding yields a single filename component, so an ID
containing `..` or `/` cannot traverse out of DIR. DIR is resolved to an
absolute path (following existing symlinks), then created with mode 0700 if
absent. Existing directory permissions are preserved. The operating system's
filename/path limits also apply, including room for the `.XXXXXX.tmp` suffix
used during publication; failures are reported before payload launch.

`cpu-summary DIR --pair tools --expect ID [--expect ID ...] [--budgets PATH]
[--json]` requires an explicit inventory of expected suites. Every expected
suite must have exactly one complete, settled record; missing, unexpected,
duplicate, malformed or incomplete records return 2. Temporary files are
ignored. No nextest run metadata or JUnit report is needed. Totals sum suite
CPU and wall times, excluding launcher startup, discovery and wrapper builds.
Suite exit failures remain visible in `failures`; the invocation's exit code
must be checked by the caller. Summary exit codes retain section 7.3's meaning:
0 for valid records without exceeded CPU limits, 1 over budget, 2 invalid.

`[tools]` is an empty table until the user supplies suite limits. A suite's
table is keyed by its original path, for example
`[tools."tools/tiers/test_run.py"]`, with the same optional finite positive
`cpu_suite_s`, `cpu_test_s` and `wall_hard_s` keys. Both CPU limits apply to that
suite's single command record. A recorded wall time over its limit yields a
green starved warning. The gate passes the intended execution deadline via
`--wall-hard`; a budget table alone does not activate native cancellation.
Complete records remain mandatory for starved suites.

For example, a caller can keep a fresh directory for a gate's tools suites and
run each suite sequentially, checking each command's exit status:

```sh
tools/tiers/cpu-run --out "$records" --suite tools/tiers/test_run.py --wall-hard "$wall_hard" -- \
    python3 -I -B tools/tiers/test_run.py
tools/tiers/cpu-summary "$records" --budgets tools/tiers/budgets.toml \
    --pair tools --expect tools/tiers/test_run.py
```

The gate repeats `--expect` for its full suite inventory even when a command
fails to start; absence then fails coverage rather than implying success.
Section 7.4's hermetic suite also checks generic command stdio, environment,
exit and signal status, daemon cleanup, filename encoding, duplicate refusal,
native build admission, expected coverage and tools budgets. Deadline cases
cover SIGTERM, SIGKILL escalation, cleanup of an escaped child, a timely
completion, invalid durations, and a recorder killed after its starvation
checkpoint. The killed-recorder fixture cleans up its owned payload separately;
the incomplete record must still be rejected.
Regression checks also force a post-launch checkpoint write failure while
a TERM-ignoring payload and escaped child are live, run a native writer into
a closed pipe, and inspect starvation checkpoints during modeled settlement
after the main command has exited with a known failure status.
A descriptor-exhaustion regression lowers only the live supervisor's
RLIMIT_NOFILE to zero while a TERM-ignoring payload and escaped child run;
record creation fails, cleanup reaps both, and the summary rejects the retained
incomplete record.

## 8. Later

A caller running the after-landing tier calls this entry point
(section 10). The trees' old `scripts/browser.sh` scripts retire in their
own trees' branches.

`budgets.toml`'s first comment cites the user's decision on its figures
by a record kept outside this repository; stating where the figures come
from in this repository's own words is left for later.

Whether Smith's focused and fuzzy figures, 30 s and 120 s from
`skein/docs/foundation/testing-strategy.md`, section 8, are CPU seconds
per tree remains open. Their `cpu_suite_s` entries settle no such decision,
and no measurement establishes them here.

`budgets()` validates `[gate.<tree>.<profile>]` and nothing here reads it
beyond that (7.3): what measures a tree's share of a gate suite and records
a run over it is left for later.

`complete_scope` (7.2) can leave CPU out, and temper's browser tier shows a
difference of that size with no cause found. On 2026-11-06 the collector read
13.324 and 13.516 CPU s of two tests whose process trees, sampled through `/proc`
with a recording proxy on Chromium's pipe, read about 18.6 and 18.2. Whether the
figures leave out Chromium's descendants, and what would count them, is left for
later.

## 9. Fixture readers

`.config/fixture-readers.toml` declares the readers and generators of tracked
`**/fixtures/**` files (`skein/docs/foundation/testing-strategy.md`, 8).
Every entry has at least one reader, or `disposition = "provenance"` and a
non-empty `reason`, never both. Provenance is restricted to files whose
basename is `README.md` or capture metadata named `meta.json`. Other fixture
data requires readers; generators use the separate binding below.

A `[[fixture.reader]]` has exactly one form: a root nextest profile `pair`,
`package` and `filter` selecting at least one test, or `suite` naming a tracked
tools suite discovered by the own-tool precheck. The two forms never mix.

```toml
[[fixture]]
paths = ["tree/tests/fixtures/*.json"]
regenerate = ["tree/tests/fixtures/regenerate.sh"] # optional
reader = [{ pair = "default", package = "package", filter = "binary(=suite)" }]

[[fixture]]
paths = ["tools/example/fixtures/input"]
reader = [{ suite = "tools/example/test_input.py" }]

[[fixture]]
paths = ["tree/tests/fixtures/README.md"]
disposition = "provenance"
reason = "Provenance of the recorded fixture bytes."
```

`regenerate` is an array of tracked scripts that write the entry's files.
A generator is named by at least one entry's `regenerate` and matches no
entry's `paths`. Every tracked fixture file either matches exactly one
entry's paths or is named by at least one regenerate array, never both.
A tracked script under a fixture root without a generator binding is
unresolved. A changed generator requires the readers of every entry naming
it. Proof is offline: validation never runs a generator or a capture/provider
API. A generator with stale literal paths and an unambiguous intended output
in the same directory, with the same file names, binds to that output; a
comment records the intended binding.

The entry digest is SHA-256 of its canonical TOML, covering paths, readers,
disposition, reason and regenerate. Keys are sorted, including keys within
readers; arrays and readers retain their order in the file. The map digest
is SHA-256 of the map file's bytes.

## 10. A caller's interface

A caller is whatever runs a tier or a gate suite under the collector and
reads its verdict: a person, a script or a gate. This section is its
contract with `tools/tiers/`; sections 4 to 7.5 hold the behaviour behind
it. Its names, arguments, results and files are kept as they stand: a
change to the build command a caller wraps changes none of them.

### 10.1 The commands

- `tools/tiers/run [--cpu DIR] TREE TIER [-- NEXTEST_ARGS...]`,
  `tools/tiers/run --verify TREE TIER` and `tools/tiers/run --verify-all`,
  from the checkout root, with section 4's exit codes (sections 1, 6
  and 7).
- `tools/tiers/cpu-run --out DIR --suite ID [--wall-hard SECONDS] -- ARGV...`
  (7.5).
- `tools/tiers/cpu-summary DIR [--budgets PATH] [--pair PAIR] [--expect ID ...] [--json]`:
  prints `summary`'s result (10.2) as text, or with `--json` as one JSON
  object, and exits 0 for a valid run within its CPU budgets or with none
  set, 1 over a CPU budget, and 2 for invalid input or unverified
  coverage (7.3).

### 10.2 `cpu.py`

A caller loads `tools/tiers/cpu.py` with `runpy.run_path` and calls these
functions from the namespace it returns; the file is no importable module.
Relative paths are the caller's working directory's, the checkout root.

- `budgets(path, pairs=None)`: the budgets file at `path`, in 7.3's form,
  as a dictionary: each pair's table by its name, under `tools` each
  suite's table by its ID (7.5), and under `gate` each tree's table of its
  profiles' tables (7.3). With `pairs`, a set of pair names, a table for
  any other pair is refused; `tools` and `gate` are no pairs. A refused
  key, value or pair raises
  `ValueError`. The result is the `limits` of `execute` and `summary`; an
  empty dictionary sets no limit.
- `prepare(directory, profile, wrapper, config=Path('.config/nextest.toml'))`:
  makes `directory` (mode 0700, with its parents) the fresh collector of one
  nextest run of `profile`, wrapped by the binary at `wrapper` (section 7),
  writes its unfinished `run.json` and returns the arguments the caller
  appends to that nextest command,
  `['--config-file', 'DIR/repository.toml', '--tool-config-file', 'tiers-cpu:DIR/nextest.toml']`
  with DIR absolute (7.1). A directory that holds anything raises
  `ValueError`: one directory per run.
- `execute(command, env, directory, profile, limits)`: runs `command`, an
  argument list (the caller's build command around nextest, with
  `prepare`'s arguments), with `env`, in a session and process group of its
  own, its stdin closed and its stdout and stderr forwarded to the caller's
  stdout. It holds the profile's `wall_hard_s` from `limits` (7.3), writes
  `run.json` and `command-output.txt` (10.3), prints the summary as text and
  returns the
  run's exit status: 1 over a CPU budget; otherwise 100 when a failure
  stands; otherwise 0 when the run was starved, or when nextest's failure
  exit came from timeout terminations alone, counted starved; otherwise the
  command's own exit status, nextest's 4 for an empty selection among
  them. An invalid summary (7.3's exit 2) raises `ValueError`; an
  interruption stops the command (7.3) and is raised again.
- `summary(directory, limits=None, pair=None, expected=None)`: the verdict
  on a finished run directory, as the dictionary below. A run nextest's
  `global-timeout` cancelled is starved as one past the hard wall is (7.3):
  `summary` reads that from `command-output.txt`, so a caller never writes
  `run.json` itself. `pair` defaults to
  the run's profile and must equal it. `pair='tools'` reads `cpu-run`'s
  suite records instead, and `expected`, the list of suite IDs, is then
  required (7.5); with any other pair it is refused. Invalid input or
  unverified coverage raises `ValueError`; an unreadable or malformed file
  raises `OSError`, `KeyError`, `TypeError` or `xml.etree.ElementTree`'s
  `ParseError`, all of which `cpu-summary` reports as exit 2.
- `print_summary(result, as_json=False)`: prints a result as `cpu-summary`
  does.

`summary`'s result:

| Key | Value |
| --- | --- |
| `pair` | the pair, or `tools` |
| `verdict` | `over budget`, `starved`, `within budget` (the pair has a table) or `no budget set` |
| `failures` | `"BINARY_ID TEST_NAME"` for each test whose failure stands (7.3); for tools, `"ID suite"` |
| `over_budget` | one line for each CPU budget passed, naming its test or suite |
| `warnings` | one line for each `auto_reap` declaration and each unobserved attempt |
| `unobserved` | `{binary_id, test_name, attempt}` for each attempt cancellation stopped before its record (7.3); always empty for tools |
| `tests` | for each test: `binary_id`, `test_name`, `attempts`, `cpu_s`, `user_s`, `system_s`, `wall_s`, `starved`, `leaked`, `incomplete` |
| `suite` | the totals: `tests`, `attempts`, `cpu_s`, `user_s`, `system_s`, `wall_s`, `test_wall_s`, `starved`, `starved_run`, `unobserved`, `leaked`, `incomplete` |

An unobserved attempt is neither passed nor failed: a caller that reads the
run's JUnit itself sees it as a `test abort` failure, and maps it to its own
unobserved result by `unobserved`. A cancelled attempt that has a record
is starved, not failed (7.3): that caller sees it as a `test abort` failure
too, and `tests`' `starved` counts it. These keys and their meanings are the
contract; a key may be added, none removed or changed in meaning.

### 10.3 What a caller may read after a run

In a directory `prepare` made, once `execute` has returned:

- `run.json`: `profile`, `finished`, `starved`, `wall_s`, `returncode`
  (the command's) and `cancel_code` (null: cancellation is signals, with no
  exit status of its own);
- `command-output.txt`: the command's output, as forwarded;
- `junit.xml`: nextest's JUnit report for the profile, absent when nextest
  wrote none;
- `records/`: one file for each attempt, holding 7.2's JSON line; a name
  ending `.tmp` is no record;
- `starved.json`, only when the wall hard limit passed: `wall_hard_s`, the
  limit.

`repository.toml` and `nextest.toml` there are the collector's
configuration, nextest's inputs, not results. A directory `cpu-run` wrote
holds one suite record for each ID, `<ID-sanitized>.json` (7.5); a name
ending `.tmp` is no record.
