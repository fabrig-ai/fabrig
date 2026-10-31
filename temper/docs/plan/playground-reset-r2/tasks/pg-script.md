+++
id = "pg-script"
part = "playground"
kind = "feature"
needs = ["pg-start", "pg-gone", "bx-process"]
files = [
  "temper/crates/temper-fake/src/demo.rs",
  "temper/crates/temper-fake/src/tests/demo.rs",
]
effort = "medium"
minutes = 45
+++

## Goal

The scripted model's delegation names the playground's repository by
its resource tag, `forge << 32 | repository` (the init template's
`"resource": 4294967297`, forge 1, repository 1; see
`temper-init/src/tests/documents.rs` near line 249). Forge numbers are
never reused (contracts.md, 2), so after a reset, or after an unfinished
forge started over, the next playground's fake forge is not forge 1 and
the script would delegate into nothing. The fake LLM of the playground's
group N serves the script with every resource tag whose forge half is
the template's (1) rewritten to N.

Acceptance:

- `temper-fake`'s `src/tests/demo.rs`:
  `the_script_names_the_groups_forge` (the template's script read for
  forge 3 serves `"resource": 3 << 32 | 1` in the delegate call;
  forge 1 serves it unchanged) and
  `a_resource_of_another_forge_is_left_as_written`.
- The binary's fake command tests (`temper/src/fake/tests.rs`) pass,
  with one new case: `fake llm` for the playground's group N serves N's
  tag.
- `temper-fake`, `temper` (focused) and `temper-forge-setup` tests pass.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2
  (numbers) and 3 (the group file; the fake's command lines).
- `temper/docs/design/draft/playground-reset.md`, 6.1 and 6.2.
- The code: `temper-fake/src/demo.rs` (`read`, `models`), the binary's
  `src/fake.rs` (`llm_configuration`, `Source::Demo` or its successor),
  `temper-forge-setup`'s step builder for a playground group (bx-steps:
  `fake llm DIRS`), `temper_forge_setup::group::read`.

## Owns and may change

Owns the files in the header. May change, call sites only: the
binary's `src/fake.rs` and `src/fake/` (the `fake llm` command's
arguments and the forge number passed to `temper-fake`), and
`temper-forge-setup`'s playground step builder and its tests (the
command line). `notes/scope.md` applies.

Pre-approved, in this order:

- A (default): the playground group's step becomes `fake llm DIRS N`
  (N its group's forge), the command passes N to the reader.
- B, if A would change a box test or the step's identity beyond its
  command line: `fake llm DIRS` reads `<data>/box/groups/`, takes the
  one group file whose `playground` is true (`group::read`), and uses
  its number; none or more than one is a refusal naming the directory.

## Steps

1. `temper-fake/src/demo.rs`: the reader takes the forge (`forge:
   u16`) and rewrites each `resource` integer in the scripted turns'
   call arguments whose high 32 bits are 1 to `u64::from(forge) << 32
   | low`, before the script is checked and served. A comment says why
   (contracts.md, 2: numbers never reused).
2. The command's call site per A or B.
3. Tests. Then, one at a time:
   - `heavy cargo nextest run -p temper-fake`;
   - `heavy cargo nextest run -p temper-forge-setup`;
   - `heavy cargo nextest run -p temper --lib`;
   - `cargo fmt`; `heavy cargo clippy -p temper-fake -p temper -p temper-forge-setup --all-targets -- -D warnings`;
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
4. Commit and push.

## Report

The standard report, plus: A or B and why; any other place that names
the playground's forge as 1 (rg `forge 1`, `: 1,` in playground
fixtures) you found and did not change.
