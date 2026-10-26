# Session 15: the upgrade

A preparation of an owned forge whose `state` records a Forgejo older than the pin dumps it into
`backups/forgejo-<old>-to-<new>.zip`, migrates it and records the pin; one whose runner is older has the runner's files
prepared again, with no dump; a recorded version newer than its pin, Forgejo's or the runner's, is refused, and `temper
forge run` words that direction (15.1). A runner added to a prepared forge is registered, and one removed recorded as
none (setup.md 6.3 item 4). The plan's last temper work: a box at the pinned versions never meets it
(decisions.md, 10, 17). It extends temper 07's and 08's types in place.

## Read first

- Design: setup.md 6.3 (its preamble, items 3 and 4, the commands' bullets, `Interrupted`), 6.4 (the locks, `run`'s checks), 3 (`backups/`),
  13.1; the designer's answers 20, 21, 38 and 39 (in setup.md 6.3, temper `b05af1f7`). Plan: decisions.md 8, 10, 17;
  `temper/03-command-line.md`: 03.2's `cli-dump-…` and `cli-migrate-…` facts, 03.3's `runner-register-…` facts. skein:
  `programming-model.md` 3, `testing-strategy.md` 2.1. temper: `workflow.md` 2.
- temper 07 and 08, by name: `prepare::{Prepare, step, Event, Request, Found, Failure}`, `spec::{Command, Spec, command}`,
  `place::{Place, File}`, `state::{State, encode, decode}`, `Drive` (07.4); `run::{Refusal, admit_state}` (08.2);
  `crates/temper/src/forge.rs`, `tests/support/owned.rs`, the shell's `prepare.rs` and `files.rs` (08.1).
- Code at the base: `.config/nextest.toml:30-33` (the process profile), `crates/temper/tests/process.rs:1-5, 100-111`,
  `tests/support/mod.rs:6,44-71,518` (`WITHIN`, `Scratch`, `private`).

## What the design asks

- **6.3, item 3:** an older Forgejo is dumped (a backup already there is not made again, none is overwritten), then
  migrated, then recorded; an older runner has its files prepared again under the runner's lock too, with no dump; a newer
  one, Forgejo's or the runner's, refuses. The dump is a command like the others: skein-io, output bounded and masked
  (13.1). **Interrupted** (answer 38): `state` is not marked ended, so the next preparation repeats it.
- **6.3, item 4** (the designer's answer to ruling 120's questions): "A runner added is prepared under the runner's lock
  too, as a first preparation prepares it: its secret drawn into `runner/secret` when that file is missing, and kept
  otherwise; registered; its connection written. A runner removed is recorded as none, and its files and its
  registration are left, unused". "A runner's recorded version is compared with its pin only while the setup has one."
- **6.4:** `run` checks "that `state` records a preparation at the pinned versions, with a runner exactly when the setup
  has one; refuses otherwise, naming what to run: `temper forge prepare` for a forge not prepared, a preparation that did
  not end, an older version, or a runner added or removed (6.3); no command of this temper for a newer version, or one
  it cannot read".

## Choices this plan makes

- **An upgrade is a later preparation with steps inserted** (07.4's `Layout`, `app.ini`, then `admin_token` when missing):
  after `Layout` an older Forgejo is dumped; after `app.ini` it is migrated and an older runner is registered again and its
  configuration written; `state` follows with the pins. A failing command, or a termination signal (08.1's: the group
  killed, exit 1 `forge N: interrupted`), ends the preparation there, `state` unchanged, so the next run repeats the
  upgrade: it finds the dump and keeps it (a half-migrated database is never dumped over it), and `forgejo migrate` runs
  again harmlessly (fact `cli-migrate-runs-again-on-a-migrated-work-path`). The dump is taken before `app.ini` is written,
  so it holds the files Forgejo ran with.
- **The partial is one fixed name,** `backups/dump.partial.zip` (setup.md 6.3 item 3, answer 39): Forgejo's `dump` appends
  `.zip` to a `--file` that does not end with it (found by hand on 17.0.0: `x.zip.partial` is written as
  `x.zip.partial.zip`). A partial that a dump cut short left is overwritten by the next dump (by hand on 17.0.0, over
  garbage bytes: exit 0, a zip). The spec is `dump --file <partial> --tempdir <directory>/backups` after `--work-path` and
  `--config`, as 03.2's facts run it: Forgejo refuses a missing tempdir after creating an empty file, and its default,
  `/tmp`, is shared. Forgejo writes the file `0o600`.
- **A runner is prepared again by registering again** (the same secret file: idempotent, the same uuid; facts
  `runner-register-again-keeps-one-global-runner`, `runner-uuid-is-the-secrets-first-sixteen-characters-in-hex`) and writing
  its configuration with that uuid. `runner/secret` (also the connection's token file) and `state`'s secrets stay.
- **Versions are compared as dotted decimals;** the pins carry no `v`. A recorded text that is not one
  (`17.0.0-rc1`, garbage, empty) is `Unreadable`, refused like a newer one, since it cannot be shown older (the
  design is silent).

## Rules that bind it

- **The lock and the exec stay in 08.1's module.** The upgrade adds an existence check and a rename to the shell's
  `files.rs`, and no `std::process`, lock or `CommandExt`; 08.1's boundary test fails the gate if it does.
- **No secret in an argument or a refusal.** The backup holds the database: private, in a private directory, and nothing
  prints it; a refusal names versions. The programs are the pinned binaries, exported in the shell that runs a command
  below: `export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`. No live
  service, no sign-in.
- **Hotspots shared with client-api (README):** `crates/temper/tests/process.rs` and `tests/support/*` (to about 10-18 20Z).
  A rebase takes origin/main's `Cargo.lock`, then `cargo check -p temper-forge-setup -p temper-forge-setup-shell -p
  temper --tests`; 15.1 adds no dependency, so no `cargo update`.

## Increments

### 15.1 The upgrade
Scope: setup.md 6.3 items 3 and 4, 6.4 and 13.1: an older Forgejo is dumped, migrated and recorded; an older runner has
its files prepared again; a runner added is registered and one removed recorded as none; a newer or unreadable version of
either is refused by `prepare` and `run`, and `run` names `prepare` for a runner added or removed.
Needs: temper 08.2 (`run.rs`, `forge.rs`), and through it 08.1 (`temper forge prepare`, the shell's host,
`files.rs`, `owned.rs`) and 07.4 (the machine, `Drive`); 03.2 and 03.3 (the dump and runner facts).
Files: `crates/temper-forge-setup/src/upgrade.rs` (new), `lib.rs` (a `mod`), `src/prepare.rs` (07.4's: the arm),
`src/spec.rs` (`Command::Dump`), `src/place.rs` (`File::DumpPartial`, `Place::backup`), `src/run.rs` (08.2's:
`Refusal::Versions`), `src/tests/upgrade.rs` (new), `src/tests.rs` (`mod upgrade;`), `src/tests/prepare.rs` (07.4's
`a_recorded_version_other_than_the_pin_is_refused`, `tests/prepare.rs:196-203`, goes: its one row, Forgejo
`another-version`, is the garbage row of `the_upgrade_refuses_…`, and a runner's presence is no longer refused), `src/tests/run.rs`
(08.2's), `src/tests/files.rs` (a row
`(File::DumpPartial, b"/srv/forge-2/backups/dump.partial.zip")` in the table at `:43-56`), `src/tests/spec.rs` (`Command::Dump` in `no_argument_of_any_command_holds_a_secret`'s list, `:99-111`),
`src/tests/fixtures.rs` (`runner_secret` in the `Found` of `first()`, false, and `later()`, true, `:152`, `:156-165`; 07.1's, extended: `Drive::reach`, `older`,
`backup`, `ran_ok`); `crates/temper-forge-setup-shell/src/{prepare,files}.rs`, `tests/files.rs` (08.1's);
`crates/temper/src/forge.rs` (words), `tests/forge.rs` (three tests), `tests/process.rs` (**hotspot**: two tests),
`tests/support/owned.rs` (**hotspot**: `record_versions`, `file`; a `state` without a runner is made inline, as `tests/forge.rs:65-68` makes
`prepared`, each row on a `Scratch` of its own, since `owned::configuration` creates its directory). The docs that say only a first preparation takes
the runner's lock follow setup.md 6.3's preamble: `Request::Lock` (`prepare.rs:58`), `File::RunnerLock` (`place.rs:35-36`).
`Failure::Recorded` (`prepare.rs:90-91`) goes, with its words in `forge.rs`: each case it held now has its own course.
Shape: sketch 1, and these changes in place. `Command` gains `Dump`, built by `spec::command`; `File` gains `DumpPartial`,
and `Place::backup(name)` is `<directory>/backups/<name>`. `Request` gains `Exists { path: Box<[u8]> }` (answered
`Event::Exists(bool)`) and `Rename { from: Box<[u8]>, to: Box<[u8]> }` (answered `Done`); `Failure` gains `Newer { which:
Which, recorded: Box<[u8]> }` and `Unreadable { which: Which }`, and `Recorded` goes. `Found` gains `runner_secret: bool`,
whether `runner/secret` exists, which the host fills as it fills `admin_token`. On a finished `state` and passed binary
checks the machine takes `course(state.forgejo, VERSION)` and, when the setup and `state` both have a runner,
`course(state.runner, RUNNER_VERSION)`; a runner in the setup and not in `state` is added, one in `state` and not in the
setup removed, whatever version it records (setup.md 6.3 item 4). It keeps Forgejo's `Course` and the runner's (a
`Course`, added, removed or none: an enum of the machine's own) as fields of `Prepare` (`prepare.rs:114`), never as a
`bool`: it has three, and a fourth trips clippy's `struct_excessive_bools`; `Found.runner_secret` rides in the
runner's enum (`Added { secret: bool }`). A
`Newer` or `Unreadable` ends the preparation before `Layout`, nothing written; Forgejo's `Same` with the runner's `Same`,
or no runner in either, is 07.4's later preparation;
else, with an older or added runner `Request::Lock(File::RunnerLock)` first (08.1's host takes it without waiting; a held one ends
the run naming the runner, so an older runner's files are never prepared beside a running runner; a Forgejo-only upgrade
takes Forgejo's lock alone, as setup.md 6.3 item 3 reads, and goes on beside a running runner), its `Done` going on to
`Layout` (`Phase::Lock`'s arm, `prepare.rs:400`, goes to `begin()` only for a first preparation), and after `Layout`: `Exists(backup)`
and, when absent, `Run Dump` then `Rename` partial to backup (Forgejo older); `Write app.ini`; `Run Migrate` (Forgejo
older); `Draw` and `Write RunnerSecret` (runner added, and `Found.runner_secret` false: the draw's index 4,
`prepare.rs:232`); `Run Register` and `Write RunnerConfiguration` (runner older or added); `Write State` with the pins, and
the runner's only when the setup has one; then the later preparation's `admin_token` when missing. A runner removed adds
no request: `app.ini`'s `[actions]` follows the setup (07.4's `ini::text`), and its files are left. The backup is `backup_name(state.forgejo, VERSION)`. The host answers `Exists` and
`Rename` with `files::exists(&Path) -> io::Result<bool>` and `files::rename(&Path, &Path) -> io::Result<()>` (sync the file,
rename it, sync the directory). `Refusal::Versions` gains `course` (the worse of the two). `admit_state` checks the runner's presence both ways, before
the versions: `Refusal::Runner { recorded: bool }` (`recorded`: `state` has a runner the setup lacks); `Versions`
compares the runner's version only when both have one. `forge.rs` words `Newer` for
`prepare` as `forge N: forgejo is prepared at <recorded>, newer than this temper's pin <pinned>: its migrations only go
forward` (`runner` likewise), `Unreadable` as `... is prepared at a version this temper cannot read`; `run` words
`Refusal::Versions` by its `course`: `Older` as 08.2's, `Newer` as 08.2's with the tail `: no command of this temper runs
it`, `Unreadable` as `prepared at a version this temper cannot read: no command of this temper runs it` (the recorded bytes
are not echoed); `Refusal::Runner` as `prepared with a runner, and the setup has none` or `prepared without a runner, and
the setup has one`, each then 08.2's `: run temper forge prepare CONFIG.json N`.
Test shape: `src/tests/upgrade.rs` on `fixtures::Drive` (< 20 ms each); rows are literals; the recorded versions are
inputs (`16.0.5`, `13.1.0`; `older(forgejo, runner)` takes `None` for the pin), the pins' names come from `VERSION` and
`RUNNER_VERSION`. Limits: none of its own.
- `the_upgrade_dumps_an_older_forgejo_before_it_migrates` (reference, below)
- `the_upgrade_names_its_dump_by_both_versions_and_writes_a_partial` (the Dump spec's arguments, the rename)
- `the_upgrade_does_not_dump_again_when_the_backup_is_there` (`Exists(true)`: no Dump, no Rename, then `app.ini`)
- `the_upgrade_stops_where_a_command_failed` (rows: dump, migrate, register; no later request, no `State`; a new
  `Drive` on the same input asks from the start again: answer 38)
- `the_upgrade_refuses_a_newer_or_unreadable_version_first` (rows: Forgejo newer, runner newer, garbage, empty, five
  components `1.2.3.4.5`, ten digits `1234567890.0.0`)
- `the_upgrade_compares_versions_by_number_not_text` (`9.9.9` < `10.0.0`, `1.10.0` > `1.9.0`, `1.0` = `1.0.0`; the
  bounds read: `1.2.3.4`, `999999999.0.0`)
- `the_upgrade_prepares_an_older_runner_again_without_a_dump` (`Lock(RunnerLock)` first, no Dump or Migrate, `State` last)
- `a_runner_added_to_a_prepared_forge_is_registered_under_its_lock` (rows: `Found.runner_secret` false: `Lock(RunnerLock)`
  first, `Draw`, `Write RunnerSecret`, `Register`, `RunnerConfiguration`, `State` with the runner's pin; true: the same
  without `Draw` and `RunnerSecret`)
- `a_runner_removed_writes_no_runner_file_and_records_none` (no `Lock`; `app.ini` with `ENABLED = false` (`ini.rs:64`); no
  `Register`, `RunnerSecret` or `RunnerConfiguration`; `State` with no runner)
- in `src/tests/run.rs` (08.2 split its refusals into one test each, ruling 128): `course` in the expectations of
  `the_run_refuses_an_older_preparation` (:17) and `…_a_newer_preparation` (:29); `…_a_different_runner_preparation`
  (:41-48) expects `Refusal::Runner { recorded: false }`; a new test, a setup without a runner on a `state` with one,
  expects `Refusal::Runner { recorded: true }` (`the_run_refuses_a_runner_the_setup_lacks`, :50, is `NoRunner`'s: name
  the new one otherwise)

Beside them (< 0.5 s each): the shell's `tests/files.rs` `a_renamed_file_exists_under_its_new_name_only`; in
`crates/temper/tests/forge.rs`, on 08.1's and 08.2's scripts, `a_newer_recorded_forgejo_refuses_the_preparation_untouched`
(`state`, `app.ini`, `backups/` unchanged; both versions named), `the_run_refuses_a_newer_recorded_version_in_its_own_words`
(rows: newer, unreadable; the words above), `the_run_names_prepare_for_a_runner_added_or_removed` (rows: a setup with a
runner on a `state` without one, and the reverse; exit 1, the words above) and `an_interrupted_upgrade_leaves_the_state_for_the_next_preparation` (as 08.1's
`a_termination_signal_kills_…`, with `owned::record_versions` Forgejo `16.0.5` and `hang("forgejo", "dump")`: exit 1 `forge
1: interrupted`, `state` byte-equal, no backup file).
```rust
// crates/temper-forge-setup/src/tests/upgrade.rs
use super::fixtures;
use crate::{
    place::File,
    prepare::{Event, Request},
    spec::Command,
    state,
};
use temper_forge_forgejo::pin::VERSION;

#[test]
fn the_upgrade_dumps_an_older_forgejo_before_it_migrates() {
    let mut drive = fixtures::Drive::new(fixtures::owned());
    let place = fixtures::place();
    let backup = place.backup(&fixtures::backup(b"16.0.5"));
    assert_eq!(drive.reach(fixtures::older(Some(b"16.0.5"), None)), Request::Layout, "the later preparation's layout");
    assert_eq!(drive.send(Event::Done), Request::Exists { path: backup.clone() }, "the backup is looked for");
    let Request::Run { command: Command::Dump, .. } = drive.send(Event::Exists(false)) else { panic!("dumped") };
    let rename = Request::Rename { from: place.path(File::DumpPartial), to: backup };
    assert_eq!(drive.send(fixtures::ran_ok()), rename, "the partial becomes the backup");
    let Request::Write { file: File::AppIni, .. } = drive.send(Event::Done) else { panic!("app.ini after the dump") };
    let Request::Run { command: Command::Migrate, .. } = drive.send(Event::Done) else { panic!("then migrated") };
    let Request::Write { file: File::State, contents } = drive.send(fixtures::ran_ok()) else { panic!("recorded") };
    assert_eq!(&*state::decode(contents.expose()).expect("a state").forgejo, VERSION.as_bytes(), "the pin, last");
}
```
Proof: process, both binaries exported (Rules), the tree committed. `an_older_recorded_forgejo_is_dumped_then_upgraded`
(`tests/process.rs`; 08.1's `owned::{configuration, prepare}`, with a runner): `owned::record_versions(directory, forgejo,
runner)` rewrites `state`'s versions, `None` keeping one (07.2's `decode`/`encode`), here Forgejo to `16.0.5`, and a
`backups/dump.partial.zip` of garbage bytes is put there (what a dump cut short leaves); the second preparation exits 0,
`backups/` holds one `0o600` file, `forgejo-16.0.5-to-<VERSION>.zip`, starting `PK` and holding `forgejo-db.sql` and
`app.ini` (`skein_lib::bytes::find`), no `dump.partial.zip`, and `state` records the pin; recorded older again, a third
preparation leaves that file byte-equal, same mtime. `an_older_recorded_runner_has_its_files_prepared_again_without_a_dump`:
`state` rewritten to runner `13.1.0`, the runner's configuration (`owned::file(directory, forge, File)`) removed; the
preparation exits 0, the configuration is back byte-equal (its uuid came from the new registration), `backups/` is empty.
Command: `sh -c 'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper && cargo nextest run -p
temper --profile process -E "test(=an_older_recorded_forgejo_is_dumped_then_upgraded) +
test(=an_older_recorded_runner_has_its_files_prepared_again_without_a_dump)"'` ends `2 tests run: 2 passed`, about 30 s, at
most 90 s. Nothing runs higher: the migration finds nothing to do on a v17 database (its order after the dump is the unit
test's), and no older binary is used. A runner added runs the first preparation's `Register` and configuration, which
08.1's process test runs on the real binaries: no process test of its own.
**`JWT_SECRET` is Forgejo's (ruling 128):** Forgejo reads it as base64 in the URL alphabet without padding, 32 bytes;
the 64 hexadecimal digits of `prepare.rs:230` decode to 48, so the first Forgejo command after each `app.ini` logs
`JWT_SECRET … invalid base64 decoded length: 48 … creating new key` on stdout and rewrites `app.ini`, and `ran`'s
`first_line` (`prepare.rs:265` for `Register`, `:380` for `Token`) reads that line as the uuid or the token. The draw
of index 2 becomes `skein_lib::base64::encode(Alphabet::Url, ..)` of the 32 bytes (43 characters); refresh
`tests/prepare.rs:128`'s expectation. No `Migrate` is added before `Register`. The check: in 08.1's
`forge_prepare_makes_an_owned_forge_then_remakes_nothing` (`process.rs:114`), a third preparation after removing
`admin_token` exits 0, writes `admin_token` again, its first line 40 hexadecimal digits, and leaves `app.ini` byte-equal
to the second preparation's.
Don't: touch the first-preparation or same-version paths; prune, restore or overwrite a backup; dump on a first
preparation or after a preparation that did not end; name a version in production code.
Size: 230 (150 in the drafts: the runner's arm and the partial's rename are the difference; a runner added or removed,
about 50).
Removes: `Failure::Recorded` and its words (08.1's); 07.4's `a_recorded_version_other_than_the_pin_is_refused`.

## Interface sketches

```rust
// crates/temper-forge-setup/src/upgrade.rs
use alloc::boxed::Box;

/// How a recorded version stands against its pin; a later variant is the worse.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Course {
    /// The same version: a later preparation.
    Same,
    /// Older: Forgejo is dumped, migrated and recorded; the runner is prepared again.
    Older,
    /// Newer: refused, since migrations only go forward.
    Newer,
    /// Not a version this temper reads: refused.
    Unreadable,
}
/// Dotted decimals compared component by component, a missing one as 0; at most four components of at most nine digits.
#[must_use]
pub fn course(recorded: &[u8], pinned: &[u8]) -> Course { todo!() }
/// The dump's name under `backups/`: `forgejo-<recorded>-to-<pinned>.zip`.
#[must_use]
pub fn backup_name(recorded: &[u8], pinned: &[u8]) -> Box<[u8]> { todo!() }
```

## Questions to raise when reached

A recorded version this temper cannot read is refused (Choices). Asked of the designer with ruling 120, and answered
(setup.md 6.3 and 6.4): a runner added or removed is converged by `prepare` (item 4), and `run` names `prepare` for it
(`Refusal::Runner`, above); the runner's lock is taken "when it writes the runner's files: at a first preparation, and
for a runner older than its pin (item 3) or added (item 4)".

## Done when

With the two binaries exported (Rules):
1. The upgrade works on the real binaries: 15.1's command ends `2 tests run: 2 passed`.
2. The default-suite tests pass: `cargo nextest run -p temper-forge-setup -p temper-forge-setup-shell -p temper
   -E 'test(~the_upgrade_) + test(~newer_recorded) + test(~an_interrupted_upgrade) + test(~a_renamed_file) +
   test(~a_runner_added) + test(~a_runner_removed) + test(~the_run_names_prepare)'` ends `14 tests run: 14 passed`.
3. No version in production code: `git grep -n -E '"v?[0-9]+\.[0-9]+\.[0-9]+"' origin/main --
   crates/temper-forge-setup/src crates/temper-forge-setup-shell/src ':!*/tests/*' ':!*/tests.rs'` prints nothing.
4. The default suite passes, 08.1's boundary test included: `cargo nextest run -p temper-forge-setup
   -p temper-forge-setup-shell -p temper` ends with every test passed.
