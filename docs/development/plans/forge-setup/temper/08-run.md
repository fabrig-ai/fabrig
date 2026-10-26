# Session 08: `temper forge prepare` and `run`, and the process tier's Forgejo

`temper forge prepare CONFIG.json FORGE` hosts temper 07.4's preparation on skein-io's processes, under the forge's
locks (08.1). `temper forge run CONFIG.json FORGE forgejo|runner` takes the process's lock, checks `state` and the
binary under it, and replaces itself by `forgejo web` or `forgejo-runner daemon`, the lock being its standard input
(08.2). The process tier's Forgejo is then started by the two, its users, tokens and repository still the support's
(08.3). The upgrade is temper 15's; reconcile, CI and the box are 10 and 13.

## Read first

- Design: setup.md 3, 6.1, 6.3, 6.4, 8 (what a forge's members equal), 11, 13.1, 13.4; configuration.md 8;
  domain/testing.md 2.6. Plan: decisions.md 6, 7, 17; `findings/skein-io.md` 1, 3-5; `forge-and-tiers.md` 2, 4.
- skein: `programming-model.md` 2.1, 3; `io.md` 6; `shell.md` 12-13; `testing-strategy.md` 2.1, 8; temper `workflow.md` 2.
- temper 07, `temper/07-prepare.md`, by name: `setup::{Setup, Entry, Mode, Owned}`, `place::{Place, File}`,
  `state::State`, `spec::{Command, Spec, Which, command, version}`, `prepare::{Prepare, step, Event, Found, Ran, Exit,
  Request, Failure}` (07.4), the binary's check `setup::check` in `crates/temper/src/setup.rs` (07.3); temper 03.1's
  `temper_forge_forgejo::pin::{VERSION, RUNNER_VERSION, names_forgejo, names_runner}`.
- Code: `crates/temper/src/sign_in.rs:83-146,166-219` (a command on a `Kernel` and a `Host`: the model of both hosts),
  `src/lib.rs:22-75,302-309`, `src/main.rs:2`, `src/tests.rs:4-17,106-123`; skein-io `src/kernel.rs:523-546` (`Spawn`,
  `Pipe`), `src/tests/process.rs:90-136` (a child's pipes as streams; :118-125 a close kills its group),
  `src/layer.rs:198-206` (`Io::adopt_signals`); skein-shell `src/ring.rs:95-99` (`open_termination_signals`);
  `testing/temper-throwaway-forgejo/src/instance.rs:13-20,27-83,209-225`, `tests/throwaway.rs:3-12` (a script as the
  binary); 08.3's Files for the process tier's support.

## What the design asks

- **6.3, 8:** the preparation takes Forgejo's lock, and the runner's when 07.4's machine asks, refusing naming the forge
  if one is held; its commands run on skein-io's processes, output bounded and masked, exit awaited; a termination
  signal kills the running command's group, exit 1 `forge <n>: interrupted`, `state` not marked ended. Every `temper
  forge` command checks the document and reads no file but its own part's.
- **6.4, 6.1:** `run` checks `state` and the binary's first `--version` line against the pins, takes the lock without
  waiting, hands it on as standard input, and `execve`s with the spec's arguments and environment alone (answer 24); the
  lock and the exec are std's, in one module. **13.4, domain/testing.md 2.6:** the tier's Forgejo is owned (no runner, no CI).

## Choices this plan makes

- **One `Kernel` per command** (answer 37; 6.3). `command::run_one` opens the termination signals (`skein_shell::
  open_termination_signals`: blocked from the first call on, so one arriving between two commands waits for the next),
  a `Kernel` and an `Io` that adopts them, spawns the spec (`Request::Spawn`: its environment the whole one, `root`
  from `skein_shell::open_root`, pipes for fds 1 and 2), reads both pipes to their end keeping `spec.output` bytes of
  each (07.4's `OUTPUT`; `Ran`), awaits `Exited`, closes the child. `Event::Shutdown` closes it instead, which kills
  its group, and returns `Error::Interrupted`. The host is a plain loop over the machine's requests; `run`'s `--version`
  uses `run_one` too. The binary hands the shell jig's `profile(Small).io` (`jig-protocol-config/src/profile.rs:39`).
- **Locks:** `forgejo.lock`, `runner.lock` in the forge's directory (`File::{ForgejoLock, RunnerLock}`), 0o600, made on
  use. The host takes Forgejo's first, before it reads `Found`, and each other file 07.4's machine asks by
  `Request::Lock(File)` (a first preparation's runner lock, temper 15.1's older runner's), all held until it returns.
  A later preparation asks none, so it takes Forgejo's alone and a box restarts Forgejo beside a running runner.
- **Words:** one line `temper forge <prepare|run>: forge N: <what>`, exit 1, in `forge.rs`: `the setup has no entry
  for it`; `it is managed, not owned`; `forgejo is running or being prepared already` (`runner` likewise);
  `interrupted`; 07.4's `Failure`s as `<path> printed "<line>", not the pinned release`, `<command> ended with status
  <n>: <tail>` and, for `Token`, `no token for temper-admin (spec::ADMIN): ended with status <n>: <tail>`; `Unread`,
  `State`, `Unrecorded` and `Recorded` each a clause naming the command or the directory, never a secret. A forge
  directory not the user's own and private is refused naming it (`lib.rs:302-309`).
- **`allowed_host`, `Prepare::new`'s third argument, is none here:** its address comes from `application.webhooks`
  (temper 05.3); whichever of 05.3 and 08.1 lands second wires it, and temper 11.1 proves it.
- **`run` is checks and a host, not a machine:** pure `launch`, `admit_state`, `admit_binary` over 07.4's specs. A
  refused run exits and releases the lock. It does not wait for Forgejo: the runner retries its connection itself.
- **08.3 keeps provisioning the support's** until temper 10.2 (users, tokens, repository, collaborator, through Forgejo's
  API under temper's admin token); the bot's id goes to `forge-1/user`, a file the document names (07.3). The org
  `workshop` nobody makes; `acme` stays the support's. The throwaway's page cap of 2 goes: owned runs Forgejo's defaults
  (6.2); the cap's coverage is the conformance tool's.

## Rules that bind it

- **The lock and the exec are `process.rs`'s.** The boundary test scans `crates/*/src`, test files aside, and fails
  the gate on `std::process` or a file lock elsewhere, but `std::process::ExitCode` in `main.rs`; it scans neither
  `testing/` nor `tools/`. Tests under `tests/` may use `std::process::Command` (`support/live.rs:140` spawns git by it; the tier starts `temper` through skein-world's `Binary`, `tests/support/mod.rs:85`).
- **No secret in an argument, a refusal or a `Debug`.** A refusal prints versions, paths and commands only (`state` holds
  Forgejo's secrets). Checked by 07.4's `no_argument_of_any_command_holds_a_secret`, 08.2's
  `no_secret_of_the_state_reaches_a_refusal` and 07.2's `a_state_debug_shows_no_secret`; no other type holds one, and the
  author keeps it so.
- **The binaries,** exported in the shell that runs a command below, on a line of their own: `export
  TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`. The focused
  tests use scripts. No live service, no sign-in.
- **Hotspots shared with client-api (README):** marked in Files (to about 10-18 20Z); root `Cargo.toml` is also 07.1's and
  12.1's. A rebase takes origin/main's `Cargo.lock`, then `cargo check -p temper-forge-setup-shell -p temper
  --tests` (no `cargo update`).

## Increments

### 08.1 `temper forge prepare CONFIG.json FORGE`
Scope: setup.md 6.3 and 8 for an owned forge: the locks, the checks, the commands on skein-io, the files.
Needs: temper 07.3 (the document's checks), 07.4 (the machine and the specs; through it 07.2 and temper 03).
Files: new `crates/temper-forge-setup-shell/`: `Cargo.toml` (own `[lints]` as `crates/temper/Cargo.toml:69-84`; deps
`temper-forge-setup`, `skein-io`, `skein-lib`, `skein-shell`, and `temper-forge-forgejo`, `skein-json` (protocol.md 5,
answer 44; first used by temper 10.1); dev `jig-protocol-config`), `clippy.toml` as
`crates/temper/clippy.toml`, `src/{lib,process,command,files,prepare}.rs`, `tests/{boundary,locks,commands,files}.rs`,
`tests/support/mod.rs` (the crate's fixtures: `Scratch`, `script`, `io()`; 08.2 and 10.1 extend; each test file
declares it `pub mod support;`, as `crates/temper/tests/configuration.rs:1`, or clippy finds its unused helpers dead). `crates/temper`:
`Cargo.toml`, new `src/forge.rs`, `src/lib.rs:22,26-31,63-75` (USAGE, `Command::Forge { arguments }`, its arm;
**hotspot**), `src/tests.rs:7,14`, new `tests/forge.rs` (default suite), `tests/process.rs` (one test; **hotspot**),
`tests/support/mod.rs` (a `pub mod owned;`; **hotspot**), new `tests/support/owned.rs`. Root `Cargo.toml`, `Cargo.lock`.
Shape: sketches 1 and 2. `process.rs`: `lock` (`File::try_lock`; `WouldBlock` is `Held`), `command` (program,
arguments, `env_clear` then the spec's environment, `current_dir` its directory, the lock file as stdin, the rest
inherited), `exec` (`CommandExt::exec` on it). `files.rs`: `write_private` (beside its name, 0o600, synced, renamed
over, the directory synced), `clear` (the work path, the runner's directory, `admin_token`, `person_password`; never
`backups/` or `state`), `layout` (setup.md 3's directories, 0o700, `forgejo/repositories` too). `prepare::run` takes
Forgejo's lock, reads `Found` under it (`state` by `skein_shell::read_private_file`, `admin_token`, `files`: whether any
of what `Clear` removes exists), then loops on `step`: `Version` (its first stdout line is `Printed`), `Run` and `Token`
through `command::run_one`, `Lock(file)` by `process::lock` (`Held` ends the run), `Draw` by `skein_shell::crypto_seed`,
the rest by `files`; a command that cannot start ends the host, the machine's `End` returns. `forge.rs` has `sign_in.rs`'s
shape, `start(arguments) -> Result<u8, Refused>` and `run`: it reads the file (`CONFIG_BYTES`), calls 07.3's
`setup::check`, takes the entry (`Checked::setup.entry`; absent or managed refused), makes `Place` (`directory` absolute
against the working directory, as `lib.rs:369`; `user` the login of `effective_user()` in `/etc/passwd`), makes the
directory 0o700 and checks it, and calls `prepare::run` with `Prepare::new(owned, place.clone(), allowed_host)`.
`owned.rs`: `Names` (binaries, port, bot, person, orgs; a runner's `labels` are `["temper-host:host"]`, as 03.3's facts, and
its `path` has git, base64 and sh: `/usr/bin:/bin`), `document(value, &Names)` (forge 1's `api`, `git`, `api_token`, `user`
as a file, its identity, and the owned `setup` entry), `configuration(directory, &Names)` (`config.json` from
`support::configuration`, then `document`), `prepare(directory: &Path, forge: u16) -> Output`, `files(directory)` (the secret files),
`scripted(directory) -> Programs` (two `#!/bin/sh` scripts for the programs, printing the pin on `--version` and what 03's
`cli-` facts record for each command; `config.json` over them; `Duration` is `skein_lib`'s, as `support::WITHIN`):
`Programs::fail(&self, program: &str, command: &str)` makes a command fail until `mend(&self, program: &str)`;
`hang(&self, program: &str, command: &str)` makes it start `sleep 60` and wait for it, `hung(&self, within: Duration) -> u32`
answers that pid; `finished(directory)` (the forge's directory and a finished `state` at the pins, by 07.2's `state::encode`).
Test shape: shell tests are std-only integration tests on `support::Scratch` under `CARGO_TARGET_TMPDIR`, each under
0.5 s. Limits: jig's `profile(Small).io` (`support::io()`), none of its own.
- `locks.rs`: `a_second_lock_is_refused_while_the_first_is_held`; `a_child_holds_the_lock_as_its_stdin_after_the_
  parent_lets_go` (`/proc/<pid>/fd/0` of what `process::command` spawns is the lock file);
- `commands.rs`: `a_command_ends_with_its_status_and_both_streams` (reference); `output_past_the_bound_is_cut_and_
  the_command_ends`; `a_program_that_is_not_there_is_reported_not_run`;
- `files.rs`: `a_file_is_written_whole_private_and_renamed_over`; `clear_removes_a_first_preparations_files_and_
  never_backups`; `boundary.rs`: `std_process_and_file_locks_appear_only_in_the_process_module` (the Rules' scan),
  `the_scan_catches_each_form_and_passes_other_process_modules` (literal rows: `use std::{fs, process::Child};`,
  `std::process::exit(1)`, `CommandExt`, `try_lock` fail; `smith::process::Event` passes).
Binary tests, `tests/forge.rs`, on `owned::{configuration, scripted}`: `a_second_preparation_is_refused_while_the_
first_holds_the_lock`; `a_preparation_cut_short_is_begun_again_from_an_empty_layout` (reference);
`forgejo_files_with_no_state_are_refused_naming_the_directory`; `a_later_preparation_takes_forgejos_lock_alone` (the
test holds `runner.lock`: a later one ends 0, a first one is refused naming the runner);
`a_forge_directory_open_to_others_is_refused_naming_it`; `a_termination_signal_kills_the_commands_group_and_ends_the_
preparation_interrupted` (`hang("forgejo", "migrate")`, `sh -c 'kill -TERM <pid>'` once `hung` answers: exit 1, stderr
`temper forge prepare: forge 1: interrupted`, the `sleep` gone within `WITHIN`, `state` decoding with `prepared: false`).
In `src/tests.rs`, rows as :106-123: `forge_without_its_arguments_prints_its_usage`.
```rust
// crates/temper-forge-setup-shell/tests/commands.rs
pub mod support;
use temper_forge_setup::prepare::Exit;
use temper_forge_setup_shell::command;
#[test]
fn a_command_ends_with_its_status_and_both_streams() {
    let scratch = support::Scratch::new("command_ends");
    let spec = support::script(scratch.path(), "echo out; echo err >&2; exit 3");
    let ran = command::run_one(&support::io(), &spec).expect("it runs");
    assert_eq!((ran.exit, ran.stdout.expose(), ran.stderr.expose()), (Exit::Code(3), &b"out\n"[..], &b"err\n"[..]));
    scratch.remove();
}
// crates/temper/tests/forge.rs
pub mod support;
use std::fs;
#[test]
fn a_preparation_cut_short_is_begun_again_from_an_empty_layout() {
    let scratch = support::Scratch::new("forge_cut_short");
    let programs = support::owned::scripted(scratch.path());
    programs.fail("forgejo", "admin");
    let first = support::owned::prepare(scratch.path(), 1);
    let stderr = String::from_utf8_lossy(&first.stderr);
    assert_eq!(first.status.code(), Some(1), "{stderr}");
    assert!(stderr.starts_with("temper forge prepare: forge 1: ") && stderr.contains("ended with status 1"), "{stderr}");
    fs::write(scratch.path().join("forge-1/forgejo/left-over"), b"x").expect("a left-over file");
    programs.mend("forgejo");
    let second = support::owned::prepare(scratch.path(), 1);
    assert_eq!(second.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&second.stderr));
    assert!(!scratch.path().join("forge-1/forgejo/left-over").exists(), "begun again from an empty layout");
    let state = temper_forge_setup::state::decode(&fs::read(scratch.path().join("forge-1/state")).expect("state"));
    assert!(state.expect("a state").prepared, "state ends last");
    scratch.remove();
}
```
Proof: process, `forge_prepare_makes_an_owned_forge_then_remakes_nothing` (`tests/process.rs`, both pinned binaries):
`owned::configuration` with a runner, `temper forge prepare` exits 0, the five secret files exist and are private,
`runner/secret` holds 40 hex digits, a second preparation exits 0 with each byte-equal. Command: `sh -c
'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper && cargo nextest run -p temper --profile process -E
"test(=forge_prepare_makes_an_owned_forge_then_remakes_nothing)"'` ends `1 test run: 1 passed`, about 10 s, at most
30 s. Nothing runs higher: Forgejo runs first in 08.2.
Don't: exec anything (08.2); read `user`, `api_token` or `webhooks` files; let temper-service name either crate; take
the runner's lock but when the machine asks; wait for a lock; let a command outlive a termination signal.
Size: 310 (the commands on skein-io, signals included, about 110 of it).
Removes: nothing.

### 08.2 `temper forge run CONFIG.json FORGE forgejo|runner`
Scope: setup.md 6.4 and 6.1 for an owned forge: lock, check, exec.
Needs: 08.1 (`forge.rs`, `process`, `command`, `owned.rs`), 07.4 (`spec::command` for `Web` and `Daemon`).
Files: `crates/temper-forge-setup/src/run.rs` (new), `lib.rs`, `src/tests/run.rs` (new), `src/tests/spec.rs` (07.4's:
rows for `Web` and `Daemon`), `src/tests/fixtures.rs` (extended); `crates/temper-forge-setup-shell/src/run.rs` (new),
`lib.rs`; `crates/temper/src/forge.rs` (08.1's: the `run` arm and its words), `src/lib.rs:22` and `src/tests.rs:7,14`
(**hotspot**: one literal); `tests/forge.rs` (extended), `tests/process.rs` (**hotspot**: one test),
`tests/support/owned.rs` (extended: the scripts' `web` and `daemon`).
Shape: sketches 3 and 4. `launch` gives `Launch` from `Owned` and `Place`: `lock` is `place.path(File::ForgejoLock |
RunnerLock)`, `state` is `File::State`, `version` is `spec::version(which, ..)`, `exec` is `spec::command(Web | Daemon,
..)`: arguments and environment are 07.4's, written once, so `run` and `prepare` cannot disagree. `NoRunner` when `runner`
is asked of a setup without one. `admit_state` refuses a missing (`NotPrepared`), unfinished (`prepared: false`) or
other-versioned `state`, comparing `state.forgejo` with `VERSION` and, when `launch.runner`, `state.runner` with
`RUNNER_VERSION`; every difference is `Versions` here (temper 15.1 gives it a direction). `admit_binary` applies
`names_forgejo` or `names_runner` to the first line. Host order, so that no upgrade outruns the checks: `process::lock`
(`Held` is `Locked`; `Io(NotFound)`, no directory, is the not-prepared refusal), `read_private_file` of `state` and
`state::decode`, `admit_state`, `command::run_one` of `version`, `admit_binary`, `process::exec`. `forge.rs` takes `run
CONFIG FORGE forgejo|runner` beside `prepare`, selects the entry and builds `Place` as 08.1 does, and words the `Failed`:
`the setup has no runner`; `not prepared`, `its preparation did not end` and `prepared at <recorded>, this temper pins
<pinned>`, each then `: run temper forge prepare CONFIG.json N`; `<path> printed "<line>", not the pinned release`;
`forgejo is running or being prepared already` (`runner` likewise); 08.1's `interrupted` for `Version`. Any other word is
the usage.
Test shape: step tests in `src/tests/run.rs` on `fixtures::{owned, place}` (07.1, 07.2), each under 20 ms; the pin's
lines are built with a `Writer` from `VERSION` and `RUNNER_VERSION`. Limits: none of its own.
- `the_run_refuses_a_forge_not_prepared_at_the_pins` (rows: absent, unfinished, older, newer, runner other);
  `the_run_refuses_another_binary_naming_path_and_line` (other Forgejo; `+` build part accepted; runner `v13.2.0`
  exact, `v13.2.00` refused); `the_run_refuses_a_runner_the_setup_lacks`;
- `the_run_specs_name_work_path_configuration_and_environment` (`Web`, `Daemon`, literals; in `spec.rs`).
Binary tests in `tests/forge.rs` (each under 0.5 s), on `scripted` (`--version` prints the pin; `web` and `daemon`
write `$$`, the arguments and `env` to a marker, then `exec sleep 60`) and `finished`, with two more
helpers in `owned.rs`: `Programs::marker(&self, program: &str, within: Duration) -> Marker` waits for the marker,
then for `/proc/<pid>/comm` to read `sleep`, and fills `descriptors` from `/proc/<pid>/fd` then (before the exec, a
script's own `/proc/$$/fd` also lists its script's and a pipe's descriptors)
(`Marker { pid: u32, arguments: Vec<String>, environment: Vec<String>, descriptors: Vec<u32> }`) and
`Programs::lock(&self, program: &str) -> PathBuf` is that program's lock file:
`a_second_run_is_refused_while_the_first_holds_the_lock`;
`the_run_replaces_itself_holding_the_lock_as_standard_input` (reference);
`the_new_process_sees_the_specs_environment_alone` (temper itself started with a leaked variable);
`the_new_process_holds_no_descriptor_but_the_standard_three`; `no_secret_of_the_state_reaches_a_refusal`.
```rust
// crates/temper/tests/forge.rs
use std::process::Command;
#[test]
fn the_run_replaces_itself_holding_the_lock_as_standard_input() {
    let scratch = support::Scratch::new("forge_run_replaces_itself");
    let programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut run = Command::new(env!("CARGO_BIN_EXE_temper"))
        .args(["forge", "run", "config.json", "1", "forgejo"])
        .current_dir(scratch.path()).spawn().expect("temper forge run starts");
    let marker = programs.marker("forgejo", support::WITHIN);
    assert_eq!(marker.pid, run.id(), "the script is the process temper started");
    let stdin = fs::read_link(format!("/proc/{}/fd/0", run.id())).expect("standard input");
    assert_eq!(stdin, programs.lock("forgejo"), "standard input is Forgejo's lock file");
    run.kill().and_then(|()| run.wait()).expect("script killed and reaped");
    scratch.remove();
}
```
Proof: process, `a_prepared_forge_runs_forgejo_and_its_runner_once_each` (`tests/process.rs`, both pinned binaries,
08.1's `owned::{configuration, prepare}`): after `prepare` with a runner, `run forgejo` answers `GET /api/v1/version`
with the pin's release and `forge-1/admin_token` answers `GET /api/v1/user` as site admin `temper-admin` (30 s at
most each), `run runner` logs `declared successfully` (30 s), a second `run forgejo` exits 1 with `forge 1: forgejo
is running or being prepared already`, and killing both leaves neither answering. Command: `sh -c
'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper && cargo nextest run -p temper --profile process
-E "test(=a_prepared_forge_runs_forgejo_and_its_runner_once_each)"'` ends `1 test run: 1 passed`, about 20 s, at
most 60 s. It is also the first run of the runner on 07.2's configuration (Question 1). Nothing runs higher.
Don't: spawn the final process through skein-io; write a second environment or argument function (07.4's serve);
wait for a lock; read any file but `state`; print `state` beyond versions; use `unsafe` or `libc`.
Size: 160.
Removes: nothing.

### 08.3 The process tier's Forgejo is started by `prepare` and `run`
Scope: setup.md 14 (4) and 13.4's first half: `Throwaway::start` prepares and runs the tier's Forgejo through the
binary; users, tokens, the repository and the collaborator stay the support's until temper 10.2.
Needs: 08.2.
Files: `testing/temper-throwaway-forgejo/src/instance.rs` (`attach`; the ready wait of `start`, :64-77, becomes a
method both use); `crates/temper/tests/support/forgejo.rs` (**hotspot**: `start` :16-26, `stop` :105-107,
`configuration` :114-133), `support/mod.rs` (**hotspot**: `Forge::Throwaway` :11-14 gains `forgejo: &'a Path`;
`configuration` :19-41 makes its directories with `create_dir_all`). `process.rs:36,73` and `live.rs:8` call `start`.
Shape: sketch 5. `Throwaway::start(scratch)` keeps its signature and does, in order: `Forgejo::from_env()` (the
refusal naming `TEMPER_FORGEJO` stays); a free loopback port; `owned::configuration` (forge 1 owned on that port, no
runner, bot `temper-bot`, person `ada`, orgs `["workshop"]`: names that clash with neither `acme` nor `temper-bot`
nor `temper-admin`); `owned::prepare` (exit 0, else a panic with its stderr); `temper forge run config.json 1 forgejo`
spawned with its output in `<scratch>/server.log`, the one log name (`attach` reads it; temper 10.1's `owned::run` writes
Forgejo's there too); `Instance::attach` over the child with `forge-1/admin_token` as the admin's token; provisioning as
:21-24 does it today. All of it is one function, for temper 10 to lift out.
`forgejo::configuration` then edits the document by `owned::document` with the bot's token and id, writing them to
`forge-1/{api_token,git_token,user}` as it writes `secrets/forge-1.*` today (:116-117). `stop` is `instance.stop()`: the
attached instance kills its child and removes nothing (the scratch is the test's); so does a panicking test's `Drop`.
Test shape: no new test: the increment is test support, and the tier's six tests are its test, two of them on a
Forgejo (`temper_open_adopts_a_repository_by_commands`, `a_persons_change_lands_on_the_forge_and_is_found_after_a_
restart`). Limits: the tier keeps the shipped profile's (`support/configuration.rs:43`, `"profile":"small"`);
`Settings::small()` still gives `attach` its answer bound.
Proof: process, the tier whole, once, `TEMPER_FORGEJO` set: `scripts/proof-one-process.sh` exits 0 and
every test passes, the landing story merging its pull request on an owned Forgejo (base: 42.4 s for six tests,
`findings/forge-and-tiers.md` 4; now at most 42.4 s plus the ceilings of the tests added to the `process` binary
since the base: 08.1's 30 s, 08.2's 60 s, and each other landed by then, named in the report). Second, the live binary in its fake mode, which shares `start`
(no `TEMPER_RUN_LOGS`): `sh -c 'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper &&
cargo nextest run -p temper --profile live'` ends `1 test run: 1 passed`, at most 90 s. Nothing runs higher:
reconcile and CI are temper 10.
Don't: change `Instance::start` or `Settings` (the conformance and git tools use them); run reconcile or the runner;
read a password (the admin has none, 6.3).
Size: 40 (the process tier's support, counted in `crates/temper/tests/support`).
Removes: the tier's call of `Forgejo::start` (`forgejo.rs:18-20`) and its page cap.

## Interface sketches

```rust
// crates/temper-forge-setup-shell/src/process.rs (sketch 1): the one module of std's lock and process.
use std::{io, path::Path};
use temper_forge_setup::spec::Spec;
/// An exclusive lock on a forge's lock file, held while the value lives or until `exec` hands it on.
#[derive(Debug)] pub struct Lock { file: std::fs::File }
#[derive(Debug)] pub enum Refused { Held /* a running Forgejo or runner, or a preparation */, Io(io::ErrorKind) }
/// Takes the lock without waiting; makes the file 0o600 when absent.
pub fn lock(path: &Path) -> Result<Lock, Refused> { todo!() }
/// The command `exec` runs, `lock` as its standard input; the tests spawn it.
pub fn command(spec: &Spec, lock: Lock) -> std::process::Command { todo!() }
/// Replaces this process by the spec's program; returns only why it did not.
pub fn exec(spec: &Spec, lock: Lock) -> io::Error { todo!() }
// crates/temper-forge-setup-shell/src/command.rs (sketch 2): one spec run to its end on skein-io.
use temper_forge_setup::{prepare::Ran, spec::Spec};
/// Why a command did not run to its end.
#[derive(Debug)] pub enum Error {
    Kernel(String),          // not opened, or the termination signals not taken
    Spawn(skein_io::Error),  // no such program, or a slot short
    Interrupted,             // a termination signal: the child's group is killed and reaped
}
/// Runs `spec` on a kernel of its own: both streams read to their end (`spec.output` bytes of each kept), the exit awaited.
pub fn run_one(limits: &skein_io::Limits, spec: &Spec) -> Result<Ran, Error> { todo!() }
// crates/temper-forge-setup-shell/src/prepare.rs (08.1): hosts 07.4's machine.
use crate::command;
use std::io;
use temper_forge_setup::{place::{File, Place}, prepare::{Failure, Prepare}};
/// Why `run` returned without a finished preparation.
#[derive(Debug)] pub enum Failed {
    Held(File),                   // that lock is held: the process runs, or another preparation does
    Io(&'static str, io::ErrorKind), // a lock file or a file of the forge; the first word names which
    Command(command::Error),      // a command could not start, or a termination signal ended it
    Refused(Failure),             // the machine ended the preparation
}
/// Locks, reads what is found, and loops on the machine's steps until it ends.
pub fn run(limits: &skein_io::Limits, place: &Place, machine: Prepare) -> Result<(), Failed> { todo!() }
```
```rust
// crates/temper-forge-setup/src/run.rs (sketch 3, 08.2)
use crate::{place::Place, setup::Owned, spec::{Spec, Which}, state::State};
use alloc::boxed::Box;
/// Why a run did not start; the binary words each, naming the forge.
#[derive(Clone, PartialEq, Eq, Debug)] pub enum Refusal {
    NoRunner,    // `runner` was asked and the setup has none
    NotPrepared, // no `state`
    Unfinished,  // `state` says the first preparation did not end
    Versions { forgejo: Box<[u8]>, runner: Option<Box<[u8]>> }, // `state` records other versions than the pins
    Binary { printed: Box<str> },                               // the first `--version` line is not the pin's
}
/// What the host needs to run one process of one forge.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Launch {
    pub which: Which,      // which process
    pub runner: bool,      // whether the setup has a runner, so that `state` must record its version too
    pub program: Box<[u8]>, // the binary's absolute path, as the setup names it
    pub lock: Box<[u8]>,   // this process's lock file, the one the preparation takes too
    pub state: Box<[u8]>,  // the forge's `state` file
    pub version: Spec,     // the binary with `--version`, in the process's environment
    pub exec: Spec,        // the process: `Command::Web` or `Command::Daemon`
}
pub fn launch(owned: &Owned, place: &Place, which: Which) -> Result<Launch, Refusal> { todo!() }
pub fn admit_state(launch: &Launch, state: Option<&State>) -> Result<(), Refusal> { todo!() }
pub fn admit_binary(launch: &Launch, printed: &str) -> Result<(), Refusal> { todo!() }
```
```rust
// crates/temper-forge-setup-shell/src/run.rs (sketch 4, 08.2): returns only why it did not start.
use crate::command;
use std::io;
use temper_forge_setup::run::{Launch, Refusal};
/// Why `run` returned: it returns only when the process did not start.
#[derive(Debug)] pub enum Failed {
    Refused(Refusal),        // a check refused
    Locked,                  // another process holds the lock
    Lock(io::ErrorKind),     // the lock file could not be made or opened (`NotFound`: no directory)
    State(&'static str),     // `state` could not be read or decoded
    Version(command::Error), // the `--version` command could not run
    Exec(io::Error),         // the exec failed after the checks
}
/// Locks, checks, and replaces this process by the launch's; returns only why it did not.
pub fn run(limits: &skein_io::Limits, launch: &Launch) -> Failed { todo!() }
```
```rust
// testing/temper-throwaway-forgejo/src/instance.rs (sketch 5, 08.3)
use crate::{Failure, Instance, Settings, Token};
use std::{path::Path, process::Child};
impl Instance {
    /// An instance over the Forgejo `child` runs on `port`, `scratch` its directory (its `server.log` read for failures),
    /// `admin` an admin's token. Waits until the version answers. `stop` and `Drop` kill the child, remove nothing.
    /// # Errors
    /// A `Failure` when the version does not answer in time or the child exits first.
    pub fn attach(child: Child, port: u16, scratch: &Path, admin: Token, settings: &Settings) -> Result<Self, Failure> { todo!() }
}
```

## Questions to raise when reached

1. **The runner's first run** (07.2's `timeout`, `shutdown_timeout` and the connection's `uuid` source are the
   plan's until 08.2 runs it): if `declared successfully` is missing from 08.2's Proof, raise it with the runner's
   log; do not edit 07.2's text here.

## Done when

With the two binaries exported (Rules); the counts are of this session's own tests, which the filters name:
1. Prepare and run work on the pinned binaries: `sh -c 'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper
   --bin temper && cargo nextest run -p temper --profile process -E "test(=forge_prepare_makes_an_owned_forge_then_remakes_nothing)
   + test(=a_prepared_forge_runs_forgejo_and_its_runner_once_each)"'` ends `2 tests run: 2 passed`.
2. The whole tier passes on an owned Forgejo: README Done when 4's run (not repeated here) exits 0, and `git grep -n -F
   'actions: false' origin/main -- crates/temper/tests/support/forgejo.rs` prints nothing.
3. The focused binary tests pass in the default suite: `cargo nextest run -p temper --test forge -E
   'test(/a_second_(preparation|run)|a_preparation_cut_short|forgejo_files_with_no_state|a_later_preparation|a_forge_directory_open|a_termination_signal|the_run_replaces|the_new_process_|no_secret_of_the_state/)'`
   ends `11 tests run: 11 passed`, under 5 s.
4. The shell crate's tests pass, the boundary scan among them: `cargo nextest run -p temper-forge-setup-shell -E
   'test(/a_second_lock|a_child_holds|a_command_ends|output_past_the_bound|a_program_that_is_not_there|a_file_is_written_whole|clear_removes_a_first|std_process_and_file_locks|the_scan_catches/)'`
   ends `9 tests run: 9 passed`; the run's step tests: `cargo nextest run -p temper-forge-setup -E
   'test(/the_run_refuses_a_(forge_not_prepared|runner_the_setup)|the_run_refuses_another_binary|the_run_specs_name/)'` ends `4 tests run: 4 passed`.
5. One module locks and execs: `git grep -l -E 'process::(Command|Child|Stdio|exit|abort)|CommandExt|try_lock|lock_shared'
   origin/main -- 'crates/*/src/*' ':!*/tests/*' ':!*/tests.rs'` prints only `origin/main:crates/temper-forge-setup-shell/src/process.rs`.
6. The usage names both commands: `git grep -c --all-match -F -e 'temper forge prepare CONFIG.json FORGE'
   -e 'temper forge run CONFIG.json FORGE' origin/main -- crates/temper/src/lib.rs` prints `origin/main:crates/temper/src/lib.rs:1`.
