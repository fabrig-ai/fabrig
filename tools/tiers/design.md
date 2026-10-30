# Slow tier runner

Provisional, 2026-10-29. Run one tree's slow tier with its prerequisites.

## 1. In one page

- **One entry point:** from a fabrig checkout root on a builder,
  `tools/tiers/run TREE TIER [-- NEXTEST_ARGS...]` builds prerequisites and
  runs `heavy cargo nextest run -p PKG ... --profile TIER-TREE --no-fail-fast`,
  using the pair's sorted package names from `tools/tiers/packages.toml`,
  followed by the extra arguments unchanged. The map is generated from the
  packages with selected tests in each profile's workspace JSON listing.
- **The config owns the pairs:** the root `.config/nextest.toml` profile
  tables whose names contain a hyphen supply the pairs, split at the last
  hyphen into tier and tree. Unknown pairs and pairs missing a valid package
  set fail before any command runs.
- **Builds are sequential:** every build and nextest command uses `heavy`;
  the built bundle tool runs directly. Cargo metadata builds nothing and
  runs directly to discover the target directory.
- **Disk before each heavy command:** wait for at least 80,000,000,000 bytes
  available on `/`, checking once per minute and printing each wait.
- **Visible evidence:** print each step's command; heavy prints its log path
  and nextest's summary. Prerequisite failures name their step.

## 2. Prerequisites

Paths to source assets are relative to the checkout root. Build artifacts
and bundle output directories are beneath `cargo metadata`'s
`target_directory`, including when Cargo config relocates it. All builds
use Cargo's default dev profile, as the trees' browser scripts do.

| Pair | Prerequisites in order | Bundle environment |
| --- | --- | --- |
| browser-skein | wasm `skein-ui-example-shell-web`; `skein-ui-bundle`; run the tool; `skein-ui-example-server` | `SKEIN_UI_EXAMPLE_BUNDLE`, target's `ui-example-bundle` |
| browser-jig | wasm `jig-client-test-shell-web`; `skein-ui-bundle`; run the tool; `jig-test` | `JIG_TEST_BUNDLE`, target's `jig-test-bundle` |
| browser-temper, qa-temper | wasm `temper-client-shell-web` with `loopback-navigation`; `skein-ui-bundle`; run the tool; `temper` | `TEMPER_TEST_BUNDLE`, target's `temper-bundle` |

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
`TEMPER_TEST_CLAUDE` this way for its two program-login tests. The temper
adapter and smith's adapter both call `smith_fake_claude::main()`.

After any bundle step, build each listed binary in order through
`heavy cargo build -p PACKAGE --bin BINARY`, then run nextest. The browser
servers in the table above are native list entries. `qa-temper` and
`process-temper` also list `temper-fake-claude`'s `fake-claude`: temper's
fake-backend tests execute that adapter as the `temper` binary's sibling.
A failed binary build returns 2 and starts no later step.

All other configured pairs run nextest alone. Tier tests also use binaries
from their own packages (`skein-lab-browser`, `harness_fixture`, `jig-test`,
`smith`, `temper`); Cargo/nextest builds those integration-test targets.
The browser servers cross package boundaries and require the explicit builds
above. Smith's selected live tests use the `smith` package's own binaries;
the separate `smith-fake-claude` package's binary tests are not selected by
any of these pairs. No other native binary requirement was found across
the fourteen pairs.

## 3. Environment and output

Every child inherits the caller's environment with `NEXTEST_DOUBLE_SPAWN=0`
(skein/docs/foundation/testing-strategy.md, 8). The chosen bundle variable
is set to the absolute output directory for nextest, replacing any inherited
value. `TEMPER_BUNDLE=none` stays as the builder sets it: temper's native
binary must not embed a person's browser bundle. Chromium, fonts and live
sign-ins remain the caller's environment prerequisites; this tool installs
nothing and performs no sign-in.

The runner sets umask 077, closes child stdin, and leaves child output visible.
Heavy supplies its own log paths and command summaries. A direct bundle tool
has no heavy log. Commands remain sequential, and their inputs must stay
still until completion. A low disk check sleeps 60 seconds, checks again,
and prints another line only if disk remains low; there is no disk timeout.

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
environment, map refusal and sorted package arguments, verification with fake
JSON listings, disk admission and refusal/failure exit codes without builds,
network access, Chromium or credentials. The suite's limit is 60 seconds.

## 6. Package-map verification

Run `tools/tiers/run --verify-all` with every green baseline. It checks all
configured pairs, including live profiles and process-temper, without running
their tests or building bundles or prerequisite binaries. Both verification
forms validate the native binary lists; they do not execute them.
`tools/tiers/run --verify TREE TIER` checks one pair. Neither form accepts
extra nextest arguments.

For each pair, run `cargo nextest list --message-format json --profile TIER-TREE`
through heavy, first over `--workspace`, then over the map's `-p` arguments.
Compare the selected package/binary/test identities, ignoring filtered tests
and build artifact paths. Print both counts for every pair, and continue
checking the remaining pairs after a difference. Heavy's complete listing
output is captured with `HEAVY_TAIL` set to retain all lines; its evidence
around the JSON stays visible. A listing failure names its step and stops.

Verification follows the same unchanged 80,000,000,000-byte admission rule
before each heavy command. Workspace listings build the workspace test
executables; ordinary tier runs build only their mapped packages' tests and
the prerequisites in section 2. When profiles gain or lose packages, regenerate
the sorted map from their workspace listings and verify it before the baseline.

## 7. Opt-in CPU accounting and budgets

`tools/tiers/run --cpu DIR TREE TIER [-- NEXTEST_ARGS...]` collects every
executed test attempt in a fresh, private DIR. Without `--cpu`, the runner
uses exactly section 1's command, prerequisites, environment and exit behavior.
CPU mode builds `tools/tiers/cpu-wrapper.c` with `heavy cc -std=c11 -O2 -static
-Wall -Wextra -Werror` beneath Cargo metadata's target directory, before the
nextest command. This Linux process supervisor uses C for direct `wait4`,
`prctl` and signal calls with no interpreter or dynamic-loader startup. It
requires the builder's C compiler and static libc development files. It runs
in section 2.1's prerequisite phase, after the per-pair Cargo binary builds,
sharing their heavy admission and any metadata target already discovered.
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

The queue can use the same collector for `default` and `fuzzy`, retaining its
own package selection and heavy gate command. Ops builds the wrapper once
through heavy, loads `cpu.py` with `runpy.run_path`, calls
`prepare(DIR, PROFILE, WRAPPER)` for each fresh run directory, appends the
returned arguments to nextest, checks disk admission, and calls
`execute(COMMAND, ENV, DIR, PROFILE, LIMITS)`. `LIMITS` is `budgets(PATH)`'s
result; it may be empty. `execute` owns output forwarding, completion metadata,
wall supervision, evaluation and the resulting exit status. Do not wrap an
already running heavy command in another heavy command. Keep each DIR until
its report is retained. Gate integration and deployment belong to ops.

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
Only `/proc/self/task/SELF/children` is inspected for cleanup; no foreign
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
user/system CPU, wall time, attempts, starved, leaked and incomplete counts.
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
tests are never claimed as executed. `run.json` identifies
the profile and completion; `heavy-output.txt` retains the public command
output. A nonempty DIR is refused to prevent mixing separate runs. CPU mode
reserves profile and configuration selection, and rejects stress mode, `--no-run` and `--debugger`.
Pending tests and checkpointed tests still settling descendants both count as
active for hard-wall supervision.

`tools/tiers/budgets.toml` contains optional per-pair tables with finite positive
seconds: `cpu_suite_s`, `cpu_test_s`, `wall_hard_s`. No limits are set until the
user supplies numbers; `[tools]` reserves the suite namespace in section 7.5.
`--verify-all` validates every budget key, value and configured
pair before any heavy command. CPU mode validates it too. A test's CPU budget
covers all its attempts; a pair's covers every test attempt. The summary exits
0 within budget or when no CPU budget is set, 1 for exceeded CPU budgets
(naming every test and suite), and 2 for invalid input or unverified coverage.

`wall_hard_s` applies to test execution, starting with the first attempt, and
is checked while a test is active. Crossing it emits a green `starved` warning
and invokes the owned command's public `heavy --cancel TOKEN`, using the token
printed by heavy. It never edits heavy state or stops its units directly.
Nextest slow-timeout terminations are also counted as starved using the JUnit
attempt outcomes, without changing configured timeout periods. Starvation
alone returns 0; observed assertion, exit or protocol failures and exceeded
CPU budgets keep their failure verdicts. Incomplete CPU on cancelled attempts
is reported explicitly alongside the green warning.

### 7.4 Offline proof

Run `python3 -I -B tools/tiers/test_cpu.py` directly in a cold or warm checkout.
Once per process, the suite invokes `cc` with section 7's compiler flags to
build the wrapper and continuous-reap deadline fixture from source into its
own temporary directory, cleaned after the suite. It never reads `target/`;
a missing `cc` fails the suite with a clear error. The suite prints the user,
system and total CPU seconds spent building both fixtures. The second build
adds `tools/tiers/fixtures/cpu-settlement.c` and
`-Wl,--wrap=wait4,--wrap=clock_gettime,--wrap=fopen,--wrap=kill` to the wrapper
build. It reaps one real child then models a stream
of successful reaps and an advancing clock, with owned-child cleanup calls
intercepted; it spawns no process storm and signals no synthetic PID.

The suite covers overlay arguments and parsed configuration,
record coverage and validation, retries and skipped tests, CPU budgets and
verdict exit codes, hard-wall heavy cancellation, standard streams, exit codes,
single and repeated group signals, exit checkpoints, explicit auto-reap
accounting scope, cancellation with extra records or pending retries,
settlement deadlines under continuous reaping, waited-child CPU,
double-forked daemon cleanup, and 100 trivial
attempts' overhead. `test_run.py` also checks budget validation during
`--verify-all` and preserves the ordinary command-line assertions in section 5.
All fixtures are local and use no service, sign-in or credential.

### 7.5 One command as a tools suite

From the checkout root, `tools/tiers/cpu-run --out DIR --suite ID -- ARGV...`
runs ARGV once under the same static supervisor. Cargo metadata locates
`target_directory/tiers-cpu-wrapper`, the path used by `run --cpu`. An absent
wrapper, or one older than its source, is built with section 7's flags through
heavy after section 3's 80,000,000,000-byte admission check. Build diagnostics
go to stderr; discovery is quiet on success. A failed discovery, build or
launcher invocation returns 2 without running the suite.

The launcher execs the native `--suite RECORD ID -- ARGV...` mode, which only
changes identity and record naming. The command inherits its environment and
all three standard streams. The supervisor returns the command's exit code
or reproduces its terminating signal; launch failures return 127 and collector
failures 125. Signals sent to the process group reach the command directly;
the supervisor ignores PID-only termination signals as in section 7.2. Group
cancellation remains the caller's responsibility. No extra heavy command
wraps an already admitted suite.

Each suite writes `DIR/<ID-sanitized>.json` directly, with section 7.2's fields:
`profile="tools"`, `binary_id=ID`, `test_name="suite"`, `attempt=1`. The filename
uses UTF-8 percent encoding for characters outside ASCII letters, digits and
`-_.~`, including `/` and `%`, so different IDs retain distinct names. The
original ID stays in JSON and in budget keys. The initial pending record is
published atomically without replacing an existing record; checkpoints and
the settled record replace it atomically. Existing suite records are refused
before command launch. A single DIR may hold several distinct suites.

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
green starved warning; `cpu-run` does not impose a wall timeout. The gate owns
command cancellation and must still require complete records.

For example, ops can retain a fresh directory for a gate's tools suites and
run each suite sequentially, checking each command's exit status:

```sh
tools/tiers/cpu-run --out "$records" --suite tools/tiers/test_run.py -- \
    python3 -I -B tools/tiers/test_run.py
tools/tiers/cpu-summary "$records" --budgets tools/tiers/budgets.toml \
    --pair tools --expect tools/tiers/test_run.py
```

The gate repeats `--expect` for its full suite inventory even when a command
fails to start; absence then fails coverage rather than implying success.
Section 7.4's hermetic suite also checks generic command stdio, environment,
exit and signal status, daemon cleanup, filename encoding, duplicate refusal,
native build admission, expected coverage and tools budgets.

## 8. Later

The after-landing runner calls this entry point. The trees' old
`scripts/browser.sh` scripts retire in their own trees' branches.
