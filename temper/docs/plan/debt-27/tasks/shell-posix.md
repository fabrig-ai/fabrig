+++
id = "shell-posix"
part = "shell-posix"
kind = "feature"
needs = []
files = ["skein/crates/skein-shell/src/posix.rs", "skein/crates/skein-shell/src/posix/**", "skein/crates/skein-shell/src/lib.rs", "skein/crates/skein-shell/src/linux.rs", "skein/crates/skein-shell/src/linux/**", "skein/crates/skein-shell/src/darwin.rs", "skein/crates/skein-shell/src/darwin/**", "skein/docs/design/shell.status.md", "skein/docs/development/debt/D042-shell-backend-copies.md"]
effort = "high"
minutes = 60
+++

# skein-shell's POSIX parts live once

## Goal

Pay skein's D042. What `linux/` and `darwin/` write alike moves to a `posix` module, built on both, and each system re-exports it, so `crate::system::...` stays one face. `skein/docs/design/shell.md`, 7.5, already says so: read it, since the cut changed it for this.

Acceptance:

- `linux/terminal.rs` and `darwin/terminal.rs` are gone, for one `posix/terminal.rs`.
- This finds nothing: `rg -n '^\s*(pub(\(\w+\))? )?(unsafe )?fn (signal_group|group_live|process_live|retry|status|shutdown|wait_readable|cstring|action|block_sigpipe|map_error|usage|cpu|signal_number|private_error|kernel_error|read_opened|kind_of)\b' skein/crates/skein-shell/src/linux.rs skein/crates/skein-shell/src/linux skein/crates/skein-shell/src/darwin.rs skein/crates/skein-shell/src/darwin`. A function you kept by an option below is named in your report.
- `linux/group.rs` and `darwin/group.rs` hold `child_id` and `group_has_other_members` alone, with `darwin/group.rs`'s `proc_listpids`.
- `shell.status.md`'s "7.5: `posix`: `linux` and `darwin` each still write what it is to hold." is gone.
- `skein/docs/development/debt/D042-shell-backend-copies.md` is deleted.
- Net lines fall by at least 200.
- No test's assertion changes. The four group tests move with their functions.

## Inputs

- `skein/docs/development/debt/D042-shell-backend-copies.md`, whole.
- `skein/docs/design/shell.md`, 1, and 7.5: "The crate's layout", "One face, two systems" and "Each system's wrappers are its own".
- `temper/docs/plan/debt-27/notes/common.md`.

## Owns and may change

Owns the files in the header. Public names don't change: `lib.rs` gains `#[cfg(unix)] mod posix;`, and `pub use system::wait_readable` may stay as it is. No reader outside the crate names a path inside it.

**The cut's scan, function by function.** These are byte-the-same today:

| Moves to | From `linux*` and `darwin*` |
|---|---|
| `posix/terminal.rs` | all of `terminal.rs`; the first comment says it's both systems' |
| `posix/group.rs` | `signal_group`, `group_live`, `process_live`, and the tests `only_specific_positive_groups_can_be_addressed`, `the_running_process_is_live`, `a_reaped_child_is_not_live`, `only_specific_positive_processes_can_be_probed` |
| `posix.rs` | `retry`, `status`, `shutdown`, `wait_readable` |
| `posix/process.rs` | `cstring`, `action`, `block_sigpipe`, `map_error`, `usage`, `cpu`, `signal_number` |
| `posix/files.rs` | `private_error`, `kernel_error`, and two that differ only in how a path is spelt: `read_opened` (`mem::zeroed`), `kind_of` (`pub(crate)`) |

**The rest differ, and stay:** `check_private` and `make_directory_root` in a type or a step (`mode_t`'s width, `F_FULLFSYNC`), `read_file`, `read_private_file` and `make_root` in how each closes, `prepare`, `spawn`, `start_binary`, `resources`, `getsockopt`, `connect`, `bind`, `nonblocking`, `count`. List them in your report as what's left.

Each module with `unsafe` takes the crate's form: `#![expect(unsafe_code, reason = "...")]` and a `# The unsafe, and why it is sound` doc, its text moved from what it replaces. `last_errno` differs by system, so `posix` calls `crate::system::last_errno()`. `linux.rs` and `darwin.rs` re-export: `pub(crate) use crate::posix::{terminal, ...}`, so that their callers' paths stay.

Options, each pre-approved:

- A function in the table that turns out to call something each system spells differently, and needs more than a path's change: leave it in both, and name it in your report.
- If two moved files read better as one, or one reads better inline, the file names inside `posix` are yours. `posix` and `posix/terminal.rs` aren't.

## Steps

`git mv` where a file moves whole, so that history follows. Tests move, so no hand break.

**macOS is built by no runner here.** Run `rustup target list --installed`. If `aarch64-apple-darwin` is there, `heavy cargo check --target aarch64-apple-darwin -p skein-shell`. If it isn't, don't install it: read each `darwin*` edit by eye against what it re-exports, and report under `## Left` that the macOS build wasn't compiled here, so `.github/workflows/macos.yml`'s next run is its first. Keep `darwin/` edits to deletions and `use` lines, so that this reading stays small.

`heavy cargo nextest run -p skein-shell`; `heavy cargo check --target wasm32-unknown-unknown -p skein-shell`; the worlds that run the shell for real: `heavy cargo nextest run` on `skein/tests/ring` and `skein/tests/io`, by the names in their manifests. No meaning changes, so no further dependents.

`cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`, `--write` if a count falls; `heavy cargo clippy -p skein-shell --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says: the search's reading, the functions left in both and why, the net lines, and whether macOS was compiled.
