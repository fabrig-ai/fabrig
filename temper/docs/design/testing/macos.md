# Testing on macOS

Provisional, 2026-11-18, from `../draft/macos.md`. macOS is a person's
own machine and Linux the platform of record. What macOS tests is one
workflow (section 1), which runs one story (section 2); the gate never
reads it, and whatever is Linux's may be degraded there, its tests
listed by name (section 3). Apple Silicon's file for the site is built
by a workflow of its own, which tests nothing (`../release.md`, 5.1).

## 1. The workflow

- **The workflow,** `.github/workflows/macos.yml`, on GitHub's `macos-26`,
  arm64: on the newest `main` four times a day by a schedule, and by
  hand, on any branch that holds it; never on a push. A red run blocks
  nothing: it is a finding.
- **What it runs:** every target of skein, smith, jig and temper
  compiled, hearth left out
  (`cargo check --workspace --exclude 'hearth*' --all-targets --keep-going`),
  then the one story (section 2). Each check runs whatever an earlier one
  gave, so that one run shows every failure.
- **What it installs:** the toolchain the repository's
  `rust-toolchain.toml` pins (`skein/docs/foundation/programming-model.md`,
  10.1), by rustup reading the file in the checkout, with no version
  written in the workflow; and `cargo-nextest`, its download checked by
  its SHA-256.
- **The runner is small:** line tables only for debug information, no
  incremental build, `TEMPER_INTERNAL_BUNDLE=none`, the registry cached
  by the lock and the dependencies' build by the lock and the
  toolchain's file, and each run reports its disk.
- **A failure reads without a login:** the repository is public, and
  each failing check annotates its run with its failing tests or errors
  and the end of its output, which the public API gives.

## 2. The one story

- **The one story:**
  `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart`
  (`process-tier.md`), never a copy of it: the workflow runs it alone under the profile
  `process-temper`, and a run that selects nothing is red.
- **Its binaries come through `temper-box forge fetch`** (`../protocol/setup/owned.md`,
  6), into a cache of the workflow's own that starts empty, so each
  run shows that the pinned addresses still answer; `TEMPER_TEST_FORGEJO` and
  `TEMPER_TEST_FORGEJO_RUNNER` then name what it fetched, and the box's own
  fetch finds them in place and holds them to the pins' binary digests.
  The workflow prints each one's version first, so that a binary that
  does not run shows apart from the story.
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
  list; none is dropped silently.

  | Test | Its reason |
  |---|---|
  | `crates/temper-box-shell/src/tests.rs`: `id_login_agrees_with_the_effective_users_passwd_entry` | Linux supplies the effective user's passwd entry to compare with macOS's command lookup |
  | `bin/temper-box/tests/process.rs`: `a_story_box_and_its_steps_die_when_the_test_process_is_killed` | a parent-death signal is Linux's |
  | `bin/temper-box/tests/suite/box_lifetime.rs`: `a_killed_box_takes_its_steps_with_it` | a parent-death signal is Linux's |
  | `bin/temper-box/tests/suite/startup.rs`: `parent_guard_observes_its_recorded_process_after_reaping` and `observing_parent_descendants_leaves_cleanup_with_their_world`; `bin/temper-box/tests/suite/box_listeners.rs`: `stop`'s check that no process outlived the box | `parent::Guard`'s retained identities and its observation are Linux's |
  | `bin/temper-box/tests/suite/startup.rs`: `forge_parent_listener_helper`, `forge_run_ignores_its_childs_term_relays_the_listener_and_dies_with_its_status` and `killing_forge_run_alone_kills_its_forgejo_child` | `forge run` stays Forgejo's parent on Linux alone (`../protocol/setup/owned.md`, 4) |
  | `bin/temper-box/tests/process.rs`: `a_forgejo_elsewhere_connected_as_its_admin_makes_the_first_repository_lands_a_chats_change_and_is_found_after_a_restart`, and `testing/temper-box-kit/src/forgejo_elsewhere.rs` | the capture witness uses Linux's abstract Unix sockets |

## 4. Later

- **macOS past the one story** (`../draft/macos.md`, section 6, steps 6 to
  8): each tree's suites on the runner, under a `macos` nextest profile
  measured there and with each tree's list of Linux's own tests; the
  checklist for a person on a real Mac; and the first run's demo in a
  browser. A test that would take real work joins section 3's list.
- **The macOS workflow's results in after-landing.** A red run is a
  finding (section 1), and nothing reads it yet. The repository is public, so
  a run's results need no token.
