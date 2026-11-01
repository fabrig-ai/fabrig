+++
id = "bx-steps"
part = "fs"
kind = "interface"
needs = []
files = ["temper/crates/temper-forge-setup/src/setup.rs", "temper/crates/temper-forge-setup/src/step.rs", "temper/crates/temper-forge-setup/src/tests/**", "temper/crates/temper-fake/src/steps.rs", "temper/crates/temper-fake/src/tests/steps.rs"]
effort = "high"
minutes = 75
+++

## Goal

One forge's setup entry read on its own, and one group's steps built from one entry, so
that the box's shell can load `<data>/box/groups/N` and serve can check and write a forge
record's setup entry (notes contracts T3). The list readers and the whole-`Setup` step
builders keep their behaviour, re-expressed on the new pieces; no reader moves in this task.

Acceptance:
- `temper_forge_setup::setup` gains the one-entry readers and the group file's writer below;
  each gives the `Entry` that `Setup::read` gives for the same entry in a list (paths
  resolved alike), with `asked: true`.
- `temper_forge_setup::step::owned_group` and `temper_fake::steps::playground_group` build
  one group each, exactly the steps, stages, names, kinds, listeners and readiness of
  today's owned asked group and today's `Playground` group, labelled as today, `recorded:
  true`, `before: 0`; `temper_fake::steps::group` picks one by the entry's mode.
- `temper_forge_setup::steps` and `temper_fake::steps` give exactly what they give today
  (their existing tests pass unchanged): `steps` builds its owned groups with
  `owned_group`, and the demo's group with `playground_group`, then sets that group's
  `recorded` to `false` and its `before` as today.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.1 and 7.3 (the steps and groups), 8 (the setup
  entry's members), 8.4 (the demo's group).
- Draft `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T1 (a record's `setup`), T3
  (the setup entry without `forge`, `directory` and `start`; `box/groups/N`; the steps by
  mode), T4 (`application.machine.owned` and `application.demo.forge` are entry bodies).

## Owns and may change

- Owns: `temper/crates/temper-forge-setup/src/setup.rs`, `src/step.rs` (and `src/lib.rs`
  for a re-export), their tests under `src/tests/`; `temper/crates/temper-fake/src/steps.rs`
  and `src/tests/steps.rs`.
- No caller outside these two crates changes: `temper_forge_setup::steps`,
  `temper_fake::steps`, `Setup::read` and `Setup::entry` keep their signatures and output.
- Options, pre-approved:
  - Sharing the list's `entry` reader is inside; default: one private reader taking which
    shape it reads (a list item; a group file, which requires `forge` and `directory` and
    refuses `start`; a body, which refuses `forge`, `directory` and `start` as unknown
    members and takes `forge` from its caller), so the three cannot drift.
  - If `asked: true` for a fake or managed body would break an existing test of these
    crates, keep the field as the list sets it for the list and `true` for the new readers,
    and say so.

## Steps

1. **One-entry readers** (`src/setup.rs`):

   ```rust
   /// A setup entry's body at most, as a forge record holds it (notes contracts T3).
   pub const BODY_BYTES: u32 = 4096;
   /// A group's settings file at most: a body with its forge and directory.
   pub const GROUP_BYTES: u32 = BODY_BYTES + 64;
   impl Entry {
       /// `<data>/box/groups/N`'s text: one `application.setup[]` entry (protocol/setup.md, 8)
       /// with `forge` and `directory`, paths resolved as `Setup::read` resolves them;
       /// problems are named from the file's root.
       pub fn group(bytes: &[u8], limits: &Limits, directories: &config::Directories) -> Result<Entry, Problems>;
       /// An entry's body, `{"bot", "mode"}`, at `at` in `document`, problems named under
       /// `path` (`application.machine.owned`, `application.demo.forge`); its forge is
       /// `forge` and its directory `forge-N` under the data directory (none for a
       /// `"managed"` body, as the list's entry has none).
       pub fn body(d: &Document, at: u32, path: &Path, forge: u16, limits: &Limits,
                   directories: &config::Directories, problems: &mut Problems) -> Option<Entry>;
       /// The same, from a record's setup text (at most `BODY_BYTES`).
       pub fn body_text(bytes: &[u8], forge: u16, limits: &Limits,
                        directories: &config::Directories) -> Result<Entry, Problems>;
   }
   /// Group N's settings text from a body's text: `{"forge":N,"directory":"forge-N",`
   /// followed by the body's members; `None` when the body is not one JSON object with at
   /// least one member, or past `BODY_BYTES`.
   pub fn group_settings(body: &[u8], forge: u16) -> Option<Box<[u8]>>;
   /// `forge-N`, the forge's directory under the data directory, for every mode.
   pub fn directory(forge: u16) -> Box<[u8]>;
   ```

   `Limits` is `skein_forge_setup::limits::Limits`, `Problems` the crate's
   `problem::Problems`, `Document`/`Path` as `Setup::read` uses them. The text readers parse
   with `jig_protocol_config::document` (bytes bound `GROUP_BYTES` or `BODY_BYTES`, depth
   32), a parse failure being one problem at the root.
2. **Owned group** (`src/step.rs`):

   ```rust
   /// An owned forge's group (notes contracts T3; protocol/setup.md, 7.3): `forge fetch`
   /// when either binary is fetched, `forge prepare`, `forge run N forgejo` (ready at its
   /// listener, given it when `forgejo_listens`), `forge run N runner` with a runner;
   /// labelled `Forge N (Forgejo)`, recorded, `before` 0.
   pub fn owned_group(forge: u16, owned: &Owned, flags: &[Box<[u8]>], forgejo_listens: bool) -> Group;
   ```

   `steps` uses it for each owned entry: asked, the group with `before` set as today;
   not asked, its steps flattened into `always` as today.
3. **Playground group and the pick** (`temper/crates/temper-fake/src/steps.rs`):

   ```rust
   /// The playground's group (protocol/setup.md, 8.4): `fake llm` at `llm`, `forge prepare N`,
   /// `fake forge N` at the fake's listener; labelled `Playground`, recorded, `before` 0.
   pub fn playground_group(forge: u16, fake: &Fake, llm: SocketAddr, flags: &[Box<[u8]>]) -> Group;
   /// One group by its entry's mode (notes contracts T3): owned, `owned_group`; fake,
   /// `playground_group` at `llm` (`None` without one); managed, `None`.
   pub fn group(entry: &Entry, llm: Option<SocketAddr>, flags: &[Box<[u8]>], forgejo_listens: bool) -> Option<Group>;
   ```

   `steps` builds the demo's group with `playground_group`, then sets `recorded: false` and
   `before: first_len`, as today.
4. **Tests.** temper-forge-setup (`src/tests/read.rs`, `src/tests/steps.rs`):
   - `a_group_file_reads_as_its_list_entry_reads` (fixture TWO's owned entry, and three()'s
     fake: `Entry::group` of each entry's text equals `Setup::read`'s entry but for
     `asked: true`, under `Directories::given(.., b"/data", b"/cache")`).
   - `a_body_takes_its_forge_and_directory_from_its_caller` (`body_text` of the owned body
     with forge 7: `forge == 7`, directory `/data/forge-7`, the fetched Forgejo resolved
     under the cache).
   - `a_body_naming_forge_directory_or_start_is_refused` (each one problem, naming the
     member).
   - `a_group_file_without_forge_or_directory_is_refused`.
   - `group_settings_adds_forge_and_directory_and_reads_back` (round trip through
     `Entry::group`; `None` for `{}`, `[]`, text past `BODY_BYTES`).
   - `owned_group_has_the_asked_groups_steps` (equal to the group `steps` gives for an
     asked owned entry but for `before`; without a runner, no runner step; fetched gives the
     fetch step).
   temper-fake (`src/tests/steps.rs`):
   - `playground_group_has_the_demos_steps_and_is_recorded` (equal to the demo group of
     `steps` but for `recorded` and `before`).
   - `group_picks_by_mode` (owned, fake with and without `llm`, managed).
5. Narrow runs, one at a time:
   `heavy cargo nextest run -p temper-forge-setup --lib -E 'test(=tests::read::a_group_file_reads_as_its_list_entry_reads)'`
   and each new test likewise; `heavy cargo nextest run -p temper-fake --lib -E 'test(=tests::steps::group_picks_by_mode)'`.
   Then both packages whole (`heavy cargo nextest run -p temper-forge-setup`, then
   `-p temper-fake`); `heavy cargo check -p temper` (the binary calls both `steps`);
   `cargo fmt`; `heavy cargo clippy -p temper-forge-setup -p temper-fake --all-targets`. Old
   forms never raised (`python3 -I -B scripts/old_forms.py` in `temper/`). Commit and push.

## Report

The exact names and signatures you landed, if any differ from the sketch: sv's startup and
serve tasks and bx-compose call them.
