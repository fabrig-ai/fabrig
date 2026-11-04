+++
id = "join-steps"
part = "join"
kind = "migration"
needs = []
files = [
  "Cargo.toml",
  "Cargo.lock",
  ".config/nextest.toml",
  "AGENTS.md",
  "hearth/AGENTS.md",
  "hearth/Cargo.toml",
  "hearth/Cargo.lock",
  "hearth/.config/nextest.toml",
  "hearth/scripts/old-forms.txt",
  "hearth/crates/hearth-client-protocol-records/**",
  "hearth/crates/hearth-domain-*/**",
  "hearth/crates/hearth-lib/**",
  "hearth/crates/hearth-protocol-*/**",
]
effort = "high"
minutes = 75
+++

# hearth's 14 step crates join the workspace, built against skein by path

## Goal

`hearth/` is a tree outside the workspace, with its own manifest, lock and nextest file and a git
dependency on an old skein. Make its 14 step crates (`notes/contracts.md`, 1) members of the root
workspace, built against `skein/` by path and run by the gate's `default` profile, and remove
hearth's own workspace. The other 16 packages stay unbuilt until `join-worlds`. Their port is that
task's: don't touch them.

Acceptance:

- `cargo metadata --format-version 1 --locked --no-deps` passes at the root and lists the 14.
- `git diff --stat Cargo.lock` shows only additions: each of the 14 packages' `[[package]]` entry,
  and no existing entry moved. `sha1` and `minijinja` aren't in it yet: nothing uses them until
  `join-worlds`.
- `heavy cargo nextest run -p 'hearth*'` passes in the `default` profile, and the report gives its
  count, over 0. No test of the 14 is removed, ignored or loosened.
- `heavy cargo clippy -p 'hearth*' --all-targets -- -D warnings` passes.
- `heavy cargo nextest run -p fabrig-boundary` passes.
- `cargo fmt --all --check` passes. `python3 -I -B scripts/old_forms.py`, from `hearth/`, passes
  with counts lowered by `--write`, and none raised.
- `python3 -I -B tools/citations/citations.py --all` reports 0.

## Inputs

- `notes/common.md` and `notes/contracts.md`, sections 1 and 2, in this plan's directory
  (`hearth/docs/plan/hearth-workspace/`).
- `hearth/docs/plan/hearth-workspace/inputs/request.md`, sections 1 and 2: the leads. Nothing was
  built for them, nor for this spec: the compiler has the last word.
- `skein/docs/foundation/programming-model.md`, 6.3.
- This cut wrote no design: the request changes none, and these sections already say what the
  port follows.

**Departures from the implementer's text,** each because the request asks it: you edit the root's
`AGENTS.md`, `hearth/AGENTS.md`, `Cargo.toml`, `Cargo.lock` and `.config/nextest.toml`, as below.

## Owns and may change

**Owns:** the 14 crates; `hearth/scripts/old-forms.txt`; the three files it removes.

**May change, only as Step 1, 2 and 5 say:** root `Cargo.toml`, `Cargo.lock`,
`.config/nextest.toml`, `AGENTS.md`, `hearth/AGENTS.md`.

**Leave alone:** `hearth/crates/hearth`, `hearth/crates/hearth-service-builder`, `hearth/testing/`,
`hearth/tests/`, `hearth/clippy.toml`, `hearth/tests/clippy.toml`, `hearth/rustfmt.toml`,
`hearth/scripts/fail_stop.py`, every other tree, `tools/`.

Pre-approved, each with its default:

- A lint the current toolchain fires in a step crate, on code you ported: a scoped `#[expect]`, as
  the implementer's text allows; on code the port didn't touch, too, and the report lists them.
- A `hearth/clippy.toml` entry that names something skein removed fails clippy's config read: stop
  and ask only if dropping the line isn't what fixes it; otherwise drop it and say so. You may edit
  that one file for that.
- A unit test of the 14 that compares a formula with a literal figure, red because `price` differs
  from the old `worst_case`: read why. If the formula is right and the literal only restated the
  old figure, update the literal, and say so in the report and the commit. Anything else is the
  rule in `notes/common.md`.
- `-p 'hearth*'` refused by a command: name the 14 with `-p`.

## Steps

1. **Root `Cargo.toml`.**
   - Its first line says `one workspace for skein, smith, jig, temper and hearth`.
   - The 14 `hearth/crates/...` lines at the end of `members`.
   - In `[workspace.dependencies]`, in a block of their own, every path entry of
     `hearth/Cargo.toml` (23) with `hearth/` before its path, including those of the 16 unbuilt
     packages, so `join-worlds` adds only members; and `sha1 = { version = "=0.10.7" }`,
     `minijinja = { version = "=2.20.0" }`. A `[workspace.dependencies]` entry nothing inherits is
     never loaded. hearth's `libc` and `skein-*` entries are already in the root, `libc` at the same
     version.
   - Don't change `[workspace.package]`, `[workspace.lints]` or the profiles. On this cut's reading
     of both files, the package table and the lints are equal key for key, and hearth's profiles
     are a subset of the root's. Check that holds before you delete hearth's.
2. **`.config/nextest.toml`.** The clauses of `notes/contracts.md`, 2, for after `join-steps`, and
   the two `global-timeout`s.
   - In the header's list, after temper's: `#   hearth: package(/^hearth(-|$)/)`.
   - jig's two slow-period overrides become hearth's too: their filters
     `package(/^jig(-|$)/) | package(/^hearth(-|$)/)`, and their comments say
     `jig's and hearth's slow periods were ...`.
3. **Remove** `hearth/Cargo.toml`, `hearth/Cargo.lock`, `hearth/.config/nextest.toml`.
4. **Port the 14,** from `heavy cargo check -p 'hearth*' --tests`:
   - `List`, `Queue`, `Stack`, `Map`, `Deadlines` `::worst_case` become `::price`. Only those:
     hearth's own functions named `worst_case` stay, as allowed old forms.
   - `hearth-protocol-config/src/json.rs`: `tk::worst_case` becomes `tk::tokenizer_bound`;
     `tk::MaxOut` is gone, so a local `MaxOut` struct as `hearth-protocol-builder/src/lib.rs`, 50,
     has one.
   - On this cut's grep that's all the 14 meet. The rest of the request's table falls in
     `join-worlds`. What the compiler adds beyond it, take the skein form a rename reaches and say
     so; what a rename doesn't reach is a question.
5. **Docs that the join makes false:**
   - `hearth/AGENTS.md`: the `Not a workspace member yet` bullet goes.
   - Root `AGENTS.md`, section 1: the first bullet reads `One repository, five trees, one Cargo
     workspace`, and adds `` `hearth/` (immutable machines, built on skein alone) `` after temper's
     entry; the bullet that says `hearth/` is a fifth tree, not yet in the workspace, goes. The
     `Dependencies point one way` bullet is the `direction` task's.
6. **Lock:** let cargo add the entries with a build-free command (`cargo metadata --format-version
   1 > /dev/null`), never `cargo update`.
7. Narrow tests, fmt, old forms, clippy and citations, then commit and push. Commits say why; the
   first names the stages (`notes/contracts.md`, 1).

## Report

The implementer's report, and: the test count; the counts old forms moved from and to; anything
the compiler asked beyond step 4; every expectation or literal the pre-approved options touched.
