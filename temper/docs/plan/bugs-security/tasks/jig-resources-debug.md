+++
id = "jig-resources-debug"
part = "jig-debug"
kind = "feature"
needs = []
files = [
  "jig/testing/jig-test-service/src/program_sources.rs",
  "jig/testing/jig-test-service/src/tests/program_sources.rs",
  "jig/testing/jig-test-service/src/tests.rs",
]
effort = "medium"
minutes = 30
+++

# jig-test-service's program resources print counts, never the environment

## Goal

`jig/testing/jig-test-service/src/program_sources.rs`'s `Resources` derives `Debug`, and its
`environment` entries may hold credentials (`API_KEY=...`), so a debug print or a trace of it, or
of the `Component` that holds it, shows them. Replace the derive with a hand-written `Debug` that
shows only counts, as temper's copy (`temper/crates/temper-service/src/program_sources.rs`, ~20)
does.

Acceptance:

- `format!("{resources:?}")` of a `Resources` is exactly `Resources { programs: 1,
  environment_entries: N }`, `N` the number of environment entries, the string temper's copy
  gives; no byte of a path or an entry appears.
- A `Component` configured with such a `Resources` formats without the canary (its derived
  `Debug` now prints the resources through the new impl).
- The new test fails on the old code (the derived form prints the canary) and passes on the new.
- `Component`, `Command` and every other type of the file keep their derives.
- clippy clean; the crate's lib tests pass.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md`.
- `jig/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 1.
- `jig/docs/design/domain/hosts.md`, 5.1: the program's "explicit executable, private directory
  and bounded environment below the root", whose entries may carry a program's credential.
- `skein/docs/design/lib.md`, 6: `Secret`'s `Debug` shows its length only, so a record holding
  one "prints, logs and traces without it": the rule this fix follows for bytes that are not a
  `Secret`.
- For the shape: `temper/crates/temper-service/src/program_sources.rs`, ~16-25 (the impl and
  its comment), and its test `program_resource_debug_exposes_only_counts` in
  `temper/crates/temper-service/src/tests/worker_parts.rs` (~362). This task does not touch them.

## Owns and may change

- `jig/testing/jig-test-service/src/program_sources.rs`: `Resources`' derive and a hand-written
  `impl core::fmt::Debug for Resources`, nothing else in the file.
- `jig/testing/jig-test-service/src/tests/program_sources.rs`: new, the tests below.
- `jig/testing/jig-test-service/src/tests.rs`: the one line `mod program_sources;`, among the
  other `mod` lines in name order.

Nothing else. No signature changes, so no reader edits: `jig/testing/jig-test/src/lib.rs` and
`src/tests.rs` build `Resources` through `from_paths` and never format it.

Pre-approved options:

- If clippy's `missing_fields_in_debug` fires on the impl, add a scoped
  `#[expect(clippy::missing_fields_in_debug, reason = "environment entries may hold credentials; Debug shows counts (hosts.md, 5.1)")]`.
- Formatting in the test: `alloc::fmt::format(format_args!(...))` or a scoped
  `#[expect(clippy::disallowed_macros, reason = "...")]`, whichever the crate's lints accept
  (`notes/common.md`).
- If `Command`'s `ProgramSource` (from `smith-host-protocol`) prints the environment through a
  running command's `Debug`, that is smith's type: leave it and name it under `## Left`.

## Steps

1. Tests first, in `src/tests/program_sources.rs` (the crate's lib tests,
   `tests::program_sources`), each with a made-up canary (`notes/common.md`):

   ```rust
   const CANARY: &[u8] = b"canary-7f3a9c";

   fn resources() -> crate::program_sources::Resources {
       // claude and directory paths as temper's test gives them; one entry `API_KEY=<CANARY>`
   }

   #[test]
   fn program_resources_debug_shows_only_counts() {
       // exactly "Resources { programs: 1, environment_entries: 1 }"; the canary absent
   }

   #[test]
   fn a_configured_component_formats_without_the_environment() {
       // crate::program_sources::Component::new(64), configure(resources()), format it:
       // "environment_entries: 1" present, the canary absent
   }
   ```

   Run `heavy cargo nextest run -p jig-test-service --lib -E 'test(/program_sources/)'` on the
   unchanged `Resources` and see both fail (the derived form prints the entry); keep the output
   for the report.
2. In `program_sources.rs`, drop `Debug` from `Resources`' derive and write the impl temper's
   copy has, with its comment ("Command entries may contain credentials. Debug exposes only
   retained counts.").
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p jig-test-service --lib -E 'test(/program_sources/)'`: 2 passed.
   - `heavy cargo nextest run -p jig-test-service --lib`.
   - `heavy cargo check -p jig-test --tests` (the one reader of `Resources`).
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`;
   `heavy cargo clippy -p jig-test-service --all-targets -- -D warnings`.
5. One commit, the impl and its tests, saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code (command, count, the
assertion that failed), the passing runs and their counts, and anything under `## Left`.
