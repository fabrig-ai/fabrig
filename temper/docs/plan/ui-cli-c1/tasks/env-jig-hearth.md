+++
id = "env-jig-hearth"
part = "env-access"
kind = "migration"
needs = ["env-ban"]
files = ["jig/**/*.rs", "hearth/**/*.rs"]
effort = "medium"
minutes = 70
+++

## Goal

`env-ban` made clippy refuse every read of the process's arguments and
environment (`tools/lints/design.md`, 4). jig's and hearth's reads now
say what they are with a scoped `expect`, so their clippy is green
again: each waits for its tree's own round of the command line's
conventions (`skein/docs/foundation/cli.md`, 10).

Acceptance:

- Every read in `jig/` and `hearth/` (`std::env`'s eight functions,
  `skein_shell::arguments` and `environment`) is under a scoped
  `#[expect(clippy::disallowed_methods, reason = "...")]` whose reason
  starts `jig's round: ` or `hearth's round: `, then what it reads
  (`notes/contracts.md`, 2). An expectation already there for
  `std::env::var` stays as it is.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` over every
  jig and hearth package with a read passes, and the wasm32 clippy of
  each such package listed in `.config/gate/wasm32.txt`.

## Inputs

- `tools/lints/design.md`, 4; `skein/docs/foundation/cli.md`, 6 and 9,
  check 4.
- `temper/docs/plan/ui-cli-c1/notes/contracts.md`, 2 (binding), and
  `notes/common.md`.

## Owns and may change

- In `jig/` and `hearth/`, expectation lines alone: an attribute added,
  or nothing. No code changes, no other lint, no `clippy.toml`.
- Smallest item that holds the read, or a test file's `#![expect(...)]`
  where four or more of its items read (contracts 2). If a read sits
  where an item-level `expect` can't reach (inside a macro's
  expansion), take the nearest enclosing item and say where.
- Not yours: skein's, temper's and smith's code.

## Steps

1. The sites: `rg -n 'env::(args|args_os|var|var_os|vars|vars_os|set_var|remove_var)\b|skein_shell::(arguments|environment)\b' jig hearth`,
   and `rg -n 'use std::env|env::\{' jig hearth` for the hidden ones;
   then `heavy cargo clippy` over their packages to catch any the search
   missed.
2. Each item gets its `expect`. Reasons, for example:
   `"hearth's round: the binary's arguments and XDG_CACHE_HOME, read in main"`,
   `"jig's round: reads SKEIN_CODEGEN_WRITE, the codegen's rewrite switch"`.
3. `cargo fmt`; the clippy runs. An attribute changes no behaviour, so
   no test runs (a departure from the implementer's text; say so). Bite:
   remove one `expect` and see clippy name `disallowed_methods` there;
   restore. Commit, push.

## Report

How many expectations, by package; the clippy runs; the bite; any
read placed by judgement.
