+++
id = "printed-words"
part = "printed-words"
kind = "feature"
needs = []
files = ["temper/crates/temper/src/printed.rs", "temper/crates/temper/src/printed/tests.rs", "temper/crates/temper/src/directories.rs", "temper/crates/temper/src/tests/program_environment.rs"]
effort = "high"
minutes = 60
+++

## Goal

The command on the welcome's Model step and ConnectModel names the binary that's running, and is short: `temper` when that binary is first on `PATH`, otherwise its path, with `$HOME` as `~/`, slashes collapsed. The three `TEMPER_` values read the same way. A box running from a checkout's `target/debug`, with a months-old `temper` first on `PATH`, prints a command that works.

Acceptance:

- Each rule of `temper/docs/design/client/README.md`, 4.6, *The words of a command for this machine* has a test on a pure function, with a fake `PATH`, `HOME` and file system, in `temper/crates/temper/src/printed/tests.rs`. Rows it must have:
  - first on `PATH` is this file by canonical paths, reached through a symlink: `temper`
  - first on `PATH` is another file, this one second: the path, not `temper`
  - none on `PATH`, `PATH` unset: the path
  - a directory or a non-executable named `temper` earlier on `PATH` is passed over (the fake's `canonical` answers `None`)
  - an empty or relative `PATH` entry counts for nothing
  - ` (deleted)` is dropped, then the rule holds: rebuilt at the same path and first on `PATH`, `temper`; not on `PATH`, the path without the suffix
  - no executable read: `temper`
  - `/home/person/src//a///temper` with `HOME=/home/person`: `~/src/a/temper`
  - `HOME=/home/person/` and `HOME=/home//person` lead the same
  - `/home/personal/temper` with `HOME=/home/person`: not led, `/home/personal/temper`
  - the path equal to `$HOME`: `~`
  - `HOME` unset, empty, relative or `/`: leads nothing
  - `/home/person/my dir/temper`: `~/'my dir/temper'`; `/my dir/temper`: `'/my dir/temper'`; an apostrophe: `'\''` within the quotes; a byte that isn't UTF-8: quoted, carried as it is
- `Directories::program_environment` answers: empty for the default directories with the word `temper`; the word alone for the default directories with another; all three `TEMPER_` values as words, then the word, otherwise. `TEMPER_CONFIG=~/a TEMPER_DATA=~/b TEMPER_CACHE=/moved ~/bin/temper` is its shape.
- A word for the binary that would take the answer past 49,216 bytes gives way to `temper`. `the_widest_directories_fit_the_wire` holds.
- `sh -c` makes of the words what they say: with `HOME` set in the child, each value and the binary's word, a `~/` among them, reach the paths they name, spaces, apostrophes and all (`the_words_survive_a_shell`, grown).
- The pages draw `temper program-login NAME` for an empty answer, and otherwise the answer, a space and `program-login NAME`. A test in `temper-client-view/src/tests/connect_model.rs` holds `~/bin/temper program-login claude-pat`, on ConnectModel's and the welcome's.
- `On each host, run: temper program-login` (`Word::ProgramCommand`), the tools' placeholders (`temper-client-tools/src/limits.rs`), `Word::SpecTooLong`'s `temper ctl DIR` and `forge_sign_in`'s `forgejo` stay as they are: they name other machines, or are placeholders, or aren't temper.
- Remove the first **Not yet** line of `temper/docs/design/client/README.status.md`, the one naming *The words of a command for this machine*.

## Inputs

- `temper/docs/design/client/README.md`, 4.6, *The program's environment* and *The words of a command for this machine*; 3.5's mentions of the command.
- `temper/docs/plan/box-small/notes/contracts.md`, *The command's words*, its last section first if `program_login::line` is on your base.
- `temper/docs/plan/box-small/notes/common.md`.

## Owns and may change

- Owns the files in the header, and creates `printed.rs` and `printed/tests.rs`.
- `temper/crates/temper/src/lib.rs`: `mod printed;`, and the one call in `prepare_with_limits`, which passes the word. No other edit.
- `temper/crates/temper-client-view/src/pages/connect_model.rs`, `command` alone. `temper/crates/temper-client-view/src/accounts/words.rs`: one `Word` variant and row for `program-login `, beside `Command`.
- Fixtures whose words now need the binary's: `ENVIRONMENT` in `temper/crates/temper-client-view/src/tests/connect_model.rs`, and `the_owner_sees_the_command_the_box_answers`'s first argument in `temper/tests/client/tests/suite/connect_model.rs`, each gains ` temper` at its end, so what the page draws stays the same bytes. New tests in the first of those files.
- `temper/crates/temper-service/src/tests/family.rs` and `temper-client-protocol-wire/src/tests.rs` carry the answer as opaque bytes: leave them.
- `temper-forge-setup`'s `sign_in::quote` stays: `printed.rs` calls it for what needs quotes. If its shape doesn't fit "quotes only what follows `~/`", still call it, on the part after `~/`.
- If `program_login::line` or `printed_command` is on your base, do what `notes/contracts.md` says, and `temper/docs/design/client/README.md`, 4.6's one sentence it names is yours to change. Otherwise leave them to the catch-up.
- `first-usable-model` is on `lib.rs`'s call and `connect_model.rs`'s `command`: take by catch-up whatever has landed. `sign-in-new-tab` is elsewhere in those crates.

## Steps

The outer layer, in `temper/crates/temper/src/printed.rs`:

```rust
//! The words of a command for this machine (temper/docs/design/client/README.md, 4.6).

/// What a word is read from, given so that a test fakes the machine.
pub(crate) struct Machine<'a> {
    pub executable: Option<&'a [u8]>,
    pub path: Option<&'a [u8]>,
    pub home: Option<&'a [u8]>,
    /// The canonical path of an executable file; `None` for anything else.
    pub canonical: &'a dyn Fn(&[u8]) -> Option<Box<[u8]>>,
}
pub(crate) fn program_word(machine: &Machine<'_>) -> Box<[u8]>;
pub(crate) fn path_word(path: &[u8], home: Option<&[u8]>) -> Box<[u8]>;
/// `program_word` of this process: `current_exe`, and `PATH` and `HOME` from ENVIRONMENT.
pub(crate) fn running_word(environment: &[crate::directories::EnvironmentEntry]) -> Box<[u8]>;
```

`Directories::program_environment(&self, environment, working, word: &[u8])`, its `HOME` read from `environment`. The bytes that stay unquoted are ASCII letters, digits and `_-./+,:@%`. A value's `~` and `=` are quoted: `~` so no literal one expands, `=` so that no word reads as an assignment.

1. The rows above, failing first; then `printed.rs`.
2. `program_environment` and its tests, then the call in `lib.rs`.
3. `command` and its word; the fixtures; the new view tests.
4. Break by hand, one at a time, and see the rule's test fail: the first-on-`PATH` check (take any match), the `(deleted)` strip, the component check on `$HOME`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper -p temper-client-view --all-targets -- -D warnings`; `heavy cargo nextest run -p temper --lib`; `heavy cargo nextest run -p temper-client-view`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(the_owner_sees)'`. Citations. Commit and push.

## Report

As the implementer's text asks; and whether `program_login::line` was on your base, and what came of it.
