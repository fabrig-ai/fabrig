+++
id = "env-ban"
part = "env-access"
kind = "migration"
needs = []
files = ["tools/lints/clippy/*.toml", "skein/**/clippy.toml", "jig/**/clippy.toml", "temper/**/clippy.toml", "hearth/**/clippy.toml", "skein/**/*.rs", "tools/lints/design.status.md"]
effort = "medium"
minutes = 90
+++

## Goal

Check 4 of the command line (`skein/docs/foundation/cli.md`, 9):
clippy's `disallowed-methods`, in every variant of `tools/lints/clippy/`,
lists the process's argument and environment readers, and every
`clippy.toml` copy follows. Then every read in skein says what it is
with a scoped `expect`, so skein's clippy stays green. jig's, hearth's
and temper's reads are three later tasks' (`env-jig-hearth`,
`env-temper-src`, `env-temper-tests`): until they merge, those trees'
clippy fails `disallowed_methods` on this plan's trunk, as planned.

Acceptance:

- `tools/lints/clippy/step.toml`, `ordinary.toml` and `ring.toml` each
  list `notes/contracts.md`, 2's ten entries in `disallowed-methods`,
  with its one reason; `std::env::var`'s old entry is replaced by its
  new one. `python3 -I -B tools/lints/lints.py --write` has rewritten
  every copy, and `python3 -I -B tools/lints/lints.py` reports 0
  findings.
- Every read in `skein/` (`std::env`'s eight functions,
  `skein_shell::arguments` and `skein_shell::environment`) is under a
  scoped `#[expect(clippy::disallowed_methods, reason = "...")]` whose
  reason starts with contracts 2's prefix: `capture` for
  `skein_shell::arguments` and `environment` themselves, `skein's round`
  for every other read. An expectation already there for `std::env::var`
  stays as it is.
- `heavy cargo clippy` over every skein package with a read, `--all-targets
  -- -D warnings`, passes, and the wasm32 clippy of `skein-shell`
  (`--target wasm32-unknown-unknown`) too.
- A crate that links no `skein-shell` gets no warning from the two
  `allow-invalid` paths: shown by one clippy run of such a crate (say
  `skein-lib`).
- The **Not yet** entry 4 in `tools/lints/design.status.md` (its two
  lines, from `4: the variants list`) is removed, its lines alone. This
  departs from the implementer's text, which closes design files to you:
  the entry is this task's.

## Inputs

- `tools/lints/design.md`, 4 (the paragraph *The process's arguments
  and environment*) and 5 (`lints.py --write`).
- `skein/docs/foundation/cli.md`, 6 and 9, check 4.
- `temper/docs/plan/ui-cli-c1/notes/contracts.md`, 2 (binding), and
  `notes/common.md`.

## Owns and may change

- Owns the variants, every `clippy.toml` copy (rewritten by `lints.py
  --write`, never by hand) and, in `skein/`, the expectation lines alone:
  an attribute added or its reason's text, nothing else of the code.
- In `skein/crates/skein-shell/src/settings.rs`, only the lines above
  `pub fn arguments` and `pub fn environment` (`table-types` changes the
  rest of that file in parallel). Same in `listeners.rs`,
  `listener_tests.rs` and `readiness.rs`: expectation lines only.
- If clippy rejects a `std::` path in `step.toml` for a `no_std` crate
  (a warning that it doesn't resolve), give that entry `allow-invalid =
  true` too and say so.
- If a skein read sits in a macro's expansion or a place an item-level
  `expect` can't reach, put it on the nearest enclosing item and say
  where.
- Not yours: jig's, hearth's, temper's and smith's code; smith's own
  `clippy.toml` files (outside `tools/lints/`, smith's round).

## Steps

1. **The variants:** contracts 2's ten entries, in each of the three,
   beside the existing `disallowed-methods`; the reason once per entry,
   word for word.
2. `python3 -I -B tools/lints/lints.py --write`, then
   `python3 -I -B tools/lints/lints.py` (0 findings) and
   `python3 -I -B tools/lints/test_lints.py`.
3. **The sites:** `rg -n 'env::(args|args_os|var|var_os|vars|vars_os|set_var|remove_var)\b|skein_shell::(arguments|environment)\b' skein`,
   and the ones a `use std::env;` hides (`rg -n 'use std::env|env::\{' skein`).
   Each item gets its `expect`, the smallest item that holds the read,
   or a test file's `#![expect(...)]` where four or more of its items
   read (contracts 2). Reasons, for example:
   `"capture: what each binary's crate calls once (skein/docs/foundation/cli.md, 6)"`,
   `"skein's round: reads SKEIN_CODEGEN_WRITE, the codegen's rewrite switch"`.
4. `cargo fmt`; `heavy cargo clippy -p PKG ... --all-targets -- -D
   warnings` naming each skein package you changed (list them with `rg
   -l` and the manifests); then the clippy of one crate not linking the
   shell; then the wasm32 clippy of `skein-shell`. The tests of what
   you changed need no run: an attribute changes no behaviour, so name
   that in the report instead. Bite: remove one `expect` and see clippy
   name `disallowed_methods` there; restore. Commit, push.

## Report

The variants' diff in one line each, how many expectations you added
and where (by crate), the clippy runs, the bite and what it showed,
anything `allow-invalid` needed, and, under `## Left`, that jig's,
hearth's and temper's reads wait for the three tasks named above.
