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
- **One process per binary:** the focused and fuzzy suites' second mode
  runs a binary's tests in one process and measures CPU per process
  (section 11); nextest's path stays as it is.
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

Each browser pair builds its tree's bundle from the bundle list (2.2), the
one list of the browser bundles, read by any runner that builds them.

| Pair | Prerequisites in order | Bundle environment |
| --- | --- | --- |
| browser-skein | wasm `skein-ui-example-shell-web`; `skein-ui-bundle`; run the tool; `skein-ui-example-server` | `SKEIN_UI_EXAMPLE_BUNDLE`, target's `ui-example-bundle` |
| browser-jig | wasm `jig-client-test-shell-web`; `skein-ui-bundle`; run the tool; `jig-test` | `JIG_TEST_BUNDLE`, target's `jig-test-bundle` |
| browser-temper | wasm `temper-client-shell-web` with its listed features; `skein-ui-bundle`; run the tool; `temper/scripts/bundle.sh --playground`; `temper` | `TEMPER_TEST_BUNDLE`, target's `temper-bundle`; `TEMPER_TEST_PLAYGROUND`, target's `temper-playground` |

The wasm target is `wasm32-unknown-unknown`. Each tool invocation uses the
module's underscored package name, `/bundle/` prefix, and the options of
its tree's table in the list. Temper matches
`temper/scripts/bundle.sh`, without embedding the bundle in the
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
Variable names are unique within a pair. For example, an entry with
`env = "DEMO_TEST_ADAPTER"` sets that variable to the built adapter's path.

After any bundle step, build each listed binary in order with
`cargo build -p PACKAGE --bin BINARY`, then run nextest. The browser
servers in the table above are native list entries. `process-temper` also
lists `smith-fake-claude`'s `fake-claude` and `temper`'s `temper`:
temper's fake-backend tests execute that adapter and the deployment's client
story executes that client as the `temper` binary's siblings. It also lists
`temper-engine`, `temper-worker` and `temper-agent`, which the cluster's story
on the deployment's binaries runs as the `temper` binary's siblings.
A failed binary build returns 2 and starts no later step.

All other configured pairs run nextest alone. Tier tests also use binaries
from their own packages (`harness_fixture`, `jig-test`,
`smith`, `temper`); Cargo/nextest builds those integration-test targets.
The browser servers cross package boundaries and require the explicit builds
above. Smith's selected live tests use the `smith` package's own binaries;
the separate `smith-fake-claude` package's binary tests are not selected by
any of these pairs. No other native binary requirement was found across
the configured pairs.

`[test-binaries]` lists, by package, the binaries of other packages that the
package's focused and fuzzy tests run, in the same records without `env`,
validated alike; the delivery check builds them before its tests
(`tools/pre-pr/design.md`, 4). `smith`'s lists `smith-fake-claude`'s `fake-claude`.

### 2.2 The bundle list

`tools/tiers/bundles.toml` is the one list of the browser bundles: this
runner reads it, and so does any other runner that builds them, so that
each builds the same bundle for a pair. It holds one table for each tree
with a browser pair, `skein`, `jig` and `temper`, of these five fields and
no other:

- `package`, a string: the wasm package;
- `out`, a string: the output directory's name, beneath the target
  directory;
- `variable`, a string: the environment variable that gives the tests the
  output directory's absolute path;
- `options`, an array of strings: the bundle tool's options and their
  values, in order, asset paths relative to the checkout root;
- `features`, an array of strings: the package's Cargo features, empty when
  it has none.

For the pair's tree, with TARGET the target directory and MODULE the
package's name with each `-` made `_`, a runner runs in order:

1. `cargo build --target wasm32-unknown-unknown --features F1,F2 -p PACKAGE`,
   the features joined by commas, with no `--features` when there are none;
2. `cargo build -p skein-ui-bundle`;
3. `TARGET/debug/skein-ui-bundle --module TARGET/wasm32-unknown-unknown/debug/MODULE.wasm --name MODULE --prefix /bundle/ OPTIONS... --out TARGET/OUT`,

and gives every later step VARIABLE, set to `TARGET/OUT` (section 3).

This runner reads the list before the first of those builds. A missing or
unreadable file, a tree with no table, a field missing or of the wrong
type, an empty `package` or `variable`, an `out` that is no plain directory
name (empty, `.`, `..` or holding a `/`), and a field the list does not
define each fail the pair's setup: the message names the file and the field
(`temper.options`), the runner returns 2 (section 4), and no build starts.

## 3. Environment and output

Every child inherits the caller's environment with `NEXTEST_DOUBLE_SPAWN=0`
(skein/docs/foundation/testing-strategy.md, 8). The chosen bundle variable
is set to the absolute output directory for nextest, replacing any inherited
value. `TEMPER_INTERNAL_BUNDLE=none` stays as the caller's environment sets it: temper's
native binary must not embed a person's browser bundle. Chromium, fonts and
live sign-ins remain the caller's environment prerequisites; this tool
installs nothing and performs no sign-in.

Three settings come from that environment. Unset, each does nothing, and no
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
- **`TIERS_HOLD`, the hold:** a command prefix, split on spaces as
  `TIERS_WRAPPER` is, that keeps a run's build directory from being reclaimed
  while the run uses it, for an environment whose cargo lends a build
  directory only while a cargo command runs. A run that builds and then runs
  what it built after cargo has exited needs the directory between the two,
  so the hold spans the whole run, not a command. What the hold does to keep
  the directory is the environment's; these tools name none. Only an
  outermost entry point that runs built executables itself takes it:
  `tools/tiers/one-process` by hand (11.9) and `.temper/pre-pr`
  (`tools/pre-pr/design.md`, 4); `run` and `cpu-run` take none (section 8).
  Such an entry point:
  1. starts the hold once, before its first cargo command, `cargo metadata`
     included, as the prefix alone, from the checkout root, in the entry
     point's process group and session (never one of the hold's own, so that
     a stop of the entry point's tree stops it too), with its stdin a pipe the entry point
     keeps open, its stdout a pipe, its stderr the entry point's;
  2. waits for the hold's first line on stdout, whatever it says: the hold
     is taken. Until then no cargo runs. A hold that cannot start, exits or
     closes its stdout before that line, or gives none within 60 seconds is
     a warning, `<entry point>: hold: <why>; going on unheld`, the entry
     point `one-process` or `pre-pr`, and never a failure: the hold's
     process, if alive, is killed, and the run goes on with none;
  3. runs with `TIERS_HOLD` removed from every child's environment, so that
     an entry point under it (one-process under pre-pr) takes no hold of its
     own: one hold for the run;
  4. at its end, whatever it ends in (a verdict, a failure, an
     interruption), closes the hold's stdin, waits for the hold to exit,
     up to 10 seconds, then kills it with `SIGKILL`.

  The hold keeps the directory until its stdin ends, which the kernel also
  gives it when the entry point dies, by `SIGKILL` too: so it is released at
  the run's end or by its process's death. Unset or empty, nothing is
  started and the run is as before.

  So a hold program, the prefix started alone with no arguments added and
  without `TIERS_HOLD` in its environment: keeps the directory first, then
  writes one line, newline-ended and flushed, to its stdout within 60
  seconds; writes nothing more there, since no one reads it after that
  line; keeps the directory until it reads end of file on its stdin; then
  gives it back and exits within 10 seconds. Its keeping must also end if
  it is killed by `SIGKILL` at that point, and a signal sent to the entry
  point's process group reaches it too. It holds no build of its own: no
  cargo command waits on it.

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
each command, map refusal and sorted package arguments, the bundle list's
commands for each tree, byte for byte, and its refusals, verification with fake
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
Smith's and Skein's `default` and `fuzzy` entries use `cpu_suite_s` alone,
Smith's at 30 and 120 CPU seconds and Skein's at 40 and 120. Jig's and
Temper's `fuzzy` entries use `cpu_suite_s` alone, Jig's at 120 CPU seconds
and Temper's at 180; neither has a `default` entry. These figures are the
strategy's section 8 figures (`skein/docs/foundation/testing-strategy.md`),
by the user's word of 11-07, but for Skein's `default`, 40 by the user's
word of 11-08, and Temper's `fuzzy`, the strategy's 120 raised by half by
the user's word of 11-08, for now, to be revisited; each is recorded beside
its entry. Temper's entry follows Jig's, as does its comment, and sets no
`wall_hard_s`.
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

Switching a caller's focused and fuzzy steps to section 11's mode is a
separate change of that caller, tree by tree. The delivery check runs its
focused tests so (`tools/pre-pr/design.md`, 4), and item 4 of the gate in
`.config/gate/README.md` names the mode's commands for both suites; CI's
own runner, outside the repository, follows them by its owner's change. The gate and after-landing
runs then measure the suites with process CPU; the mode's introduction
requires no CPU comparison or change to budgets, timeouts or build settings.
Per-test CPU within a shared process remains outside this mode's contract.

Two differences from the inproc-runner request remain to reconcile (11.4,
11.6, 11.8): whether estimates use previous process times with test counts
as fallback, rather than sums of kept per-test wall times; and whether
every failure gets an alone diagnostic, including a first-test failure.
Neither choice has a recorded reason for departing from that request.

A caller running the after-landing tier calls this entry point
(section 10). The trees' old `scripts/browser.sh` scripts retire in their
own trees' branches.

The trees' own bundle scripts (`temper/scripts/bundle.sh` and the like)
hold their options in their own words, and read no list: whether they read
2.2's is left for later.

`budgets.toml`'s first comment cites the user's decision on its figures
by a record kept outside this repository; stating where the figures come
from in this repository's own words is left for later.

Jig's and Temper's focused gate entries await the user's word.

Temper's fuzzy 180 (7.3), and the 217.5, 127.5 and 540 CPU seconds of
`browser-skein`, `browser-temper` and `process-temper`, are 120, 145, 85
and 360 raised by half by the user's word of 11-08, for now, as
`budgets.toml`'s comments say: revisiting them is the user's.

`budgets()` validates `[gate.<tree>.<profile>]` and nothing here reads it
beyond that (7.3): what measures a tree's share of a gate suite and records
a run over it is left for later.

`run` and `cpu-run` take no hold (section 3): nextest runs their tests under
the cargo command that built them, but a prerequisite's built tool that the
runner runs itself (section 1) and `cpu-run`'s supervisor run after cargo has
exited, unheld. Whether they take the hold too is left for later, until an
environment that lends build directories runs a slow tier through them.

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
  coverage (7.3). A one-process run's directory reads the same way (11).
- `tools/tiers/one-process [--out DIR] [--budgets PATH] PROFILE [NEXTEST_ARGS...]`
  (11.9), with `execute_one_process`'s exit statuses.

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
- `execute_one_process(command, env, directory, profile, limits, config=Path('.config/nextest.toml'))`:
  runs the selection of `command`, the caller's nextest command as it would
  give `execute` without `prepare`'s arguments, a binary's tests in one
  process (section 11), into `directory`, fresh as `prepare` makes one, which
  it makes itself. It holds `wall_hard_s` from `limits` as `execute` does,
  writes 11.8's files, prints the summary as text and returns as `execute`
  does: 1 over a CPU budget; otherwise 100 when a failure stands; otherwise 0
  when the run was starved or its only blemishes are starved tests; otherwise
  the empty selection's status (4 for nextest's default) or 0. A refusal
  (11.1), an invalid run or summary raises `ValueError`; an interruption stops
  the lanes (11.7) and is raised again.
- `summary(directory, limits=None, pair=None, expected=None)`: the verdict
  on a finished run directory, as the dictionary below. A run nextest's
  `global-timeout` cancelled is starved as one past the hard wall is (7.3):
  `summary` reads that from `command-output.txt`, so a caller never writes
  `run.json` itself. `pair` defaults to
  the run's profile and must equal it. `pair='tools'` reads `cpu-run`'s
  suite records instead, and `expected`, the list of suite IDs, is then
  required (7.5); with any other pair it is refused. A directory whose
  `run.json` has `mode` `"one-process"` is read from its `results.json`
  (11.8), and a `limits` table for its pair with `cpu_test_s` is refused. Invalid input or
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
| `suite` | the totals: `tests`, `attempts`, `cpu_s`, `user_s`, `system_s`, `wall_s`, `test_wall_s`, `starved`, `starved_run`, `unobserved`, `leaked`, `incomplete`, `cpu_by` |
| `processes` | for each process of a one-process run: `binary_id`, `tests` (its tests' names, in the order they ran), `cpu_s`, `user_s`, `system_s`, `wall_s`, `exit_code` or `signal`, `kind`, `leaked`; empty for a nextest run and for tools |

`suite`'s `cpu_by` is `"test"` for a nextest run and for tools, where each
attempt or suite has its CPU, and `"process"` for a one-process run, where
only processes have it (section 11). In a one-process run:

- `tests`' `cpu_s`, `user_s`, `system_s`, `leaked` and `incomplete` are null,
  "not measured", and `attempts` is 1; `suite`'s `cpu_s`, `user_s` and
  `system_s` are the processes' sums, its `leaked` and `incomplete` null, and
  `attempts` the tests that ran. A test's `starved` is 1 for a starved or
  cancelled test (11.6). Each test also has `alone`: `"passed"`, `"failed"`
  or null, its diagnostic's result where it failed (11.6), and `before`, the
  tests that ran before it in its process where it failed, else empty;
- `failures` names each failed and crashed test; `unobserved` each test the
  run never reached, `attempt` 1; `warnings` holds each `may depend on an
  earlier test` label, each leaked process and each unobserved test;
- a process's `kind` is `"tests"`, `"fresh"` (after a process that ended
  before its names did, 11.6) or `"alone"` (a diagnostic), and its `cpu_s` is `user_s` plus `system_s`. An `alone`
  process's CPU is in the suite's: every process with a record is counted.
  A forced kill can leave a started process without a record; its CPU is
  unknown and excluded, so the sum is a lower bound (11.7).

An unobserved attempt is neither passed nor failed. In a nextest run, a
caller that reads JUnit itself sees it as a `test abort` failure, and maps
it to its own unobserved result by `unobserved`. In a one-process run it
has no JUnit case (11.8). A cancelled attempt that has a record
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
configuration, nextest's inputs, not results. A directory
`execute_one_process` made holds 11.8's files instead: `run.json` with its
`mode` and `lanes`, `command-output.txt`, `junit.xml`, `results.json`,
`plan.json` (the lanes' input, no result) and `starved.json` as above; no
`records/`. A directory `cpu-run` wrote
holds one suite record for each ID, `<ID-sanitized>.json` (7.5); a name
ending `.tmp` is no record.

### 10.4 The bundle list

A caller that builds a browser pair's bundle itself reads it from
`tools/tiers/bundles.toml` and builds it as 2.2 gives: the fields and the
commands there are kept as they stand.

## 11. One process per binary

The focused and fuzzy suites' second mode of the collector, beside section 7's
nextest one: each test binary's selected tests run in one process, one at a
time, and binaries run in parallel (`skein/docs/foundation/testing-strategy.md`,
8, *A binary is a process*). A process costs some milliseconds before a test
checks anything, so a run that starts one per binary rather than one per test
spends its budget on the tests. Nextest still selects the tests and builds
them; the mode runs what it selected. Nextest's path (sections 7 to 7.5) is
unchanged, every slow tier stays on it, and `cpu-wrapper.c` stays.

CPU is measured per process only, by the user's word: each process's user
and system time, its own and its waited-for descendants', from `wait4`'s
rusage, and a suite's CPU is their sum. No test's CPU is measured: the
kernel's per-process times tick at 10 ms, and libtest reports a result after
it has moved on, so an outer runner can't divide a process among its tests.
A test's record carries its outcome and wall time, and its CPU is null, "not
measured", never a share or a guess. No `/proc` sampling.

### 11.1 Entry points and refusals

`cpu.py`'s `execute_one_process` (10.2) and `tools/tiers/one-process` (11.9)
run the mode. Its body is `tools/tiers/one_process.py`, loaded with
`runpy.run_path` as `cpu.py` is and no importable module:
`run(command, env, directory, profile, limits, config=Path('.config/nextest.toml'))`
runs one suite into a fresh, private `directory` (mode 0700, refused when it
holds anything) and writes 11.8's files; `execute_one_process` adds the
summary and the exit status.

`command` is the caller's nextest command, as it would pass it to `execute`
without `prepare`'s arguments: its first element whose basename is `cargo`
splits it into the caller's **prefix**, whatever precedes it, and `nextest
run ARGS`. Every command the mode runs that builds or runs tests runs under
that prefix: the listing (11.2), and the lanes (11.4), once for the whole
run, never once per process. `cargo metadata` builds nothing and runs
directly, as in section 1.

The mode reads the profile from `config` as nextest does, the profile's
table over `[profile.default]`'s, and its overrides, then `default`'s. It
refuses, raising `ValueError` before anything runs, what it cannot hold:

- a setup script for the profile; `test-threads` set in the profile, or `-j`,
  `--test-threads` or `NEXTEST_TEST_THREADS` given;
- `retries` other than unset or 0, in the profile, an override, `--retries` or
  `NEXTEST_RETRIES`; a `slow-timeout` with `terminate-after`; an override with
  `threads-required`, `run-extra-args` or `platform`; a profile with `inherits`;
- `--no-run`, stress (`--stress-count`, `--stress-duration`), `--debugger`,
  `--config-file`, `--tool-config-file`; a `--profile` or `-P` other than
  `profile`;
- an override filter outside 11.3's filtersets, and the profile's
  `default-filter` outside them where the caller gives no filter of its own
  (11.2);
- a `limits` table for the profile with `cpu_test_s`: no test's CPU is measured.

### 11.2 Selecting

The listing is the prefix, then `cargo nextest list --message-format json
--profile PROFILE --config-file CONFIG`, CONFIG `run`'s `config` as an absolute
path, and the run command's arguments that `list` takes: package
and target selection, features and build options, filters (`-E`, test-name
filters, `--ignore-default-filter`), `--run-ignored` and `--partition`. Nextest
therefore selects with the configuration the mode reads, a caller's private one
(a gate's, with its own wall cap) as well as the checkout's. Run-only
arguments (`--no-fail-fast`, `--fail-fast`, `--max-fail`, `--no-capture`, the
output and status levels, `--hide-progress-bar`) are dropped, and `--no-tests`
is the mode's own. The listing builds, as `run` would; its output is forwarded,
and its JSON read as section 6's listing is, a prefix's lines around it
allowed. The selection is every `rust-suites` entry with `status` `listed`,
and in it each test whose `filter-match` has `status` `matches`: the
profile's filter, ignored handling and partition as nextest applied them.
Each entry gives the binary's `binary-path`, `cwd`, `binary-id`,
`package-id`, `kind` and `binary-name`.

An empty selection ends the run as nextest would: `--no-tests=fail`, the
default, exit 4; `warn`, nextest's warning, `warning: no tests to run`, in the
output, and 0; `pass`, 0.

A caller's own filter is a filterset in any of nextest's spellings (`-E EXPR`,
`-EEXPR`, `-E=EXPR`, `--filterset EXPR`, `--filterset=EXPR`, and
`--filter-expr` as `--filterset`), a test-name filter, `--ignore-default-filter`
or `--partition`. Where the caller gives none, the mode also evaluates the
profile's `default-filter` over the listing with 11.3's filtersets, and a test
where its reading and nextest's `filter-match` disagree (a test nextest
excluded by its default filter, or matched) refuses the run: the proof that
its reading of the overrides is nextest's. Where the caller gives one, the
mode never compiles the default filter, so one outside 11.3's subset refuses
nothing.

### 11.3 Overrides and test groups

The mode evaluates the overrides' filters itself, over the listing's fields,
in a subset of nextest's filtersets: `all()`, `none()`, `package(M)`,
`binary(M)`, `binary_id(M)`, `kind(M)`, `test(M)`; `not` and `!`, `and` and
`&`, `or`, `|` and `+`, `-` (and not), parentheses; matchers `=` (equal), `~`
(contains), `/…/` (a regular expression, searched) and `#` (glob), with
nextest's defaults: contains for `test()`, equal for `kind()`, glob for the
rest. A filter outside the subset is refused, naming it.

For each test, the first override that matches and sets a key gives it, the
profile's overrides before `default`'s, as nextest's precedence has it. Two
keys are honoured: `priority` (0 when none sets it) and `test-group`. Every
other key is either refused (11.1) or has no effect here (`slow-timeout`'s
period, `leak-timeout`, output and JUnit settings).

**`process-of-its-own`.** A test in the test group of that name runs in a
process of its own, still under the group's `max-threads`: the place for a
test that cannot yet leave its process as it found it (testing-strategy.md,
8). Nextest gives the group no other meaning; it is declared in
`[test-groups]` with `max-threads = "num-cpus"` only when a test needs it,
and each such override says why beside it.

### 11.4 Processes and lanes

A process runs one binary's tests:
`BINARY --exact --nocapture --test-threads=1 --include-ignored --color never --logfile LOG NAME...`,
in the suite's `cwd` with 11.5's environment, its stdin `/dev/null`, its
stdout and stderr one pipe, and libtest's results in a second pipe: `LOG` is a
named pipe (mode 0600) in a private temporary directory the lanes make for the
process, outside the run directory, and remove when they settle it. Libtest
opens its logfile close-on-exec, so the pipe's writer is libtest's alone: an
inherited writer, a `/dev/fd/N`, would reach every child a test spawns, against
the descriptor isolation its fixtures check, and keep the pipe open past the
process. The lanes hold a writer of their own until libtest's first
announcement, so that the reader sees no end of file before libtest has opened
the pipe. No file content is written per test or during the run. Libtest runs
the names in its own order. `--include-ignored` runs a
selected ignored test; the names already hold the selection. Each process
leads a process group of its own.

The lanes are `PREFIX python3 -I -B tools/tiers/one_process.py lanes
DIR/plan.json`, started once in a session and process group of its own as
`execute` starts a command (7.3), with stdin closed. `plan.json`, written
before the lanes start, holds every binary to run: its path, `cwd`,
environment, selected tests in libtest's order (by name) with their
priorities, groups and kept times, the test groups' `max-threads` and the
profile's `global-timeout`. The lanes start, read, reap and stop the
processes, and report each event as one line of their stdout beginning
`tiers-one-process: ` and a JSON object; any other line is the prefix's own
and is forwarded. The lanes write no file but those named pipes, which hold
nothing.

- **As many lanes as CPUs:** `len(os.sched_getaffinity(0))` where the lanes
  run, which is nextest's default number of test threads.
- **Longest first.** A binary's estimate is the sum of its tests' kept wall
  times (11.8); a test with none counts the mean kept time per test of the
  selection, and with none kept anywhere every test counts 1, so binaries
  start by test count. Processes start by priority, the highest of their
  tests', then by estimate, longest first. A fresh process (11.6) takes its
  place among those waiting by the same order, its tests' priority and
  estimate; an alone diagnostic goes before them all (11.6).
- **A long binary is split.** Where a binary's estimate is more than the
  total over the lanes (the selection's estimate divided by the lanes), its
  tests in libtest's order are dealt into
  `min(selected test count, lanes, ceil(estimate / share))` contiguous runs
  of about equal estimate, a process each, and each run keeps at least one
  test: a process given no names would run every test of its binary. A
  process's names stay within 256 KiB of arguments; more splits it further.
- **Test groups.** A process counts once against each test group any of its
  tests is in, and starts only while each such group's processes stay within
  its `max-threads` (an integer, or `num-cpus`: the lanes). A lane free and the
  next process held by its group starts the next that can start.

When a process exits, the lanes reap it with `os.wait4`, read its output
until end of file or 100 ms (nextest's default leak timeout) and then
`SIGKILL` its process group; a process whose output stayed open past that is
`leaked`, a warning.

### 11.5 A process's environment

What nextest gives a test today, resolved per binary from the listing and
`cargo metadata --format-version 1 --no-deps`, over the caller's `env`:

- the caller's `env` with every variable whose name begins `NEXTEST` removed,
  then cargo's `[env]` configuration as nextest applies it (a set variable
  kept unless `force`, `relative` paths from their configuration's directory);
- `CARGO`, the cargo the command names, by path; `CARGO_MANIFEST_DIR`, the
  package's directory, and `CARGO_MANIFEST_PATH`; `CARGO_PKG_NAME`,
  `CARGO_PKG_VERSION` with `_MAJOR`, `_MINOR`, `_PATCH` and `_PRE`,
  `CARGO_PKG_AUTHORS` (joined with `:`), `CARGO_PKG_DESCRIPTION`,
  `CARGO_PKG_HOMEPAGE`, `CARGO_PKG_REPOSITORY`, `CARGO_PKG_LICENSE`,
  `CARGO_PKG_LICENSE_FILE`, `CARGO_PKG_RUST_VERSION` and `CARGO_PKG_README`,
  empty where the package has none;
- `CARGO_BIN_EXE_<name>` and `NEXTEST_BIN_EXE_<name>` for each of the package's
  binaries in `rust-build-meta`'s `non-test-binaries` (`skein-scratch` reads
  them); `OUT_DIR`, the package's build script output directory, where it has
  one;
- the dynamic library path (`LD_LIBRARY_PATH` on Linux,
  `DYLD_FALLBACK_LIBRARY_PATH` on macOS): the target directory's `deps` and
  base output directories, the listing's `linked-paths` and the host's
  `libdir`, then the inherited value;
- `NEXTEST_PROFILE`, the profile, which tests read for a profile's patience;
  never `NEXTEST` itself, which says that nextest runs the test;
- `TIERS_ONE_PROCESS=1`, the mode's own: under it a starved pass passes as
  under `NEXTEST`, ending its process with exit 0 after its line, so that
  nothing follows it and nothing unwinds
  (`skein/docs/design/testing/README.md`, 5), and the lanes start a fresh
  process for the names it had left (11.6).

### 11.6 A test's outcome

Libtest announces each test on the output pipe (`test NAME ... `) before it
runs, with one thread, and writes its result to the log pipe as it ends
(`ok NAME`, `failed NAME`). A test's output is what the process wrote between
its announcement and its result word; its wall time runs from its
announcement's arrival to its result's. A test with no result has the output
since its announcement, and its wall time ends where its process was reaped.
Its outcome is one of:

- **passed:** `ok`;
- **failed:** `failed`;
- **starved:** the process ended with exit status 0 and no result for the
  test it had announced, and the last line of that test's output begins with
  `skein-world: starved:`: a starved pass ended its process after its line
  (`skein/docs/design/testing/README.md`, 5). A warning, never failed and
  never passed. A test with a result keeps libtest's word, whatever its output
  ends with, so that a failure stays red: a panic after the line is the
  failure libtest reports;
- **crashed:** the process ended with no result for the test it had
  announced, and not as a starved pass ends: any other exit status or a
  signal, or 0 without the line last. The test fails, with the process's
  exit status or signal and its output since the announcement;
- **cancelled:** running when its process was marked cancelled (11.7),
  whatever libtest reports after: a result read after the mark counts for
  nothing, and a test announced after it is unobserved. A known timeout
  termination, starved under testing-strategy.md, 8.1, as nextest's cancelled
  tests are (7.3).

A test the run never reached is **unobserved**, never passed.

**A process that ends before its names do,** its running test starved or
crashed, leaves them to a **fresh** process: one starts on the tests of that
process not yet announced, outside the cancellation (11.7). A process that
ended before announcing any test fails all its tests as crashed, and none
starts again: the binary does not run. A process whose running test starved
ended as a starved pass ends, with 0, whatever its earlier tests reported. A
process that ends otherwise than libtest ends (0 with every test passed, 101
with one failed) after reporting every test fails the last test it ran, with
its status. A name libtest never announced in a process that ended as libtest
ends fails as not run by its binary.

**A failure stays red.** Each failed or crashed test that had others run
before it in its process runs once more **alone**, in a fresh process, as a
diagnostic, started before the processes still waiting. Passing alone, it is
labelled `may depend on an earlier test` in the summary and the output, with
the tests that ran before it in its process; failing alone, it is a failure
with no label. Either way it stays failed. A rerun the run's cancellation
stopped labels nothing. A starved test is not rerun.

### 11.7 Cancellation

The profile's `global-timeout` holds as under nextest, from the first
process's start: at expiry the lanes start nothing more and drop the processes
still waiting, whose tests are unobserved. They mark each process not yet
reaped cancelled, send `SIGTERM` to its group and, after nextest's 10-second
grace, `SIGKILL`. Each such process's running test is cancelled and its later
tests unobserved (11.6); a process reaped before the mark keeps what libtest
and its exit gave. The lanes end once they have settled the last process, and
the run is starved. No per-test bound is added: one would be a policy, the
user's.

`limits`' `wall_hard_s` for the profile holds as `execute` holds it (7.3),
from the first process's start while a test runs: crossing it prints the
green `starved` warning, writes `starved.json` and sends `SIGTERM` to the lanes'
group, which stop their processes as at the global timeout; after a 15-second
grace, `SIGKILL` goes to every process left in the session. An interruption of
the process running `run`, or a fault in it, stops the lanes the same way and
is raised again.

**A forced kill.** A process that this `SIGKILL` left without its final event
has no record, and `run` invents no rusage for it; the run still finishes,
starved. Its tests that libtest had reported passed, failed or starved keep
those outcomes, with no process, as 7.3 keeps an attempt that has its record,
so that a failure stays red; its running test, the tests after it and a test
whose outcome rests on its end (crashed, cancelled) are unobserved. Its CPU is
unknown and in no total: the suite's CPU is the sum over the processes with a
record, as 10.2 gives it, and so a lower bound, as a nextest run's is without
its unobserved attempts (7.3). A CPU budget that bound passes is passed; one it
stays under is unproven, and the run is starved. A process left without its
final event otherwise, or lanes that exit with a status other than 0 without
this kill, make an invalid run (`ValueError`).

### 11.8 The run directory

In the directory `run` made, once it has returned:

- `run.json`: section 10.3's keys (`profile`, `finished`, `starved`, `wall_s`
  from the first process's start to the last one's end or the cancellation's,
  the forced kill's where one came (11.7), `returncode`, the lanes' exit status
  as `Popen` gives it, `-9` after a forced kill, and `cancel_code`, null) and
  `mode`, `"one-process"`, and `lanes`, their number;
- `command-output.txt`: the forwarded output: the listing's, then a status line
  for each test as it ends (`PASS`, `FAIL`, `CRASH`, `STARVED`, `CANCEL`, with
  its wall time, binary ID and name) and each failure's output after its line;
  for each alone diagnostic that ended, a line,
  `BINARY_ID NAME: may depend on an earlier test: BEFORE, ...` where it passed
  and `BINARY_ID NAME: failed alone too` where it did not, then the
  diagnostic's output where it did not pass; and the summary;
- `junit.xml` in nextest's shape, so that a caller's JUnit reading holds: a
  `testsuite` for each binary ID, a `testcase` (`name` the test, `classname` the
  binary ID, `time` its wall) for each test that ran; a failed one has a
  `failure` of type `test failure`, a crashed one `test failure` with its exit
  code or `test abort` with `process aborted with signal N (NAME)`, a
  cancelled one `test abort` with the message of the signal that ended its
  process, SIGTERM's where the process handled SIGTERM and exited with a code,
  as nextest's cancellation gives (7.3); a starved one has none, as nextest
  gives a starved pass, and every one but a passed one carries its output in
  `system-out`. An unobserved test has no case;
- `results.json`, written once after the lanes end: each process with a record
  (binary ID, kind, its tests in the order they ran, user and system seconds,
  wall, exit status or signal, leaked) and each test (outcome, wall, its
  process, none where a forced kill left that process without a record (11.7),
  the tests before it and its alone result where it failed, its output where it
  did not pass), the unobserved tests, the selection's count and the
  cancellation; `summary` reads it;
- `plan.json`, the lanes' input, no result; `starved.json` as in 10.3.

**Kept times.** After the lanes end, the mode writes each test's wall time into
`TARGET/tiers-one-process/times.json` (Cargo's target directory, from the
listing), by profile, binary ID and test, keeping the entries of tests this
run did not run: the next run's estimates (11.4). It is the only file the mode
writes outside the run's directory but 11.4's named pipes, which hold nothing,
and nothing reads it as a result.

### 11.9 By hand

`tools/tiers/one-process [--out DIR] [--budgets PATH] PROFILE [NEXTEST_ARGS...]`,
from the checkout root, runs `PROFILE` so: `cargo nextest run --profile PROFILE
--no-fail-fast` and `NEXTEST_ARGS`, with `--workspace` unless they name a
package (`-p`, `--package`), behind `TIERS_WRAPPER` as section 3 gives it, and
prints the summary as `cpu-summary` does. `DIR` defaults to a fresh
`tiers-one-process/runs/<UTC time>-<pid>` beneath Cargo metadata's target
directory, printed first; `PATH`'s limits (`budgets`) apply, none by default.
It exits as `execute_one_process` returns, 2 for an invalid invocation or
run, and takes SIGTERM and SIGHUP as interruptions, as `run --cpu` does.

With `TIERS_HOLD` set, it takes the hold as section 3 gives it: started
before anything else it runs, its target directory's `cargo metadata`
included, and ended after the summary is printed, whatever the run ended in.
Under `.temper/pre-pr`, which takes the hold itself and removes the setting,
it takes none. `run` (11.1) and `execute_one_process` (10.2), called as a
library, take none: the hold is the outermost entry point's.

**A process again.** `tools/tiers/one-process --again RUN_DIR [--out DIR]
BINARY_ID NAME...` runs one process of a finished run's binary again, for a
caller telling a failure under load from a real one
(`tools/pre-pr/design.md`, 4). It reads `RUN_DIR/plan.json` (11.4), takes
the binary whose `binary_id` is `BINARY_ID`, with its path, `cwd` and
environment as the plan holds them, and runs the names, each one of that
binary's selected tests, in one process as 11.4 starts one: one lane, no
split, no alone diagnostic, the plan's `global-timeout`, behind
`TIERS_WRAPPER` as above. It lists and builds nothing: the executable is
the one the run used, so it must still be there, which a caller's hold
keeps. It writes `DIR`'s `plan.json`, `run.json`, `results.json`,
`junit.xml` and `command-output.txt` as a run does (11.8), and no kept
times; `DIR` defaults as above and is printed first. It exits 0 when every
name passed, 100 when one did not, and 2 for an invalid invocation or run:
no `plan.json` in `RUN_DIR`, no such binary, a name not among its selected
tests or given twice, no names, or a `DIR` not fresh. The hold is as
above: taken with `TIERS_HOLD` set, none under `.temper/pre-pr`.

### 11.10 Offline proof

`python3 -I -B tools/tiers/test_one_process.py`, run directly as 7.4's suite
is, builds its fixture test binaries from the Rust sources in
`tools/tiers/fixtures/one-process/` with `rustc --test` into a temporary
directory, once per suite run, and never reads `target/`; a missing `rustc`
fails the suite with a clear error. A fake `cargo` among the fixtures answers
the listing and `cargo metadata` for them, over a fixture checkout's
`.config/nextest.toml`. Its fixtures cover a crash and the fresh process after
it, a hang past the global timeout with processes still waiting, a test that
handles `SIGTERM` and reports a result, one that ignores it until the forced
kill, a starved pass that ends its process and the fresh process after it,
one that ends its process last after an earlier failure, a starved line
followed by a panic or by another exit status, a failure that passes alone
and one that fails alone, a
split binary, test groups and priorities, the environment, the refusals, the
caller's filters and configuration, and the run directory's files. A fake
hold among them records when it starts and when its stdin ends, beside the
fake `cargo`'s calls: one hold, started before the first cargo command and
ended after the last test's process; none with the setting unset or empty;
and a hold that exits before its line leaves the warning and a run that
goes on. `--again`, from a fixture run's directory: its names run in one
process, in order, with the plan's environment and no cargo command; a
pass exits 0 and a failure 100, with the run directory's files and no kept
times; a binary or a name the plan doesn't select, no names and no plan
exit 2.
`test_cpu.py` covers `summary`'s reading of a one-process directory, a forced
kill's included, and the exit statuses.
