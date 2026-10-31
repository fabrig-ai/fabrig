+++
id = "bx-children"
part = "box-groups"
kind = "feature"
needs = ["bx-steps"]
files = [
  "temper/crates/temper/src/forge.rs",
  "temper/crates/temper/src/forge/**",
  "temper/crates/temper/src/forge_sign_in.rs",
  "temper/crates/temper/src/forge_sign_in/**",
  "temper/crates/temper/src/fake.rs",
  "temper/crates/temper/src/fake/check.rs",
  "temper/crates/temper/src/fake/check/**",
  "temper/crates/temper/src/fake/tests.rs",
]
effort = "high"
minutes = 60
+++

## Goal

The commands the box runs for a group read forge N's setup entry from
`<data>/box/groups/N` (`notes/contracts.md`, 3, "The child commands"),
and a new once command, `temper forge remove DIRS N`, removes forge N's
directory, keeping the cache (`temper/docs/design/draft/playground-reset.md`,
4, "Its box group stopped and removed"). While a deployment's document
still names `application.setup` (until `forge-records` drops it), a
command with no group file falls back to the document's entry, so
today's tests and deployments go on unchanged; `bx-process` removes
the fallback.

Acceptance, `temper`'s lib target (unit tests, real files in a scratch
directory):

- `forge::tests::group_entry_reads_the_group_file_before_the_document`:
  with a `groups/N` file (written with `temper_forge_setup::group::compose`,
  0600, in a 0700 `box/groups`) and a document whose `application.setup`
  names forge N differently, `group_entry` gives the file's entry.
- `forge::tests::group_entry_falls_back_to_the_document_without_a_file`.
- `forge::tests::group_entry_refuses_a_group_file_others_can_read`
  (0644: refused as `read_private_file` refuses), and an invalid one
  (the refusal line `group::read` gives).
- `forge::tests::remove_takes_the_forge_directory_and_keeps_the_cache`:
  `<data>/forge-3/` with nested files and directories and a
  `<cache>/forgejo/VERSION/forgejo` file: after `remove` the directory
  is gone, the cache file and `<data>/box/groups/3` are still there
  (the box removes the group's file, not the command).
- `forge::tests::remove_ends_at_once_without_a_file_or_a_directory`
  (exit 0 both ways: a kept `remove` sent again ends).
- `forge::tests::remove_refuses_a_directory_not_strictly_beneath_data`:
  an entry whose `directory` resolves to `<data>` itself, to a path
  outside `<data>`, to an ancestor of `<cache>` or `<config>`, or to
  `<data>/box` or beneath it: refused, nothing removed.
- `forge::tests::remove_refuses_a_link`: a `directory` that is a
  symbolic link: refused, the link and its target kept.
- `fake::tests::the_fake_forge_reads_its_entry_from_the_group_file`
  (the `Source::Setup` branch of `forge_document`).
- Every existing test of `temper`'s lib target and of its `suite`
  binary passes unchanged (they run the commands with a document and no
  group file: the fallback).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md` (what you may
  change without asking).
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 3.
- `temper/docs/design/draft/playground-reset.md`, 3 ("The box knows
  kinds, not forges"), 4 ("Its box group stopped and removed").
- `temper/docs/design/protocol/setup.md`, 6.6 (the cache), 7.1 (the
  commands), 7.3.
- Code: `temper/crates/temper-forge-setup/src/group.rs` (`bx-steps`:
  `GroupFile`, `Machine`, `read`, `compose`, `path`, `FILE_BYTES`);
  `temper/crates/temper/src/forge.rs` (`start`, its verbs and usage
  line), `forge_sign_in.rs` (`run`), `fake.rs` (`forge_document`'s
  `Source::Setup` branch), `fake/check.rs` (`configuration`),
  `directories.rs` (`Directories`, `flags`, `jig`), and the existing
  tests in `src/forge/tests.rs` and `src/fake/tests.rs` for their
  scratch-directory style.

## Owns and may change

- Owns the files above. `temper forge remove` is a new verb of
  `forge.rs`'s own dispatch; its usage line gains `temper forge remove
  [DIRECTORIES] FORGE`.
- May change, as `notes/scope.md` says, whatever the change forces
  elsewhere; in particular the `temper` CLI help test that pins the
  `forge` usage line, if one does, only for the new verb (say where).
  A help page under `temper/docs/help/` that lists the forge verbs may
  gain the verb's line only if a test compiles it and fails without it
  (a departure: say so).
- Not yours: `setup.rs`, `lib.rs`, `box_command.rs` (`bx-box`),
  `fake_backends*`, `temper/crates/temper/tests/`. The fake LLM's
  members (`llm_configuration`'s `Source::Demo`) stay read from the
  document's `Deployment` as today: `forge-records` renames them there.
  `reconcile_settings` (a managed forge's legacy reconcile) stays on
  the document.

## Steps

In `forge.rs`, its doc gaining a line citing
`temper/docs/design/draft/playground-reset.md, 3`:

```rust
/// Forge N's setup entry: `<data>/box/groups/N` when it exists, else the
/// document's `application.setup` entry while the document has one (the
/// fallback `bx-process` removes). `checked` is the document's, if read.
pub(crate) fn group_entry(
    directories: &super::Directories,
    number: u16,
    checked: Option<&temper_forge_setup::check::Checked>,
) -> Result<Option<temper_forge_setup::setup::Entry>, String>;

/// `temper forge remove DIRS N`: removes the group's forge directory.
fn remove(directories: &super::Directories, number: u16) -> Result<u8, String>;
```

- `group_entry` reads the file with `skein_shell::read_private_file`
  (bound `group::FILE_BYTES`, the effective user), NotFound meaning no
  file; it checks it with `group::read` and a `group::Machine` made of
  `directories.flags()`, `directories.jig()`, `checked.limits` when
  given (else `temper_forge_setup::limits::small()`), `llm: None`,
  `forgejo_listens: cfg!(target_os = "linux")`. Without a file it
  returns `checked.setup.entry(number).cloned()`. `Ok(None)` is "the
  setup has no entry for it", as the callers say today.
- Callers: every `checked.setup.entry(number)` of `forge.rs`'s `start`,
  `forge_sign_in.rs`'s `run`, `fake.rs`'s `forge_document`
  (`Source::Setup`) and `fake/check.rs`'s `configuration` becomes
  `group_entry(directories, number, Some(&checked))`, keeping each
  caller's refusal words. They still read and check the document as
  today for what else they read from it.
- `remove`: matched in `start` before the document is read (it reads
  no document): reads the group file as `group_entry` does, without
  the fallback; no file: exit 0. With one, the entry's `directory`
  (resolved by `group::read`; empty for none: exit 0): refused unless
  it is strictly beneath `directories.data` component by component,
  is not `<data>/box` nor beneath it, and neither it nor any of its
  ancestors up to `<data>` is `directories.cache` or
  `directories.config` or a symbolic link (`symlink_metadata`); a
  missing directory is exit 0; otherwise `std::fs::remove_dir_all`.
  Refusals print as `temper forge remove: forge N: WHY` as the other
  verbs' do. The command never touches `box/groups/N` nor
  `box/started/N` (the box removes them after it, `bx-box`), nor the
  cache.
- Tests in `src/forge/tests.rs` and `src/fake/tests.rs`, in the style
  of their scratch directories (`std::env::temp_dir()` plus the process
  id, or `skein-scratch`, a dev-dependency already), each test its own
  directory, removed at its end.

Then: `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D
warnings`; narrow: `heavy cargo nextest run -p temper --lib -E
'test(/forge::tests::|fake::tests::/)'` (check the count); then the
focused tests of the package: `heavy cargo nextest run -p temper` (the
`suite` binary runs the child commands through the fallback). Old
forms from `temper/`: `python3 -I -B scripts/old_forms.py`. Commit and
push your branch.

## Report

- Each caller changed, and any refusal whose words changed.
- How `remove` tests "strictly beneath" and the links, and what it does
  with a directory it cannot fully remove (a permission error part way).
- Any suite test that reaches the new verb or `group_entry` in a way
  the unit tests do not.
