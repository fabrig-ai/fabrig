+++
id = "engine-forge"
part = "engine-shell"
kind = "migration"
needs = ["shell-crate"]
files = ["temper/crates/temper-engine-shell/**", "temper/crates/temper/src/forge.rs", "temper/crates/temper/src/forge_sign_in.rs", "temper/crates/temper/src/forge_sign_in/**", "temper/crates/temper/src/setup.rs", "temper/crates/temper/src/commands.rs", "temper/crates/temper/Cargo.toml", "temper/docs/design/protocol/README.status.md", "temper/docs/design/protocol/setup.status.md"]
effort = "high"
minutes = 120
+++

## Goal

The owned forge's commands, `forge fetch|prepare|run|sign-in|source|remove`
and `internal forge-child`, with the document checks they make, move out
of the box's `temper` library into a new crate, `temper-engine-shell`,
which the box's `temper` and `temper-engine` both link
(`temper/docs/design/protocol/README.md`, 4.1 and 4.2;
`temper/docs/design/protocol/setup.md`, 11). Their rows become constants
both tables hold, and their lines name the running binary. The box keeps
every command, word and test.

Acceptance:

- `temper/crates/temper-engine-shell`: ordinary Rust on the `binary` lint
  profile with the `ordinary` `clippy.toml` copy, depending on
  `temper-service` by the workspace's entry with no feature, and on
  `temper-shell`, `temper-forge-setup`, `temper-forge-setup-shell`,
  `temper-box`, `temper-fake` (the demo's check), `skein-shell` and what
  the moved code names besides; its crate doc cites
  `protocol/README.md`, 4.1. Its `forge` module is `notes/contracts.md`,
  6's: `run`, `configure`, `Command`, `command`, `FORGE` and the rows
  `FETCH`, `PREPARE`, `RUN`, `REMOVE`, `SOURCE`, `SIGN_IN`,
  `FORGE_CHILD`, and `reads_document`; `setup` (the deployment's checks,
  `setup.rs`'s `check`, `check_with`, `Deployment`) and the sign-in's
  module (`forge_sign_in.rs` and its directory) move with it, unchanged
  but for paths and the binary's name.
- **The binary's name:** every line, usage and hint these commands print
  takes `binary` where it says `temper` today (`forge.rs`'s
  `{binary} forge {verb}`, `run {binary} forge prepare …`, the usage
  line; the sign-in's printed Forgejo command and lines, where they name
  the binary that runs them). The box passes `"temper"`, so its words
  are unchanged; `temper-engine` will pass `"temper-engine"`.
- **One new row** in an existing table test of the moved forge code (find
  it with `rg 'fn .*' temper-engine-shell/src/**/tests*` once moved; the
  usage or the prepare hint's test if one exists, else a two-row table
  in `forge`'s tests): with `binary` `temper-engine`, the hint reads `run
  temper-engine forge prepare …`. Break the name once, see the row fail,
  restore.
- **The box links it:** `temper/src/commands.rs`'s `COMMANDS` holds the
  constants in place of its own rows of the same words (each row's text
  unchanged), and imports `FORGE` from the crate; `temper`'s `lib.rs`
  reaches the commands through `temper_engine_shell::forge::configure`
  and `command` in `configure_row`'s and `run`'s `forge` and `internal
  forge-child` arms, with `"temper"`; `temper::forge`, and `setup` where
  a reader needs it, stay reachable by `pub use` of the crate's, so that
  `src/tests/*`, `fake_backends`, `box_command` and
  `tests/suite/forge.rs` don't change. The reference page
  `temper/docs/help/reference/commands.md` doesn't change.
- Every test of `temper` passes unchanged; `forge_sign_in/tests.rs` moves
  with its code.
- The status entries close: in `temper/docs/design/protocol/README.status.md`
  the **Not yet** 4.1, 5 entry "`temper-engine-shell`, the forge's
  commands: …", and in `temper/docs/design/protocol/setup.status.md` the
  **Not yet** 11 entry "`temper-engine-shell` doesn't exist: …", each its
  lines alone. This departs from the implementer's text, which closes
  `docs/design/` to you: these entries are this task's.

## Inputs

- `temper/docs/design/protocol/README.md`, 4.1 (rows two tables share,
  shared crates), 4.2 (its lines name it) and section 5's
  `crates/temper-engine-shell` row.
- `temper/docs/design/protocol/setup.md`, 7.4 and 11.
- `temper/docs/plan/ui-engine-split/notes/contracts.md`, 4 and 6.
- `temper/docs/plan/ui-engine-split/notes/common.md`.

## Owns and may change

- Owns the files in `files`; in `temper/src/lib.rs`, the `mod` and `use`
  lines of `forge`, `forge_sign_in` and `setup`, the `forge` and
  `internal forge-child` arms of `configure_row` and `run`, and the
  helpers below.
- The root `Cargo.toml`: the member line
  `"temper/crates/temper-engine-shell",` right after
  `"temper/crates/temper-forge-setup-shell",`, and the dependency line
  `temper-engine-shell = { path = "temper/crates/temper-engine-shell" }`
  right after `temper-engine-domain-trials = { … }`; nothing else there.
  `Cargo.lock` as cargo writes it.
- Pre-approved:
  - A helper of `temper`'s `lib.rs` the moved code calls (`path`,
    `member`, `prefixed`, `unread`, `address`, `listeners_refusal`,
    `login`, `login_passwd`, `errno`): under ten lines, the crate takes
    a private copy; longer, it moves to the crate, `pub`, and `lib.rs`
    calls it there. `private_root` is `temper-shell`'s already.
  - If `temper-service` has the `worker` feature on your base (task
    `worker-feature`), `temper-engine-shell`'s manifest asks for none,
    and `heavy cargo check -p temper-engine-shell` must pass without it;
    if it doesn't yet, nothing to do.
  - If a test of `temper` reaches a moved private item by `crate::…`,
    make that item `pub` in the crate and re-export it, rather than
    moving the test.
- Overlaps: `tui-box-front` is changing `lib.rs`, `commands.rs` and
  `src/tests/commands.rs`: keep `commands.rs`'s other rows as they stand,
  replace only the seven forge rows and the `FORGE` const, and touch
  `lib.rs` only as above.
- Not yours: serve, its host, `bundle`, the runtimes and
  `prepare_with_limits` (task `engine-serve`), which keep calling
  `setup::check_with` through the re-export; `temper-forge-setup-shell`.

## Steps

1. The manifest, `python3 -I -B tools/lints/lints.py --set binary
   temper/crates/temper-engine-shell`, the `ordinary` `clippy.toml` copy.
2. The move, then the box's arms and rows; `heavy cargo check -p
   temper-engine-shell -p temper --tests` while errors last.
3. The binary's name and its row; see it fail first.
4. `heavy cargo nextest run -p temper-engine-shell`; `heavy cargo
   nextest run -p temper --lib -E 'test(/commands|box_kinds|reference|help/)'`;
   read the counts.
5. The status entries.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I
   -B tools/lints/lints.py`; `heavy cargo clippy -p temper-engine-shell -p
   temper --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-shell -p temper`; `heavy cargo check -p
   temper-real-loop-world -p temper-playground --tests`.
7. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit and push.

## Report

As the implementer's text says, with the helpers copied or moved and the
lines that now take the binary's name.

## Answers

### 2026-11-08: Resume engine-forge after shell-crate publishes its contracted shared directories and command types (recommended), or authorize taking over that dependency's unfinished implementation? (ui-engine-split/engine-forge-1, foreman ui-engine-split/engine-forge/foreman-1)

Resume engine-forge after shell-crate publishes its contracted shared directories and command types.
The task explicitly declares `needs = ["shell-crate"]`.
`notes/contracts.md` says: “A task builds its half to this note and never waits on another's half but where its spec's `needs` says so.”
Section 4 assigns shared `Directories`, `Row`, `Argument`, `DIRECTORIES` and command rendering to `shell-crate`.
Those files are outside engine-forge's authorized scope; taking over their implementation is not pre-approved.
The next run resumes against the completed shell-crate dependency and implements engine-forge using those shared types.
Acceptance remains unobserved; this does not accept engine-forge as done.
