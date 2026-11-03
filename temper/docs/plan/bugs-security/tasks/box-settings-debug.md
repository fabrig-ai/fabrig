+++
id = "box-settings-debug"
part = "temper-holders"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-box/src/host.rs",
  "temper/crates/temper-box/src/tests/host.rs",
]
effort = "medium"
minutes = 30
+++

# temper-box's settings print the inherited environment's count, never its entries

## Goal

`temper/crates/temper-box/src/host.rs`'s `Settings` (~33) derives `Debug` over `environment:
Box<[Box<[u8]>]>`, the whole process environment the binary inherits
(`temper/crates/temper/src/box_command.rs` ~65, `std::env::vars_os()`), so a debug print of the
settings, or of the `Service` that derives `Debug` over them (~137), shows every variable's
value, a token's among them. Give `Settings` a hand-written `Debug` that prints every field as
derived but the environment, which prints as its count, as temper-service's
`program_sources::Resources` does.

Acceptance:

- `Settings` prints as derived, the same fields in the same order, but `environment_entries: N`
  in `environment`'s place; no byte of an entry.
- `Service` keeps its derive; a `Service` made from settings whose environment holds the canary
  formats without it in either form (`notes/debug.md`).
- The new test fails on the old code and passes on the new.
- clippy clean; the crate's lib tests pass.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md` and `notes/debug.md`.
- `temper/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 4.
- `skein/docs/design/lib.md`, 6, `Secret`; for the shape,
  `temper/crates/temper-service/src/program_sources.rs` ~20 and its test
  `program_resource_debug_exposes_only_counts` (`src/tests/worker_parts.rs` ~362).
- Readers, surveyed: tests compare `environment` by value (`tests/host.rs` ~116), never its
  `Debug` text; no golden or trace file holds it. The test fixture's `Launch` (`tests/fixtures.rs`
  ~125) holds only made-up values and is not this task's.

## Owns and may change

- `temper/crates/temper-box/src/host.rs`: `Settings`' derive and its impl; nothing else.
- `temper/crates/temper-box/src/tests/host.rs`: the test below, appended.

Nothing else.

Pre-approved options:

- Build the settings as `tests/fixtures.rs`' harness does (~235), or reuse the harness and
  format its `Service`; if the harness keeps the service private, format the settings alone and
  a `Service::new` of them in the test.
- If clippy's `missing_fields_in_debug` fires, a scoped expect citing lib.md, 6.
- Formatting in the test: `alloc::fmt::format(format_args!(..))` or a scoped
  `#[expect(clippy::disallowed_macros, reason = "...")]`.

## Steps

1. The test first, appended to `src/tests/host.rs` (the crate's lib tests, `tests::host`):

   ```rust
   #[test]
   fn debug_of_the_settings_shows_the_environments_count_never_its_entries() {
       // Settings as the fixture builds them, environment [b"HOME=/box",
       // b"TOKEN=canary-7f3a9c"]: formatted, and the Service made of them formatted,
       // "environment_entries: 2" present, the canary absent in both forms
   }
   ```

   Run `heavy cargo nextest run -p temper-box --lib -E 'test(/debug_of_the_settings/)'` on the
   old code: it fails. Keep the output.
2. The impl, with a one-line comment as `Resources`' has.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p temper-box --lib -E 'test(/debug_of_the_settings/)'`: 1 passed.
   - `heavy cargo nextest run -p temper-box --lib`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/`;
   `heavy cargo clippy -p temper-box --all-targets -- -D warnings`.
5. One commit, the impl and its test, saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code, the passing runs and their
counts.
