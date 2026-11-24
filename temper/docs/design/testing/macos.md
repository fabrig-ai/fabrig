# Testing on macOS

Provisional, 2026-11-19, from `../draft/macos.md` and
`../draft/macos-tiers.md`. macOS is a person's own machine and Linux the
platform of record. What macOS tests is one workflow on GitHub (section
1), dispatched after landing for a set of parts (section 4) and giving one
report (section 5); fabrig's forge hands it its tiers' runs (section 9).
The gate never reads it, and whatever is Linux's may be degraded there,
its tests listed by name (section 3). Apple Silicon's file for the site is
built by a workflow of its own, which tests nothing (`../release.md`, 5.1).

## 1. The workflow

- **The workflow,** `.github/workflows/macos.yml`, on `workflow_dispatch`
  alone: no schedule, never on a push, and no concurrency group, since
  temper runs one run per tier at a time and GitHub queues what passes its
  limit of macOS jobs at once (`../draft/macos-tiers.md`, 4.5). A red run
  blocks nothing.
- **Its inputs** (`../draft/macos-tiers.md`, 4.1):
  - `set`, a choice: `e2e`, `suites`, `live`, or `all`, which is `e2e`
    and `suites` together, for a run by hand;
  - `commit`, the forty lowercase hex digits to test;
  - `tests`, ids one a line (4.1), empty for the whole set; a line that
    is empty or blanks alone is no id, and a trailing carriage return is
    dropped;
  - `request`, the run's title: the workflow's `run-name` is exactly it,
    so that whoever dispatched the run finds it among the workflow's runs.

  `set`, `commit` and `request` are required. An input reaches a step's
  script through that step's environment, never written into the script
  by an expression.
- **Dispatched on `main`,** each job checks out `commit`, keeping no
  credentials. The workflow's own helper, `tools/macos/macos.py`, comes
  from the workflow's commit (`github.sha`), checked out beside it, so
  that a probe of an older commit runs with the helper its workflow
  expects; what is tested, `.config/nextest.toml` and
  `tools/tiers/packages.toml` among it, is `commit`'s.
- **The commit check first:** the first job, `plan`, first refuses a
  `commit` that is not forty lowercase hex digits, or that `origin/main`,
  fetched at the check, doesn't hold (main's tip or one of its ancestors),
  before anything is built or any secret is reached: every other job needs
  `plan`. The `live` job checks again before it reads its secret
  (section 8).
- **Its jobs:** `plan`; then the parts that run (section 4), in parallel,
  one job each; then `report` (section 5). `plan` places the tests and
  gives each part of the set whether it runs and its ids (4.1); a part
  that doesn't run is skipped, and takes no runner. `plan` and `report`
  run on `ubuntu-24.04`, so that they take none of the macOS jobs GitHub
  runs at once (`../draft/macos-tiers.md`, section 10); the parts run on
  `macos-26`, arm64.
- **Each part's timeout** is its set's bound (`../draft/macos-tiers.md`,
  section 3): 150 minutes for `process`, `browser` and `browser-temper`,
  300 for a tree's part, 60 for `live`. Its last 15 minutes are kept for
  the part's saves (4.4).
- **Permissions:** `contents: read`, and `actions: read` for `report`,
  which reads the parts' files.
- **A failure reads without a login:** the repository is public, and each
  failing check annotates its run with its failing tests or errors and the
  end of its output, which the public API gives.

## 2. The one story

- **The one story:**
  `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart`
  (`process-tier.md`), never a copy of it: one of the stories of the pair
  `process-temper`, which the `process` part runs (section 4), and named in
  the report by its id as any other.
- **Its binaries come through `temper-box forge fetch`** (`../protocol/setup/owned.md`,
  6), into a cache of the workflow's own that starts empty, so each
  run shows that the pinned addresses still answer; `TEMPER_TEST_FORGEJO` and
  `TEMPER_TEST_FORGEJO_RUNNER` then name what it fetched, and the box's own
  fetch finds them in place and holds them to the pins' binary digests.
  The workflow prints each one's version first, so that a binary that
  does not run shows apart from the stories. The `process` and `live`
  parts each fetch them so.
- **The `temper` part fetches them too,** the same way and with the same
  set-up, after its build and before its suites, since temper's suites
  hold tests that run the pinned Forgejo, the box's owned forge's
  (`temper-box-kit`'s `owned`) and a real checkout's delivery, and read
  `TEMPER_TEST_FORGEJO` and `TEMPER_TEST_FORGEJO_RUNNER` as Linux's
  builders give them (`../cli/test-env.md`). Its cache is not empty:
  it is kept between runs as the part's other downloads are (section 6),
  since what the suites test is the binaries, and the addresses are
  `process`'s and `live`'s to show.
- **Its set-up before the fetch** is what the forge commands ask of a
  host with no box (`../protocol/setup/box.md`, 4), on the scratch
  directories the workflow makes, each empty and private:
  `temper-box init --box`, then forge 1's first `temper-box forge
  prepare`, with no group's file, so that the forge is the machine's
  owned. Its flags are `--listen 127.0.0.1:3000`, `--person ada
  ada@example.invalid` and the stock runner's label, `--runner-label
  temper-host:host`, so that the setup file it writes gives Forgejo and
  the runner both as fetched (`../protocol/setup/place.md`, 4). The
  fetch reads that file, which off the box nothing but that prepare
  writes.
- **The prepare's refusal is expected:** it goes on to the preparation,
  which refuses while Forgejo is not yet fetched
  (`../protocol/setup/owned.status.md`, *Departs*). So the workflow
  holds the file's presence, not the prepare's status: the prepare's
  output stays in the log, unannotated, and the step goes on to the
  fetch only where `forge-1/setup` is in the data directory, failing
  otherwise with the prepare's output annotated as a failing check's.
  That failure is a fault (5.4).
- **Its support on macOS,** each the simplest that works:
  - `parent::Guard` holds its child and gives it no parent-death signal,
    which is Linux's: the test's own ending stops what it started;
  - the steps of the story's CI use what both systems' tools have, within
    the cleared `PATH`s of the box and of the owned runner;
  - `left`, Linux's whole-tree proof, proves nothing under the tree
    keeper's Group (`skein/docs/design/runtime/shell.md`, section 8): it reads
    empty there, so the story holds the box's clean exit alone.
- **No containers:** its jobs run on the runner's host under the
  `temper-host` label, as on Linux.

## 3. Linux's own tests

- **The Darwin check.** On our Linux machines clippy reaches, for
  `aarch64-apple-darwin`, the members that do not reach rustls, whose
  `ring` crate compiles C against Apple's headers (skein's `runtime/shell.md`,
  section 8); the rest are the workflow's.
- **What both systems have.** A test, a kit or a script they write runs
  each program at a path both systems give it: `/bin/sh`, `/bin/kill`,
  `/bin/sleep`, `/bin/date`, `/bin/cat`, `/bin/ps`, `/usr/bin/true`,
  `/usr/bin/false`, `/usr/bin/env`, `/usr/bin/git`, `/usr/bin/python3`.
  Linux's `/usr/bin/kill` and `/usr/bin/date` and macOS's missing
  `/bin/true` are no such path, and util-linux's `flock`, `setsid` and
  `unshare`, `/dev/full`, and GNU's `stat -c` and `timeout` are Linux's
  alone. So:
  - **its own descriptors** are listed in `/dev/fd`, which on Linux is a
    link to `/proc/self/fd`, so Linux reads the same list;
  - **another process's state** (its name, its run state, its
    descriptors) is read from `/proc` on Linux, unchanged, and on macOS
    from `/bin/ps` and `/usr/sbin/lsof`, the two behind
    `cfg(target_os = "linux")` and its negation;
  - **a lock a script holds across an `exec`** is `flock(2)` taken on a
    descriptor the script opened, by `/usr/bin/python3 -I -B -c 'import
    fcntl; fcntl.flock(9, fcntl.LOCK_EX)'`, the same lock as Rust's
    `File::lock` and util-linux's `flock -x 9`, so the scripts and the
    kit's own Rust exclude each other as before;
  - **a file's type, mode and size** a script checks without reading it
    come from Python's `os.lstat`, not `stat -c`.

  A program that is a test world's hosted name, never run (`/usr/bin/git`
  in a world's table), is no such site.
- **Linux's own tests, by name.** A test that only Linux can run is built
  for Linux alone, `cfg(target_os = "linux")` on its test, module or
  binary, with its reason beside it, and listed here. A test that
  compiles and fails on macOS and would take real work to port joins the
  list; none is dropped silently. A test that reaches a kit's or a
  fixture's Linux-only item is Linux's too: it takes the item's gate and
  is listed beside it, so that each test binary builds on macOS. The
  members that reach rustls, temper-box among them, are past the Darwin
  check, so the runner's build is the first to see a test there that
  reaches a Linux-only item ungated.

  | Test | Its reason |
  |---|---|
  | `crates/temper-box-shell/src/tests.rs`: `id_login_agrees_with_the_effective_users_passwd_entry` | Linux supplies the effective user's passwd entry to compare with macOS's command lookup |
  | `bin/temper-box/tests/process.rs`: `a_story_box_and_its_steps_die_when_the_test_process_is_killed` | a parent-death signal is Linux's |
  | `bin/temper-box/tests/suite/box_lifetime.rs`: `a_killed_box_takes_its_steps_with_it` | a parent-death signal is Linux's |
  | `bin/temper-box/tests/suite/startup.rs`: `parent_guard_observes_its_recorded_process_after_reaping` and `observing_parent_descendants_leaves_cleanup_with_their_world`; `bin/temper-box/tests/suite/box_listeners.rs`: `stop`'s check that no process outlived the box; `bin/temper-box/tests/suite/box_lifetime.rs`: `a_hangup_stops_a_box_and_leaves_nothing_running`'s check that nothing outlived the box; `bin/temper-box/tests/process.rs`: `a_killed_serve_keeps_its_port_and_prints_a_fresh_link` | `parent::Guard`'s retained identities and its observation are Linux's |
  | `bin/temper-box/tests/fake_backends/deployment.rs`: `a_stopped_deployment_restored_at_its_place_uses_new_files`; `bin/temper-box/tests/fake_backends/newer_build.rs`: `a_stopped_fake_box_starts_with_new_files_at_the_same_paths`; `testing/temper-process-guard/tests/parent.rs`, the whole binary | an executable's observation reads `parent::Guard`'s retained Linux process identities |
  | `bin/temper-box/tests/suite/startup.rs`: `forge_parent_listener_helper`, `forge_run_ignores_its_childs_term_relays_the_listener_and_dies_with_its_status` and `killing_forge_run_alone_kills_its_forgejo_child` | `forge run` stays Forgejo's parent on Linux alone (`../protocol/setup/owned.md`, 4) |
  | `bin/temper-box/tests/process.rs`: `a_forgejo_elsewhere_connected_as_its_admin_makes_its_first_repository_and_is_found_after_a_restart`, and `testing/temper-box-kit/src/forgejo_elsewhere.rs`; `bin/temper-box/tests/suite/support_tests.rs`: `the_elsewhere_deployment_keeps_the_whole_fixture_provider_in_its_data_directory` and `elsewhere_settings_follow_the_connected_project`, which reach that module | the capture witness uses Linux's abstract Unix sockets |
  | `bin/temper-box/tests/process.rs`: `a_default_box_with_a_sign_in_registration_starts_without_dns` | it takes the box's network and name service away with Linux's namespaces (`unshare`) |
  | `bin/temper-box/tests/process.rs`: `temper_box_restarts_a_killed_forgejo_while_the_engine_serves`; `bin/temper-box/tests/suite/startover.rs`: `an_owned_setup_killed_at_its_run_is_kept_and_a_removal_cut_finishes_on_restart` | each finds a box's step among its processes by the step's `/proc` executable or command line, and on macOS the box's tree reads empty (section 2) |
  | `crates/temper-box-shell/src/tests.rs`: `the_standalone_operations_check_retains_no_descriptors` | it names each open descriptor's file by its `/proc/self/fd` link, and macOS's `/dev/fd` holds no links |
  | `crates/temper-qa-tool/tests/suite/builds.rs`: `a_manifest_keeps_a_non_utf8_path_as_its_bytes`; `crates/temper-qa-tool/tests/suite/build_files.rs`: `real_checksums_keep_a_non_utf8_path_as_its_bytes`; `crates/temper-qa-tool/tests/suite/run_passes.rs`: `installation_reads_a_non_utf8_manifest_path_as_its_bytes`; `crates/temper-qa-tool/tests/suite/place.rs`: `a_new_recipe_reads_a_non_utf8_commit_path_as_its_bytes` | APFS, macOS's file system, holds no file name that is not UTF-8; their plain and escaped UTF-8 halves run on both systems (`qa/builds.md`, 4) |
  | The cluster's stories, each reading its description through `testing/temper-box-kit`'s `Cluster::read` or `Cluster::read_from`: in `bin/temper-box/tests/process.rs`, `an_engine_alone_prints_its_lines_and_drains_on_a_signal`, `the_cluster_stops_in_reverse_each_exiting_0`, `a_worker_alone_with_the_owned_examples_worker_links_and_drains_on_a_signal`, `a_task_runs_on_each_worker_of_the_deployments_binaries`, `a_worker_killed_mid_run_ends_by_its_signal_and_its_link_closes`, `a_worker_stopped_past_the_silence_links_again_as_the_same_process` and `an_unexpected_member_exit_is_reported_after_its_client_settles`, with the helpers only they use; its `cluster_turn` module, whole; `bin/temper-box/tests/live_github.rs`, the whole binary | the cluster's members run in skein-world's cgroup trees, which the user's service manager delegates on Linux alone (`../protocol/cluster.md`, 5); macOS has no cgroups |

## 4. The parts

| Part | Set | What it runs | Holds an id whose package is |
|---|---|---|---|
| `process` | `e2e` | the pair `process-temper`: `tools/tiers/run temper process` | in `tools/tiers/packages.toml`'s `process-temper` |
| `browser` | `e2e` | the pairs `browser-jig` and `browser-skein`: `tools/tiers/run TREE browser` for each | in one of those pairs' lists |
| `browser-temper` | `e2e` | the pair `browser-temper`, the one that takes the playground: `tools/tiers/run temper browser` | in `tools/tiers/packages.toml`'s `browser-temper` |
| `skein`, `smith`, `jig`, `temper` | `suites` | the tree's focused suite, then its fuzzy one, every layer: `tools/tiers/one-process --layers all default`, then `fuzzy`, with a `-p` for each of the tree's packages | the tree's |
| `live` | `live` | section 8's stories | in `live-temper`'s or `live-skein`'s list |

- **`browser-temper` apart from `browser`** (6.3): the playground is that
  pair's alone and most of its build, so the pair takes a job, a cache
  and a bound of its own. Its compiler's red is `build.browser-temper`;
  `build.browser` is jig's and skein's pairs' build.
- **A tree's packages** are the workspace's packages (`cargo metadata
  --no-deps`, which builds nothing) its name takes, by the rules at the
  head of `.config/nextest.toml`: skein's `skein-*`,
  `forgejo-conformance` and `github-conformance`; smith's `smith` and
  `smith-*`; jig's `jig` and `jig-*`; temper's `temper` and `temper-*`.
  hearth and `fabrig-boundary` are no part's (`../draft/macos-tiers.md`,
  section 3).
- **A tree's suites as the gate runs them** (`.config/gate/README.md`,
  item 4): each test binary's tests in one process (`tools/tiers/design.md`,
  11), with no `--gate`, `--walls` or `--ledger`, and no budget: `--budgets`
  names an empty file, since macOS has no budgets yet
  (`../draft/macos-tiers.md`, section 8).
- **Every layer, the real loop and end to end with them.** The gate runs
  the layers that replay and leaves the real-loop and end-to-end binaries
  to after landing (`tools/tiers/layers.md`, 9); a suites part passes
  `--layers all`, so that those binaries run on macOS's kernel as before:
  kqueue's readiness backend, real children and files are what a macOS
  run checks that Linux's can't (`skein/docs/design/testing/conformance.md`). The binaries
  `tools/tiers/packages.toml`'s `[test-binaries]` lists for the tree's
  packages are built first, as the delivery check builds them
  (`tools/pre-pr/design.md`, 4).
- **The pairs as tier runs have them** (`tools/tiers/design.md`, 1 and 2):
  their prerequisites, bundles and listed binaries, then nextest under the
  pair's profile.

### 4.1 Which part holds an id

- **An id is `classname.name`,** as tier main's report has it
  (`../domain/after-landing-runs.md`, section 3): a test's binary ID, a `.`,
  its name. The binary ID is the id's text before its first `.`; its
  package is the binary ID's text before `::`, or the whole binary ID
  where it has none (a library's).
- **Read from the binary, never from a listing:** `plan` places each id
  by its package alone, from `commit`'s `tools/tiers/packages.toml` and
  the name rules above, building nothing. `build.PART` (5.2) is `PART`'s.
- **Held twice:** under `all`, `temper-box` is `process-temper`'s and
  temper's, and an id of it goes to both parts, each running it where its
  profile selects it.
- **Held by none:** an id no part of the set holds runs nowhere, so it
  fails nowhere, as a test absent at its commit fails nowhere in a
  bisect's probe; `plan` names each such id in a notice.

### 4.2 The tests given

- **A part runs only its ids:** each of its runs takes the filterset
  `binary_id(=B) & test(=T)` for each of its ids, joined by `|`, beside
  the profile's own selection, and `--no-tests=warn`, so that an id the
  profile doesn't select, or absent at the commit, runs nowhere. A pair or
  a live story that holds none of the part's ids doesn't run.
- **A part holding none of them is skipped,** its build too: its job
  doesn't start.
- **Its build is whole:** a part given ids builds what its whole set
  would, so that a build's red stays the part's (5.2); a probe costs most
  of a run (`../draft/macos-tiers.md`, section 8). A part given
  `build.PART` alone builds and runs nothing.
- **With no tests given,** a part runs its whole set, and one that selects
  nothing is a fault (5.4).

### 4.3 Build, then run

- **Each part builds first** everything its runs need: a pair's
  prerequisites and test binaries (`tools/tiers/run TREE TIER -- --no-run`),
  a tree's listed binaries and test binaries (`cargo nextest run
  --no-run` with the tree's `-p`s); then it fetches the forge where it
  takes one (section 2) and Chrome for the browser parts (6.1); then it
  runs, building nothing more.
- **A browser pair's run makes its bundles again** from what the build
  step built, every unit fresh, since the job's environment is the same
  for both steps (6.3): the playground is built once a job. Where a
  tier run's log shows cargo compiling a crate (`Compiling NAME vVERSION`),
  the helper annotates the run with a warning naming each such crate,
  so that a second build shows.
- **The build shows what it built:** after the build, the helper gives
  a notice for each build command: the number of dependencies cargo
  compiled and of the workspace's crates, read from its `Compiling`
  lines, where a workspace crate's line names its path. A warm run's
  count of dependencies is zero (6.2).
- **Each check runs through the runner of checks,** which keeps its
  output and annotates its failure (section 1).

### 4.4 The part's deadline

- **The saves keep the bound's last 15 minutes:** a part's deadline is
  its job's first step (`MACOS_STARTED`) plus its timeout, less 15
  minutes. The helper runs each command it starts as the leader of a
  process group of its own, and at the deadline it stops that group:
  SIGTERM, then SIGKILL to whatever is left after 60 seconds, and it
  waits until the command has ended. The part is then cut short, a
  fault (5.4), its log's tail annotated as a failing check's, and its
  last steps prune and save its dependencies' build (6.2) and print its
  disk within the minutes kept. The prune's one command, the query
  (`cargo metadata`) naming the workspace's packages and targets, builds
  nothing and so runs past the deadline, still through the helper's
  log; where it fails, the prune warns, removes nothing it would need
  those names for, and the save goes on.
- **A cancel stops it the same way:** the steps that install, build or
  run run the helper as the step's own process (`exec`), so the SIGINT
  or SIGTERM a cancel or GitHub's timeout sends the step reaches it;
  it stops its command's group as at the deadline, then exits as a
  fault. Nothing the part started writes into `target` while the save
  reads it.
- **Why:** in run 44561458165 (6.3) GitHub's timeout cancelled the run
  step while the commands it had started went on, and the save that
  followed failed: `/opt/homebrew/bin/gtar failed with exit code 1`,
  GNU tar's status when a file changes as it reads it.
- **The bounds are the helper's table:** the helper reads a part's
  timeout from its own table of parts, which a test holds equal to each
  job's `timeout-minutes`; the timeouts stay the sets' bounds
  (section 1). A command started with no `MACOS_STARTED`, as in the
  helper's own tests, has no deadline.

## 5. The report

### 5.1 Each part's file

- **`PART.xml`:** JUnit in nextest's shape, every case the part ran with
  its time, the test suites of each of its runs in one file: a browser
  pair's each, a tree's two suites', each live story's. A nextest run's
  comes from its profile's JUnit (section 7); a tree's suite's from its
  run directory's `junit.xml` (`tools/tiers/design.md`, 11.8).
- **Uploaded** as the artifact `part-PART`, whether its tests passed or
  failed. The part's job fails when one of its cases did.

### 5.2 A build that fails is a red

- **One failing case:** the part's file then holds one case, `classname`
  `build` and `name` the part, so that its id is `build.PART`
  (`build.temper`), its failure's type `build failure` and its text the
  first errors cargo printed, each whole, within 16 KiB. The part runs
  nothing more.
- **The compiler's refusal alone:** a build fails as a red when its output
  holds cargo's `could not compile` line. A build that fails otherwise, a
  download refused or the disk full, is a fault (5.4).
- **A rerun or a probe of `build.PART`** builds that part (4.2): built, it
  reports no such case.

### 5.3 The one report

- **`report`,** after every part, whatever each ended in, makes the run's
  artifact `junit`: `junit.xml`, holding a `testsuite` for each part that
  ran, named for it, with each case of the part's file that has a
  `failure` or an `error`, as the file has it; and beside it each part's
  file. A run whose parts all passed has a `junit.xml` with no case.
- **Its ids are tier main's,** `classname.name`. Whoever relays the run
  reads `junit.xml` (`../draft/macos-tiers.md`, 5.2); the parts' files are
  for whoever reads durations, macOS's budgets among them (section 10).
- **The parts' files** reach `report` through GitHub's CLI on its job's
  token, `gh run download`, so that the workflow pins no action beyond
  those it already has.

### 5.4 Faults

- **Anything but a test's failure or a build's red is a fault,** never a
  red (`../draft/macos-tiers.md`, 4.4): a runner lost, a job past its
  timeout or cut short at its deadline (4.4), the disk full, a download refused or not its pinned digest,
  the live secret absent (section 8), a run whose status is neither a pass
  nor a test's failure (nextest's 0 and 100, one process per binary's 0
  and 100), a whole set that selects nothing, and a run that left tests
  of its selection unobserved, its profile's `global-timeout` or a wall
  past (`tools/tiers/design.md`, 11.7). The part then uploads no file and
  fails.
- **The run then has no report:** where a part that ran left no file,
  `report` makes no `junit`, names each such part in an annotation and
  fails, so that the run ends with no report, which the relay takes as a
  fault (`../draft/macos-tiers.md`, 5.2). A report of the other parts alone
  would read the missing part's tests as passed.

## 6. What it installs and caches

### 6.1 What each part installs

- **As before:** the toolchain the repository's `rust-toolchain.toml` pins
  (`skein/docs/foundation/programming-model.md`, 10.1), by rustup reading
  the file in the checkout, with no version written in the workflow, and
  with the targets the file lists; `cargo-nextest`, its download checked by
  its SHA-256; and the registry, cached by the lock.
- **The runner is small:** 14 GB of disk by GitHub's table. Line tables
  only for debug information, no incremental build,
  `TEMPER_INTERNAL_BUNDLE=none` and `NEXTEST_DOUBLE_SPAWN=0`.
- **Debug information is not split:** beside the two settings that give
  line tables, `CARGO_PROFILE_DEV_DEBUG` and `CARGO_PROFILE_TEST_DEBUG`,
  the workflow's environment sets `CARGO_PROFILE_DEV_SPLIT_DEBUGINFO` and
  `CARGO_PROFILE_TEST_SPLIT_DEBUGINFO` to `off`, with a comment saying
  why. On macOS, Cargo's dev profile defaults to `unpacked`, which leaves
  the debug information in the object files; with line tables alone, a
  backtrace reads them and finds no function's name, and skein's
  fail-stop tests check that a panic's backtrace names the panicking
  function (`skein/tests/fail-stop/tests/suite/fail_stop.rs`). With
  `off`, the object files go after linking, and a backtrace names each
  frame from the executable's symbol table, without file and line; a
  panic's own line still gives its location. `off` is also the one value
  rustc accepts for `wasm32-unknown-unknown`, whose bundles the browser
  parts build in a nested cargo that inherits the environment. The
  workspace's profiles name no split, so Linux's builds keep their
  platform's default, and the fail-stop tests keep their check on both.
- **Each part's dependencies' build,** cached as 6.2 has it.
- **The browser parts', `browser`'s and `browser-temper`'s alike:** the
  `wasm32-unknown-unknown` target, checked present,
  and Chrome for Testing at the build nearest the Chromium Linux's
  browser tiers run, on that Chromium's branch: Chrome for Testing does
  not publish every patch, so today the pin is 147.0.7727.117 beside
  Linux's 147.0.7727.101. Its version and the SHA-256 of its `mac-arm64`
  archive are pinned in the workflow, and they move with Linux's Chromium
  when its major version moves. The archive comes from Chrome for Testing's public storage, is
  checked against the digest before it is unpacked, and
  `SKEIN_TEST_CHROMIUM` names its executable.
- **The browser parts' C compiler for wasm:** ring's build script compiles C for
  `wasm32-unknown-unknown` while a part builds its bundles, and the
  runner's `clang` is Apple's, which has no such target. Before the
  build, after the target's check, the helper's `wasm-compiler` command
  takes Homebrew's LLVM: the newest `llvm` or `llvm@N` formula the runner
  image has installed, or, where it has none, `llvm` installed by Homebrew
  then, its bottle checked by Homebrew's own digest. It checks that the
  LLVM's `clang` lists the `wasm32` target among its registered targets
  (`clang --print-targets`), prints `clang --version` through the runner
  of checks, and writes to the job's environment
  `CC_wasm32_unknown_unknown`, that `clang`'s path, and
  `AR_wasm32_unknown_unknown`, its `llvm-ar`'s, which the `cc` crate
  reads for that target alone, so the native builds keep the runner's
  compiler. No LLVM, a `clang` without the target or no `llvm-ar` is a
  fault (5.4). Only the browser parts' jobs run it: Linux's tiers name no
  compiler, and the workflow's set-up is not temper's, which takes
  Homebrew as a URL alone (`../draft/macos.md`, 4.3).
- **`process`'s and `live`'s:** Forgejo and its runner, fetched as section
  2 has it.
- **`temper`'s:** Forgejo and its runner, fetched as section 2 has it,
  into a fetch cache that `actions/cache` restores before the fetch and
  saves once per key, as the registry is: its key is
  `forge-macos-26-`, then the hash of the files that pin the binaries,
  `skein/crates/skein-forge/src/forgejo/pin.rs` (versions and digests)
  and `temper/crates/temper-forge-setup/src/fetch.rs` (their addresses),
  so that a moved pin fetches afresh. The box's fetch holds what it finds
  in place to the pins' digests and fetches nothing more
  (`../protocol/setup/owned.md`, 6). The configuration and data
  directories of the fetch's set-up start empty each run; only the cache
  is kept.
- **`smith`'s:** ripgrep 14.1.1, the release Linux's machines carry at
  `/usr/bin/rg`, from its GitHub release, the `aarch64-apple-darwin`
  archive. Its version and the archive's SHA-256 are pinned in the
  workflow, and the archive is checked against the digest before it is
  unpacked, as Chrome for Testing's is. The directory holding its `rg`
  goes on the job's `PATH`, so that smith's suites find it on the test
  process's `PATH`, as smith finds `rg` on the commands' `PATH`
  (`smith/docs/design/protocol/agent.md`, section 2;
  `smith/docs/design/testing.md`, 2.2). The workflow prints
  `rg --version` through the runner of checks first, so that a binary
  that does not run shows apart from the suites. A download refused or
  not its digest is a fault (5.4).
- **Each part's disk and wall time:** a last step, whatever the part ended
  in, prints the disk (`df`, and `du` of `target` and the registry) and the
  job's wall time since its first step.

### 6.2 Each part's dependencies' build

- **Its keys:** `target` is cached under the part's complete key,
  `target-macos-26-PART-rust-`, the toolchain file's hash, `-`, the
  lock's hash; and a run that did not build everything saves under a
  partial key, the complete key, `-partial-`, the run's id, `-` and its
  attempt (`target-macos-26-browser-temper-rust-T-L-partial-44561458165-1`).
- **Restored** by the complete key; else by the newest partial key that
  begins with it; else by the newest key of the part and the toolchain,
  `target-macos-26-PART-rust-` and the toolchain file's hash, whatever
  its lock.
- **Saved whatever the part ended in,** after the prune and once the
  part's own commands have ended (4.4): under the complete key where the
  build step built everything; under a partial key where it did not, a
  compiler's red, a fault, the deadline or a cancel having stopped it;
  and not at all where the restore found the complete key, which already
  holds every dependency the lock names. So each run cut short leaves
  the next warmer, and an entry under a complete key is whole.
- **The prune keeps the dependencies alone:** in each profile's
  directory, any directory holding `.fingerprint` (`debug`, `web`,
  `playground`, `wasm32-unknown-unknown/debug`, `wasm32-unknown-unknown/web`,
  `wasm32-unknown-unknown/playground`), it keeps `deps`, `build` and
  `.fingerprint`, less the workspace's entries; it removes everything
  else under `target`: the profiles' final outputs, `examples`,
  `incremental`, the bundles, `nextest`, `tmp` and the tiers'
  directories, whatever their names, keeping only `target`'s own
  `.rustc_info.json` and `CACHEDIR.TAG`.
- **The workspace's entries by their whole name:** an entry of `deps`,
  `build` or `.fingerprint` is the workspace's when its name, less a
  leading `lib`, is a workspace package's or target's name (with `-` or
  `_`) alone, followed by `-` and cargo's sixteen hex digits, or followed
  by `.` and an extension, the two together too
  (`libtemper_box-0123456789abcdef.rlib`); never because it only begins
  with one, so that a workspace test named `http` would not take
  `http-body-0123456789abcdef`. A workspace name that is also a
  dependency's in `Cargo.lock` is left, its outputs kept, and named in a
  warning annotation.
- **Why the playground's dependencies built again** in run 44561458165,
  which had restored `browser`'s key from an earlier run: that run,
  44517790115, stopped in its build at ring's C for wasm (6.1) before
  the playground's builds, so its entry held none of the optimized
  profiles; the prune's former rule, by a name's beginning, matched no
  dependency of `Cargo.lock` at `ee9999b230`; and 44561458165's own save
  failed (4.4). Under fat LTO a warm cache spares little anyway (6.3).
- **Within GitHub's 10 GB a repository** for every part: a complete key
  is saved once per lock and toolchain; GitHub, past the repository's
  limit, evicts the entries used longest ago first, which are the
  superseded keys; partial keys pile up only while a part's runs are cut
  short, each warmer than the last. Each part prints its `target`'s size
  after the prune, and the save prints its archive's.

### 6.3 How the browser parts fit

- **Where it started:** the one `browser` part ran the three pairs. In
  run 44561458165 (2026-11-19, at `7cacb44526`, `macos-26`, 3 cores),
  the build step began at 16:03:03Z; `temper/scripts/bundle.sh
  --playground` took 8,564 s, its `cargo build --target
  wasm32-unknown-unknown --profile web -p temper-playground-page`
  81m48s and its `--profile playground -p temper-playground` 60m36s.
  The run step began at 18:32:06Z, began the playground's bundle again
  at 18:32:20Z, and the job was cancelled at its 150 minutes; its save
  failed (4.4). Run 44517790115, at `80e3098f39`, failed the part after
  79 minutes, mostly building, on ring's C compiler for wasm, since
  repaired (6.1). The part never reached its tests.
- **Two parts, two jobs:** `browser-temper`, the pair that takes the
  playground, runs alone; `browser` runs jig's and skein's pairs. Each
  has its runner, its cache and its 150 minutes, in parallel; `e2e`'s
  three jobs are within GitHub's macOS jobs at once.
- **The playground lighter on macOS:** `browser-temper`'s job sets, at
  the job's level, `CARGO_PROFILE_WEB_LTO` and
  `CARGO_PROFILE_PLAYGROUND_LTO` to `off`, and
  `CARGO_PROFILE_WEB_CODEGEN_UNITS` and
  `CARGO_PROFILE_PLAYGROUND_CODEGEN_UNITS` to `16`; both profiles are
  named, since a variable of `web`'s is not sure to reach the profile
  that inherits it. Linux's tiers and the release build the profiles as
  `Cargo.toml` has them (`../playground.md`, 7).
  - *Why:* fat LTO in one codegen unit optimizes the whole program anew
    in every build, on one core, from bitcode no cache spares, and took
    most of the 8,564 s. Without it, each crate's code is made once, a
    dependency's kept in the cache (6.2), and cargo compiles three crates
    at a time.
  - *What stays:* the opt-levels, `s` for the page and `z` for the
    world, so that the playground runs on macOS as fast as the browser
    harness's forty seconds (skein's `ui/web/testing.md`, section 2) and
    the stories' waits were set for; `panic = "abort"`, the playground's
    fail-stop (skein's `playground.md`, section 7); `overflow-checks`,
    which every profile keeps (`skein/docs/foundation/programming-model.md`,
    10.4); and `strip`.
  - *Nothing the stories check depends on what changes:* no test reads
    the profiles or holds a size of the modules or the download; the
    download's 8 MB bar is reported, not checked (`../playground.md`,
    8), and macOS has no budgets (`../draft/macos-tiers.md`, section 8).
    The figures the playground's story prints on macOS, its download's
    size among them, are this build's, never set beside Linux's.
- **Built once a job:** the build step's tier run builds the playground;
  the run step's makes its bundle again from units all fresh (4.3),
  since the job's environment holds for both steps: the profiles'
  variables at the job's level, the wasm compiler's written before the
  build, and nothing a later step adds that cargo reads.
- **What a run builds:** a warm one, the workspace's crates alone, four
  times: natively for the tests and the tier's binaries, and for
  wasm32 in `debug` (temper's bundle), `web` (the playground's page) and
  `playground` (its world). A cold one builds every dependency for each
  besides. A cold run fits within the bound if it can; one that doesn't
  is cut at its deadline (4.4) and saves what it built (6.2), so that
  the next run starts warm, and a warm run fits.
- **Each run shows how it fit:** the build's notices (4.3), the tier
  run's `tiers: playground bundle built in` seconds, the prune's and the
  save's sizes (6.2), and the job's disk and wall time (6.1).

## 7. Timeouts on macOS

- **Each slow timeout doubles on macOS** (`../draft/macos-tiers.md`, 4.3):
  in `.config/nextest.toml`, each profile gains, at the head of its own
  overrides, overrides for `platform = 'cfg(target_os = "macos")'` that
  give each of its tests twice the slow timeout it has on Linux under
  that profile, its `terminate-after` unchanged; there is no profile of
  macOS's own. Each override reaching the profile that sets a slow timeout
  has its double, with its filter, and a last one, with no filter, doubles
  the profile's own setting, or `default`'s where it has none.
- **Every profile, not only the parts':** nextest applies `default`'s
  overrides to every profile after the profile's own and ahead of any
  profile's settings, so that a profile with no macOS overrides of its own
  would take `default`'s on macOS in place of its own timeouts.
- **A test's Linux timeout is nextest's reading:** the first override that
  matches the test and sets a slow timeout, the profile's before
  `default`'s, else the profile's setting, else `default`'s
  (`tools/tiers/design.md`, 11.3).
- **The global timeouts stay as they are:** they are the user's, as
  macOS's budgets are (`../draft/macos-tiers.md`, section 8). A run they
  cut short is a fault (5.4), its summary naming the tests it left
  unobserved.
- **One process per binary reads no slow timeout** and passes over these
  overrides (`tools/tiers/design.md`, 11.1 and 11.3).
- **The nextest runs' JUnit:** the profiles `process-temper`,
  `browser-temper`, `browser-jig`, `browser-skein`, `live-temper` and
  `live-skein` each set `junit = { path = "junit.xml" }`, nextest's report
  under `target/nextest/PROFILE/`, which 5.1 reads.

## 8. The live part

- **Only `live` names the environment `macos-live`,** whose deployment
  branches are `main` alone, and its secret `ANTHROPIC_API_KEY`
  (`../draft/macos-tiers.md`, section 6).
- **Its first steps:** the checkout, the commit check again (section 1),
  then the secret's presence: absent, the part fails with no report
  (5.4) before anything is built. No step prints the secret.
- **The key's file:** after the build and the fetch, before the stories,
  the job writes the key to a fresh 0600 file in the runner's temporary
  directory and gives its path as `TEMPER_TEST_LIVE_ANTHROPIC_KEY` and
  `SKEIN_TEST_LIVE_ANTHROPIC_KEY`, as the live tier's stories take it
  (`live-tier.md`); a last step removes the file, whatever the job ended
  in.
- **temper's story,**
  `a_person_asks_claude_on_an_api_key_to_land_a_change_on_the_owned_forge`,
  as `temper/scripts/proof-live.sh forge-land-claude-key` runs it: the
  script's nextest command, under the profile `live-temper`,
  `--ignore-default-filter --run-ignored only` and the story by its
  binary and name, through `tools/tiers/run temper live`, with the
  script's environment: `TMPDIR` a fresh private directory,
  `TEMPER_TEST_LIVE_STORE` another, and `TEMPER_TEST_LIVE_NOTICE` a 0600
  file in the first. Its model is the story's own, `claude-haiku-5-5`.
  The script itself stays a person's sitting: it reads `/proc` and takes
  GNU's `tail` and `stat` options, which macOS doesn't have.
- **skein's Anthropic stories:** the profile `live-skein`'s stories
  named `anthropic_`, ignored but for it, through `tools/tiers/run skein
  live` with `--run-ignored only`, on
  `SKEIN_TEST_LIVE_ANTHROPIC_MODEL=claude-haiku-5-5`, and
  `SKEIN_TEST_LIVE_CA_FILE=/etc/ssl/cert.pem`, macOS's roots.
- **The risk is the user's** (`../draft/macos-tiers.md`, section 6): code
  on main runs where the key can be reached; the key serves these stories
  alone, under its own spend limit.

## 9. The tiers on fabrig's forge

- **`.forgejo/workflows/tier-macos-e2e.yml` and `tier-macos-suites.yml`,**
  beside `tier-main.yml` and shaped as it is: dispatch-only with
  `temper_commit`, `temper_key` and `temper_tests`; one job, on the runner
  label `relay`, never `dogfood` (`../draft/macos-tiers.md`, 5.1); its
  step `"$CI_TIER" --tier macos-e2e`, or `macos-suites`, with the commit,
  `temper_tests` as `CI_TIER_TESTS` and the job's token as
  `CI_TIER_TOKEN`, as tier main's; then `junit.xml` uploaded as the
  artifact `junit`, whatever the step ended in.
- **Their timeouts** sit 30 minutes above their sets' bounds, 180 minutes
  for `macos-e2e` and 330 for `macos-suites`, so that the relay's bound
  ends a run first.
- **`$CI_TIER` for a macOS tier** lives on the runner's host, outside this
  repository (`../draft/macos-tiers.md`, 5.2): it dispatches `macos.yml`
  on `main` with the tier's set, the commit, the tests and a request
  naming the tier and its run, finds the run by its title, waits within
  the set's bound, and writes `junit.xml` from the run's `junit` (5.3).
- **Their spans** are the reference deployment's configuration, set when
  each tier is turned on once its set is green (`../draft/macos-tiers.md`,
  sections 3 and 7): 3 hours for `macos-e2e`; 6 hours for `macos-suites`,
  then 24 after a week green.
- **`live` has no tier:** the relay dispatches it once a day, beside the
  live tier's run (`../draft/macos-tiers.md`, section 6).

## 10. Later

- **`packed` for skein's part:** should skein's fail-stop tests still
  find no function's name in a backtrace on the runner with split debug
  information `off` (section 6), skein's part alone takes `packed`, whose
  `.dSYM` keeps file and line too.
- **The caches' sizes:** the first run by hand prints each part's; should
  they pass GitHub's 10 GB together, the parts that share most share a
  cache. A part's entry keeps the dependencies a former lock named as
  well as the current ones, so it grows with each change of a
  dependency's version until a sweep by the lock removes them.
- **The workspace's unchanged crates fresh across runs:** cargo judges a
  workspace crate by its files' times, which every checkout renews, so
  each run builds all of them. Cargo's checksum freshness on the pinned
  nightly, with the workspace's libraries kept in the cache, would spare
  those a commit didn't change, at the cost of a larger cache.
- **The `live` job's one step** (section 8) runs the helper's commands in
  turn from a shell, so its deadline holds there but a cancel's signal
  reaches the shell alone.
- **macOS's budgets,** from the parts' files' durations, the user's figures
  (`../draft/macos-tiers.md`, section 11).
- **A repair that runs its macOS test before it lands,** and the live
  tier's Claude Code story and skein's OpenAI stories on macOS
  (`../draft/macos-tiers.md`, section 11).
- **macOS past the stories** (`../draft/macos.md`, section 6, steps 7 and
  8): the checklist for a person on a real Mac, and the first run's demo
  in a browser.
