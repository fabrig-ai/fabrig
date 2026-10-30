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
overhead must be at most 2 ms per test; the offline 100-attempt measurement
subtracts recorded test-tree CPU from `wait4` usage of a `posix_spawn`ed wrapper,
including supervisor startup, publication and exit, and reports mean and max.

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
ECHILD or after one second. A failure to prove settlement sets `complete=false`.
Only `/proc/self/task/SELF/children` is inspected for cleanup; no foreign
process environment, credentials or cgroup state is read.

Each attempt has one file in `DIR/records/`, containing one JSON line:
`profile`, `binary_id`, `test_name`, one-based `attempt`, `user_s`, `system_s`,
`wall_s`, `start_s` (host monotonic seconds), `exit_code` or `signal`,
`complete`, `leaked`, `pending`, `interrupted` and `wrapper_cpu_s`.
The initial record precedes spawning the test and has `pending=true`,
`complete=false` and no claimed exit status. Atomic rename replaces it after
settlement; temporary files are ignored. A SIGKILL therefore leaves explicit
incomplete evidence, never an invented complete zero-CPU result. Its CPU is a
lower bound. Normal wrapper records preserve the test's exit code or reproduce
its terminating signal after recording. The test's stdin, stdout and stderr
are inherited unchanged. Collector failures return 125 and are visible.

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
accounting. Unstarted tests are never claimed as executed. `run.json` identifies
the profile and completion; `heavy-output.txt` retains the public command
output. A nonempty DIR is refused to prevent mixing separate runs. CPU mode reserves profile and
configuration selection, and rejects stress mode and `--no-run`.

`tools/tiers/budgets.toml` contains optional per-pair tables with finite positive
seconds: `cpu_suite_s`, `cpu_test_s`, `wall_hard_s`. It is empty until the user
sets numbers. `--verify-all` validates every budget key, value and configured
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

After building the native prerequisite, run `python3 -I -B
tools/tiers/test_cpu.py`; `TIERS_CPU_WRAPPER` may select a different native
artifact. The suite covers overlay arguments and parsed configuration,
record coverage and validation, retries and skipped tests, CPU budgets and
verdict exit codes, hard-wall heavy cancellation, standard streams, exit codes,
signals, waited-child CPU, double-forked daemon cleanup, and 100 trivial
attempts' overhead. `test_run.py` also checks budget validation during
`--verify-all` and preserves the ordinary command-line assertions in section 5.
All fixtures are local and use no service, sign-in or credential.

## 8. Later

The after-landing runner calls this entry point. The trees' old
`scripts/browser.sh` scripts retire in their own trees' branches.
