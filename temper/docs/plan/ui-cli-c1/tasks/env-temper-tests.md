+++
id = "env-temper-tests"
part = "env-access"
kind = "migration"
needs = ["env-ban", "env-temper-src"]
files = ["temper/crates/*/tests/**", "temper/tests/**", "temper/testing/**", "temper/tools/**"]
effort = "medium"
minutes = 70
+++

## Goal

`env-ban` made clippy refuse every read of the process's arguments and
environment (`tools/lints/design.md`, 4). Every read in temper's
integration tests, test worlds, test kits and tools now says, with a
scoped `expect`, the milestone that brings it into line: C4, through
temper's test table, for a test; C8 for a tool or a fixture binary.
`env-temper-src`, before it, did the same for temper's crates'
sources, so a package's clippy is whole once this task's half is in.

Acceptance:

- Every read under `temper/crates/*/tests/`, `temper/tests/`,
  `temper/testing/` and `temper/tools/` (`std::env`'s eight functions,
  `skein_shell::arguments` and `environment`) is under a scoped
  `#[expect(clippy::disallowed_methods, reason = "...")]` whose reason
  starts `cli C4: ` or `cli C8: ` by `notes/contracts.md`, 4, then what
  it reads.
- Each existing `disallowed_methods` expectation over such a read keeps
  its place, its reason rewritten to start with its prefix.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes
  over every package you changed (for `temper/crates/*/tests/`, the
  crate's package).

## Inputs

- `tools/lints/design.md`, 4; `skein/docs/foundation/cli.md`, 6 and 9,
  check 4.
- `temper/docs/design/draft/cli.execution.md`, 1: C4 and C8.
- `temper/docs/plan/ui-cli-c1/notes/contracts.md`, 2 and 4 (binding),
  and `notes/common.md`.

## Owns and may change

- In the files of `files`, expectation lines alone: an attribute added,
  or a reason's text. No code changes, no other lint, no `clippy.toml`.
- Which milestone: `temper/tools/**` and a fixture binary under
  `temper/tests/` (its `src/bin/`, as `playground-serve`) are `cli C8`;
  everything else here is `cli C4`.
- Smallest item that holds the read, or a test file's
  `#![expect(...)]` where four or more of its items read (contracts 2).
- If clippy on a package you changed also flags a read under
  `temper/crates/*/src/` that `env-temper-src` missed, add its
  expectation by contracts 4's map and name it in the report.
- Not yours: `temper/crates/*/src/**`, `temper/crates/*/build/**`;
  other trees.

## Steps

1. The sites: `rg -n 'env::(args|args_os|var|var_os|vars|vars_os|set_var|remove_var)\b|skein_shell::(arguments|environment)\b' temper/tests temper/testing temper/tools temper/crates/*/tests`,
   and `rg -n 'use std::env|env::\{'` over the same; then clippy over
   the packages to catch any the search missed.
2. Each item gets its `expect`. Reasons, for example:
   `"cli C4: TEMPER_LIVE_ACCOUNT, through temper's test table once it exists"`,
   `"cli C8: the tool's arguments, until it's a binary of temper/bin/"`.
3. `cargo fmt`; the clippy runs. An attribute changes no behaviour, so
   no test runs (a departure from the implementer's text; say so). Bite:
   remove one `expect` and see clippy name `disallowed_methods` there;
   restore. Commit, push.

## Report

How many expectations by prefix and by package, the reasons rewritten,
the clippy runs, the bite, and anything under `## Left`.
