+++
id = "bx-process"
part = "box-groups"
kind = "migration"
needs = ["bx-box", "bx-children", "bx-serve", "fr-process"]
files = [
  "temper/crates/temper/src/box_command.rs",
  "temper/crates/temper/src/box_command/**",
  "temper/crates/temper/src/forge.rs",
  "temper/crates/temper/src/forge/**",
  "temper/crates/temper/src/forge_sign_in.rs",
  "temper/crates/temper/src/forge_sign_in/**",
  "temper/crates/temper/src/fake.rs",
  "temper/crates/temper/src/fake/check.rs",
  "temper/crates/temper/src/fake/tests.rs",
  "temper/crates/temper-fake/src/steps.rs",
  "temper/crates/temper-fake/src/tests/steps.rs",
  "temper/crates/temper-fake/src/lib.rs",
  "temper/crates/temper-fake/src/tests.rs",
  "temper/crates/temper-forge-setup/src/step.rs",
  "temper/crates/temper-forge-setup/src/tests/steps.rs",
  "temper/crates/temper/tests/suite/forge.rs",
  "temper/crates/temper/tests/suite/box_groups.rs",
  "temper/crates/temper/tests/suite/main.rs",
  "temper/crates/temper/tests/process.rs",
  "temper/crates/temper/tests/support/**",
]
effort = "high"
minutes = 75
+++

## Goal

The box-groups part's last task, once the document names no forge
(`fr-process`, after `fr-init`): the bridges go, and the shipped
binaries are tested on the group files
(`temper/docs/design/draft/playground-reset.md`, 3 and 4;
`notes/contracts.md`, 3). The binary's `bridge_groups` and the child
commands' document fallback are deleted, as is the document-wide step
builder (`temper_fake::steps`, and `temper_forge_setup::steps` if no
caller is left); `temper forge remove` and a box starting its recorded
groups from their files are tested with the real binary.

Acceptance:

- `temper`'s `suite` binary, new module `box_groups`:
  - `forge_remove_takes_the_directory_and_keeps_the_cache`: a scratch
    deployment (`temper init` into it, as the suite's other tests make
    one), an owned group's file (`group::compose` of the owned stock
    definition's `setup` member, or the fixture the suite uses for an
    owned entry) naming `forge-N`, a populated `<data>/forge-N/` and a
    `<cache>/forgejo/VERSION/forgejo` file: `temper forge remove DIRS N`
    exits 0, the directory is gone, the cache file and the group file
    are left (the box removes the file).
  - `forge_remove_without_a_group_ends_at_once` (exit 0, nothing
    printed on stdout).
  - `a_child_command_without_a_group_file_says_so`: `temper forge
    prepare DIRS N` with no `groups/N` refuses with `forge N: the setup
    has no entry for it` (no document fallback now).
- `temper`'s `process` binary (the process tier, `process-temper`
  profile, not the gate):
  `the_box_starts_a_recorded_group_from_its_file`: before the first
  `temper box`, the test writes a fake forge's group file (not the
  playground's, its `listen` the document's `application.ports.forge`)
  and its `box/started/N`; once the box serves, the fake forge answers
  on that port, and after the box's stop a second `temper box` starts
  it again; then the test removes nothing itself, and the box's own
  listeners are unchanged (`application.ports` held as the suite's
  `box_listeners` checks them).
- Every test of `temper` (lib, `suite`), `temper-fake`,
  `temper-forge-setup` and `temper-box` passes; the `process` tier's
  tests that ran before this task still pass (run once with the
  `process-temper` profile; a failure your change cannot reach runs
  once alone, AGENTS.md 6).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md` (what you may
  change without asking).
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 3, 4.
- `temper/docs/design/draft/playground-reset.md`, 3, 4, 8 (the process
  tier).
- `temper/docs/design/protocol/setup.md`, 7.1, 7.5.
- `temper/docs/plan/playground-reset-r2/tasks/bx-box.md`,
  `bx-children.md` (what the bridges are and where).
- Code: `temper/crates/temper/src/box_command.rs` (`bridge_groups`,
  `machine_ports`, `startup_groups`), `forge.rs` (`group_entry`,
  `remove`), `fake_backends.rs` (`checked_peers`'s call of
  `temper_fake::steps`), `temper/crates/temper-fake/src/steps.rs`,
  `temper/crates/temper-forge-setup/src/step.rs`;
  `temper/crates/temper/tests/suite/` (its scratch deployments),
  `tests/support/boxed.rs` (the process tier's box harness),
  `tests/process.rs`.

## Owns and may change

- Owns the files above.
- `fake_backends.rs`'s `checked_peers` (owned by `forge-records`, then
  `playground`): its call of `temper_fake::steps` is replaced by the
  reserved ports it needs, from `crate::box_command::machine_ports`
  (the box's held ports) and serve's listeners; that call site only. If
  a `pg-` task changed it already, follow what it did.
- A test of `temper` (lib or `suite`) or of `temper-fake` that still
  gave a setup entry through the document for a child command now
  writes its group file (`group::compose`, 0600 in 0700); a test that
  only checked the document-wide step builder goes with it (each
  listed in the report, with the property it checked and the test that
  covers it now, or why it no longer holds).
- Not yours: `setup.rs`, `lib.rs`, `init.rs` of the binary,
  `temper-service`, `temper-box`'s sources (a box change found here is
  a question).

## Steps

- Delete `box_command.rs`'s `bridge_groups` and its call, and its
  tests; `machine_ports` stays as `forge-records` left it (from
  `application.ports`).
- `forge::group_entry` loses its `checked` fallback: the group file or
  `Ok(None)`; its callers keep their refusal words. Delete the fallback
  test; keep the others.
- Delete `temper-fake/src/steps.rs` and its tests and its `pub use`
  in `temper-fake/src/lib.rs`, once `checked_peers` no longer calls it.
  Then delete `temper_forge_setup::steps(setup, ...)` and the private
  helpers only it used if nothing else calls it (`rg
  'temper_forge_setup::steps|step::steps'`), with its tests in
  `src/tests/steps.rs` that test only it; `step::serve` and `Group`,
  `Steps`, `Stage`, `Kind`, `Step` stay (the box names them).
- New suite module `tests/suite/box_groups.rs` (and its `mod` line in
  `tests/suite/main.rs`), using the suite's own scratch and command
  helpers in `tests/support/`.
- The process test, in `tests/process.rs` beside the other box stories
  (it uses `support/boxed.rs`'s harness), named as in the Goal.

Then: `cargo fmt`; `heavy cargo clippy -p temper -p temper-fake -p
temper-forge-setup --all-targets -- -D warnings`; narrow: `heavy cargo
nextest run -p temper --test suite -E 'test(/box_groups::/)'` (check
the count); the process test alone: `heavy cargo nextest run
--profile process-temper -p temper --test process -E
'test(=the_box_starts_a_recorded_group_from_its_file)'`; then the
focused tests: `heavy cargo nextest run -p temper -p temper-fake -p
temper-forge-setup -p temper-box -p temper-service`; and the process
tier once: `heavy cargo nextest run --profile process-temper -p
temper`. Old forms from `temper/`: `python3 -I -B
scripts/old_forms.py`, lowering with `--write` what the deleted
bridges and builders lower. Commit and push your branch.

## Report

- Each deleted function and test, and the test covering what it
  covered.
- The process tier's result, with the failures that ran alone and
  their logs.
- Whether a group removed end to end (`RemoveForge` to the box's
  `removed N`, the forge's directory gone, the cache kept) is reachable
  in the process tier yet; that story is `rm-process`'s (D4).
