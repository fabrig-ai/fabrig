+++
id = "machine-names"
part = "machine-names"
kind = "feature"
needs = []
files = ["smith/crates/smith-protocol-machine/src/environment.rs", "smith/crates/smith-protocol-machine/src/boundary.rs", "smith/crates/smith-protocol-machine/src/tests.rs", "smith/crates/smith-protocol-machine/src/tests/debug.rs", "smith/docs/design/domain/tools.status.md"]
effort = "medium"
minutes = 20
+++

# smith-protocol-machine: a variable prints its name alone

## Goal

`Variable`, `Environment` and `Spawn` print each variable's value. They print names only, as `Spawn` in `skein/crates/skein-io/src/kernel.rs` does, and the component that holds the environment prints through them.

Acceptance:

- `Variable` prints `Variable { name: [..] }`.
- `Environment` prints `Environment { names: [[..], ..], bytes: N }`.
- `Spawn` prints `Spawn { program: .., args: .., env: [[..], ..], dir: .., pipes: .. }`, its `env` holding names.
- An entry with no `=` prints whole, as skein-io's does.
- The tests below pass and fail with the derives put back.
- `heavy cargo nextest run -p smith-protocol-machine -p smith-machine-world` passes.

## Inputs

- `temper/docs/plan/debug-secrets/notes/common.md`
- `smith/docs/design/domain/tools.md`, 5 (A variable prints its name alone)
- `skein/crates/skein-io/src/kernel.rs`, `Spawn` and `EnvNames`: the form to follow

## Owns and may change

- **Owns:** the files in `files`. `src/tests/debug.rs` is new, with its `mod debug;` line in `src/tests.rs`.
- **Status entry:** in `smith/docs/design/domain/tools.status.md`, remove the `5: smith-protocol-machine's ...` sentence and keep the rest of that bullet. `.config/gate/read-by-tests.txt` covers it by a glob, but no test reads the status file: `smith-bench` reads `tools.md`'s sections as task guards, and this plan moves none of them. You don't run `smith-bench`; the trunk's end check does.
- **`EnvNames`:** skein-io's and skein-sim's are private. Write a private one in `environment.rs`, `pub(crate)` for `boundary.rs`, and don't change skein. Say in your report that it is a third copy.
- **Stays as it is:** every other derive, and everything that reads these fields.

## Steps

1. Write the `Debug` of `Variable` and `Environment` in `environment.rs`, and of `Spawn` in `boundary.rs`, each in place of the derive's `Debug`.
2. In `src/tests/debug.rs`, with a made-up canary as a value:
   - `debug_of_a_variable_shows_its_name_never_its_value`;
   - `debug_of_an_environment_shows_names_never_values`, from `Environment::encode`, and the same for a `Component` holding it;
   - `debug_of_a_spawn_shows_env_names_never_values`, with one `NAME=canary` entry and one with no `=`.
   Each asserts the exact form where it has one, and both renderings as `notes/common.md` says.
3. Narrow tests, then `cargo fmt`, `old_forms.py` and clippy on `smith-protocol-machine`, as the implementer's text has them. Commit and push.

## Report

As the implementer's text has it, and name each rule you broke by hand with the failure you saw.
