+++
id = "jig-runner-debug"
part = "jig-holders"
kind = "feature"
needs = []
files = [
  "jig/testing/jig-test/src/lib.rs",
  "jig/testing/jig-test/src/tests.rs",
]
effort = "medium"
minutes = 25
+++

# jig-test's agent start prints the program environment's count, never its entries

## Goal

`jig/testing/jig-test/src/lib.rs`'s `SourceSettings` (~83) derives `Debug` over `environment:
Box<[Box<[u8]>]>`, the agent configuration's command environment, which may hold a program's
credential; `AgentStart` (~78) derives `Debug` over it. It is the same environment as
`jig-test-service`'s `program_sources::Resources`, whose `Debug` shows counts
(`jig-resources-debug`, landed). Give `SourceSettings` a hand-written `Debug` that prints the
paths as derived and the environment as its count.

Acceptance:

- `SourceSettings` prints `SourceSettings { directory: .., executable: .., environment_entries: N }`.
- `AgentStart` keeps its derive; formatted with sources holding the canary, it shows
  `environment_entries: 1` and not the canary in either form (`notes/debug.md`).
- The new test fails on the old code and passes on the new.
- clippy clean; the crate's lib tests pass.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md` and `notes/debug.md`.
- `jig/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, items 1 and 4.
- `jig/docs/design/domain/hosts.md`, 5.1 (the program's bounded environment);
  `skein/docs/design/lib.md`, 6, `Secret`.
- For the shape: `jig/testing/jig-test-service/src/program_sources.rs`'s `impl Debug for
  Resources` and its test in `src/tests/program_sources.rs`.
- Readers, surveyed: no test or file reads either type's `Debug` text.

## Owns and may change

- `jig/testing/jig-test/src/lib.rs`: `SourceSettings`' derive and its impl; nothing else.
- `jig/testing/jig-test/src/tests.rs`: the test below, appended.

Nothing else.

Pre-approved options:

- Formatting in the test: `alloc::fmt::format(format_args!(..))`, `std::format!`, or a scoped
  `#[expect(clippy::disallowed_macros, reason = "...")]`, whichever the crate's lints accept.
- If clippy's `missing_fields_in_debug` fires, a scoped expect citing hosts.md, 5.1.

## Steps

1. The test first, appended to `src/tests.rs` (the crate's lib tests, `tests`):

   ```rust
   #[test]
   fn debug_of_an_agent_start_shows_the_environments_count_never_its_entries() {
       // super::SourceSettings { directory: b"/private/programs", executable: b"/private/claude",
       //   environment: [b"API_KEY=canary-7f3a9c"] }: formatted alone it is exactly
       // "SourceSettings { directory: <derived>, executable: <derived>, environment_entries: 1 }";
       // the canary absent in both forms
   }
   ```

   Run `heavy cargo nextest run -p jig-test --lib -E 'test(/debug_of_an_agent_start/)'` on the
   old code: it fails. Keep the output.
2. The impl, with a one-line comment as `Resources`' has.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p jig-test --lib -E 'test(/debug_of_an_agent_start/)'`: 1 passed.
   - `heavy cargo nextest run -p jig-test --lib`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`;
   `heavy cargo clippy -p jig-test --all-targets -- -D warnings`.
5. One commit, the impl and its test, saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code, the passing runs and their
counts.
