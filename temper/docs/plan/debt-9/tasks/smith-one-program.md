+++
id = "smith-one-program"
part = "smith-one-program"
kind = "feature"
needs = []
files = ["smith/crates/smith-agent-shell/src/config.rs", "smith/crates/smith-agent-shell/src/config/programs.rs", "smith/crates/smith-agent-shell/src/config/tests/programs.rs", "smith/crates/smith-agent-shell/src/agent_shell.rs", "smith/crates/smith-local-shell/src/fixture.rs", "smith/tests/agent-process/src/lib.rs", "smith/tests/agent-process/src/program.rs", "smith/tests/local-process/src/process.rs", "jig/tests/channel/src/program_runs.rs", "jig/testing/jig-test/src/lib.rs", "temper/crates/temper-playground/src/programs.rs", "temper/docs/design/domain/worker.md", "smith/docs/development/debt/D008-one-program-map.md", "smith/docs/development/debt/README.md"]
effort = "medium"
minutes = 50
+++

# smith: pay D008, the configuration holds its one program

## Goal

Pay `smith/docs/development/debt/D008-one-program-map.md`. `Configuration` and `Decoded` hold `Option<Program>`
and the set of endpoints it serves, while `ProgramName` has one variant. The configuration's document is
unchanged: its `programs` table and each endpoint's `program` are read as they are, with the same refusals.

Acceptance:

- `git grep -c 'zero_sized_map_values' -- '*.rs' | awk -F: '{s+=$2} END {print s}'` prints nothing or 0.
- `rg -n 'endpoint_programs' smith jig temper -g '*.rs' -g '*.md' -g '!**/docs/plan/**'` finds nothing.
- `rg -n 'BTreeMap<ProgramName' smith -g '*.rs'` finds nothing; `ProgramName` itself stays (the local shell's
  credentials name it).
- The item's file and its README row are gone.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `smith/docs/design/protocol/programs.md`, 3 and 4.2; `smith/docs/design/protocol/agent.md`, 4.
- `temper/docs/design/domain/worker.md`, 6 and `jig/docs/design/domain/hosts.md`, 8.1 (the hello's program
  endpoints).
- `smith/docs/development/debt/D008-one-program-map.md`.

## Owns and may change

- Owns: `smith-agent-shell`'s `config.rs`, `config/programs.rs`, `config/tests/programs.rs`, and the two reads
  in `agent_shell.rs`.
- The break of this public type is the item's own, so it is approved. Its readers, each only for the fields'
  names and types, removing each `#[expect(clippy::zero_sized_map_values, ...)]`:
  `smith-local-shell/src/fixture.rs`, `smith/tests/agent-process/src/{lib,program}.rs`,
  `smith/tests/local-process/src/process.rs`, `jig/tests/channel/src/program_runs.rs`,
  `jig/testing/jig-test/src/lib.rs`, `temper-playground/src/programs.rs`.
- `temper/docs/design/domain/worker.md`, 6: the one phrase "its `endpoint_programs` keys" becomes
  "its `program_endpoints`". This departs from the implementer's text for that line alone.
- If `config/tests/programs.rs` reads the map's length or keys, it moves to the new shape, each assertion
  saying the same fact.

## Steps

1. The fields, in `Configuration` and alike in `Decoded`:

   ```rust
   /// The program the configuration declares (smith/docs/design/protocol/agent.md, section 4).
   pub program: Option<Program>,
   /// The endpoints the program serves; only endpoints naming a program are present.
   pub program_endpoints: std::collections::BTreeSet<u32>,
   ```

2. In `config/programs.rs`, `name` stays and still refuses an unknown program:

   ```rust
   pub(super) fn build(documents: BTreeMap<String, ProgramDocument>) -> Result<Option<Program>, ConfigurationRefusal>;
   pub(super) fn endpoints(endpoints: &[Endpoint], program: Option<&Program>) -> Result<BTreeSet<u32>, ConfigurationRefusal>;
   pub(super) fn backend(program: Option<&Program>, endpoints: &BTreeSet<u32>) -> Option<smith_agent_service::ProgramBackend>;
   pub(super) fn domain(endpoints: &BTreeSet<u32>, limits: &mut smith_domain::Limits) -> Result<Box<[smith_domain::Program]>, ConfigurationRefusal>;
   ```

   An endpoint that names a program with none declared keeps its `Refusal::Field`. `backend`'s two `cfg`s both
   change, and the crate is built for Unix here: read the non-Unix copy by eye.
3. Readers: literals become `program: None, program_endpoints: BTreeSet::new()`; `jig-test`'s two reads become
   `configuration.program.as_ref()` and `configuration.program_endpoints.iter().copied()`.
4. `cargo fmt`; for each tree `old_forms.py`; `heavy cargo clippy -p smith-agent-shell -p smith-local-shell
   -p smith-agent-process-world -p smith-local-process-world -p jig-test -p temper-playground --all-targets --
   -D warnings` (take the worlds' package names from their `Cargo.toml`, and name `jig/tests/channel`'s);
   `heavy cargo nextest run -p smith-agent-shell -p smith-local-shell`; `heavy cargo check --tests` for each
   other reader.
5. Delete the item and its row; the citations check. Commit and push.

## Report

The common note's items. The tests only move, so no break is owed. Give the measure's count before and after.
