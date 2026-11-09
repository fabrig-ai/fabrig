+++
id = "env-temper-src"
part = "env-access"
kind = "migration"
needs = ["env-ban", "table-types"]
files = ["temper/crates/*/src/**", "temper/crates/*/build/**"]
effort = "medium"
minutes = 90
+++

## Goal

`env-ban` made clippy refuse every read of the process's arguments and
environment (`tools/lints/design.md`, 4). Every read in temper's crates'
sources and build scripts now says, with a scoped `expect`, the
milestone of `temper/docs/design/draft/cli.execution.md` that brings it
into line, or that it's a build script's, so temper's clippy is green
again and each later milestone finds its reads by `rg 'cli C2:'`.
`env-temper-tests` does the same for temper's tests, tools and test
kits after it.

Acceptance:

- Every read under `temper/crates/*/src/` and `temper/crates/*/build/`
  (`std::env`'s eight functions, `skein_shell::arguments` and
  `environment`) is under a scoped `#[expect(clippy::disallowed_methods,
  reason = "...")]` whose reason starts with `cli C2: `, `cli C3: `,
  `cli C4: `, `cli C6: ` or `build script: ` by `notes/contracts.md`, 4,
  then what it reads and where it goes. temper uses no `capture`: no
  read of its sits behind a table yet.
- Each existing `disallowed_methods` expectation over such a read keeps
  its place, its reason rewritten to start with its prefix.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes
  over every package under `temper/crates/` you changed, but for reads
  in their integration tests (`temper/crates/*/tests/`), which
  `env-temper-tests` takes next: name those packages under `## Left`.
  The wasm32 clippy of each of them listed in `.config/gate/wasm32.txt`
  passes.

## Inputs

- `tools/lints/design.md`, 4; `skein/docs/foundation/cli.md`, 6 and 9,
  check 4.
- `temper/docs/design/draft/cli.execution.md`, 1: the milestones.
- `temper/docs/plan/ui-cli-c1/notes/contracts.md`, 2 and 4 (binding),
  and `notes/common.md`.

## Owns and may change

- Under `temper/crates/*/src/` and `temper/crates/*/build/`, expectation
  lines alone: an attribute added, or a reason's text. No code changes,
  no other lint, no `clippy.toml`. `table-types` changed the `Settings`
  literals in `box_command.rs`, `fake.rs` and `temper-shell`'s
  `directories.rs`; leave them.
- Which milestone, by contracts 4: a read in `temper-client` or a
  `temper-client-*` crate is `cli C3`; in `temper-agent`, `cli C6`; in
  a test (a `src/tests` module, a `#[cfg(test)]` item), `cli C4`; in a
  build script (`build/`), `build script`; any other read, in a crate
  the `temper` or `temper-engine` binary links (`cargo tree -p temper -e
  normal --prefix none`, which builds nothing), `cli C2`. A crate
  neither links, outside tests, is a tool's or a fixture's: `cli C8`,
  and the report names it.
- Smallest item that holds the read, or a test file's
  `#![expect(...)]` where four or more of its items read (contracts 2).
  A `main` reading at several places takes one `expect` on `main`.
- Not yours: `temper/crates/*/tests/**`, `temper/tests/**`,
  `temper/testing/**`, `temper/tools/**` (`env-temper-tests`); other
  trees.

## Steps

1. The sites: `rg -n 'env::(args|args_os|var|var_os|vars|vars_os|set_var|remove_var)\b|skein_shell::(arguments|environment)\b' temper/crates --glob '!**/tests/**' --glob '!*.md'`
   (then the `src/tests` modules, which that glob leaves out: `rg` them
   too), and `rg -n 'use std::env|env::\{' temper/crates`; then clippy
   over the packages to catch any the search missed.
2. Each item gets its `expect`. Reasons, for example:
   `"cli C2: TEMPER_DUMP_LLM, read where the service starts; temper-box's table will pass it"`,
   `"cli C3: the client's arguments, captured in main; temper's table will read them"`,
   `"build script: TEMPER_BUNDLE and Cargo's OUT_DIR"`.
3. `cargo fmt`; the clippy runs. An attribute changes no behaviour, so
   no test runs (a departure from the implementer's text; say so). Bite:
   remove one `expect` and see clippy name `disallowed_methods` there;
   restore. Commit, push.

## Report

How many expectations by prefix and by crate, the reasons rewritten,
any crate placed by judgement, the clippy runs and the bite.
