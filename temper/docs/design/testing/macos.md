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
  section 3): 150 minutes for `process` and `browser`, 300 for a tree's
  part, 60 for `live`.
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
  | `bin/temper-box/tests/process.rs`: `a_forgejo_elsewhere_connected_as_its_admin_makes_the_first_repository_lands_a_chats_change_and_is_found_after_a_restart`, and `testing/temper-box-kit/src/forgejo_elsewhere.rs`; `bin/temper-box/tests/suite/support_tests.rs`: `the_elsewhere_deployment_keeps_the_whole_fixture_provider_in_its_data_directory` and `elsewhere_settings_follow_the_connected_project`, which reach that module | the capture witness uses Linux's abstract Unix sockets |

## 4. The parts

| Part | Set | What it runs | Holds an id whose package is |
|---|---|---|---|
| `process` | `e2e` | the pair `process-temper`: `tools/tiers/run temper process` | in `tools/tiers/packages.toml`'s `process-temper` |
| `browser` | `e2e` | the pairs `browser-temper`, `browser-jig` and `browser-skein`: `tools/tiers/run TREE browser` for each | in one of those pairs' lists |
| `skein`, `smith`, `jig`, `temper` | `suites` | the tree's focused suite, then its fuzzy one: `tools/tiers/one-process default`, then `fuzzy`, with a `-p` for each of the tree's packages | the tree's |
| `live` | `live` | section 8's stories | in `live-temper`'s or `live-skein`'s list |

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
  (`../draft/macos-tiers.md`, section 8). The binaries
  `tools/tiers/packages.toml`'s `[test-binaries]` lists for the tree's
  packages are built first, as the delivery check builds them
  (`tools/pre-pr/design.md`, 4).
- **The pairs as tier runs have them** (`tools/tiers/design.md`, 1 and 2):
  their prerequisites, bundles and listed binaries, then nextest under the
  pair's profile.

### 4.1 Which part holds an id

- **An id is `classname.name`,** as tier main's report has it
  (`../domain/after-landing.md`, section 4): a test's binary ID, a `.`,
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
  takes one (section 2) and Chrome for `browser` (section 6); then it
  runs, building nothing more.
- **Each check runs through the runner of checks,** which keeps its
  output and annotates its failure (section 1).

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
  timeout, the disk full, a download refused or not its pinned digest,
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

- **As before:** the toolchain the repository's `rust-toolchain.toml` pins
  (`skein/docs/foundation/programming-model.md`, 10.1), by rustup reading
  the file in the checkout, with no version written in the workflow, and
  with the targets the file lists; `cargo-nextest`, its download checked by
  its SHA-256; and the registry, cached by the lock.
- **The runner is small:** 14 GB of disk by GitHub's table. Line tables
  only for debug information, no incremental build,
  `TEMPER_INTERNAL_BUNDLE=none` and `NEXTEST_DOUBLE_SPAWN=0`.
- **Each part's dependencies' build:** `target`, cached under a key of the
  part's, `target-macos-26-PART-rust-`, the toolchain file's hash, `-`,
  the lock's hash, restored by its prefix up to the lock's hash, and saved
  once per key. Before saving, the part removes the outputs of the
  workspace's own packages, which build again in each run, so that the
  cache holds the dependencies alone and the parts' caches stay within
  GitHub's 10 GB a repository; it prints the cache's size.
- **`browser`'s:** the `wasm32-unknown-unknown` target, checked present,
  and Chrome for Testing at the build nearest the Chromium Linux's
  browser tiers run, on that Chromium's branch: Chrome for Testing does
  not publish every patch, so today the pin is 147.0.7727.117 beside
  Linux's 147.0.7727.101. Its version and the SHA-256 of its `mac-arm64`
  archive are pinned in the workflow, and they move with Linux's Chromium
  when its major version moves. The archive comes from Chrome for Testing's public storage, is
  checked against the digest before it is unpacked, and
  `SKEIN_TEST_CHROMIUM` names its executable.
- **`browser`'s C compiler for wasm:** ring's build script compiles C for
  `wasm32-unknown-unknown` while the part builds its bundles, and the
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
  fault (5.4). Only the `browser` job runs it: Linux's tiers name no
  compiler, and the workflow's set-up is not temper's, which takes
  Homebrew as a URL alone (`../draft/macos.md`, 4.3).
- **`process`'s and `live`'s:** Forgejo and its runner, fetched as section
  2 has it.
- **Each part's disk and wall time:** a last step, whatever the part ended
  in, prints the disk (`df`, and `du` of `target` and the registry) and the
  job's wall time since its first step.

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

- **The caches' sizes:** the first run by hand prints each part's; should
  they pass GitHub's 10 GB together, the parts that share most share a
  cache.
- **macOS's budgets,** from the parts' files' durations, the user's figures
  (`../draft/macos-tiers.md`, section 11).
- **A repair that runs its macOS test before it lands,** and the live
  tier's Claude Code story and skein's OpenAI stories on macOS
  (`../draft/macos-tiers.md`, section 11).
- **macOS past the stories** (`../draft/macos.md`, section 6, steps 7 and
  8): the checklist for a person on a real Mac, and the first run's demo
  in a browser.
