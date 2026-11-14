# Lint profiles

How every package of skein, jig, temper and hearth gets its lints and its `clippy.toml`, and what
holds them there, and the count that holds what a lint can't: catching a panic. The rules
themselves are `skein/docs/foundation/programming-model.md`, 10 and
`skein/docs/foundation/testing-strategy.md`, 4; this says how they reach a crate.

## 1. In one page

- Cargo shares among members only the root's `[workspace.lints]`, all or nothing, and has no named
  tables. So a package takes that table, or keeps a block of its own that `lints.py` makes identical
  to a profile's definition.
- **Four profiles,** each written once in `tools/lints/profiles/`: `step`, `harness`, `binary` and
  `ring` (section 2). Which package takes which is section 3.
- **Three `clippy.toml` variants,** each written once in `tools/lints/clippy/`: `step`, `ordinary`
  and `ring` (section 4). Every `clippy.toml` in the four trees is a copy of one, byte for byte.
- **A change of policy is one edit:** change the definition and run `lints.py --write` (section 5).
- **A crate's real exception** is a scoped `#[expect(..., reason = "...")]` in its code, never a
  manifest of its own (section 6). No profile allows a `disallowed_*` lint.
- **Catches only fall:** a count per file of `catch_unwind`, `resume_unwind` and `#[should_panic]`
  in every Rust file git tracks, against a list that only falls, refuses any new one (section 7).

## 2. The profiles

A profile is one block of TOML, the text a manifest holds. The block runs from the manifest's first
`[lints` line to the next table that isn't a `[lints` one.

### 2.1 step

`[lints]` with `workspace = true`: the root's `[workspace.lints]`, the subset of
`skein/docs/foundation/programming-model.md`, 10.3. A manifest says nothing else.

### 2.2 harness

For test code: ordinary Rust (`skein/docs/foundation/programming-model.md`, 10.2). `unsafe_code` is
forbidden, a lint exception is a scoped `#[expect]` with a reason, and a `_` covering one variant is
out. `wildcard_enum_match_arm` is allowed: tests are ordinary Rust, and listing every variant to
pick one event out costs hundreds of lines and checks nothing. A test keeps an exhaustive match by
convention where breaking the build is the point: a referee classifying events, a judge over
stories.

### 2.3 binary

For ordinary Rust that ships and runs on no ring of its own: `harness`, with
`wildcard_enum_match_arm` denied, since a new variant must not build silently in code a user runs.

### 2.4 ring

For a shell on the ring: ordinary Rust, with the lints that keep the one module allowed `unsafe`
small and documented (`skein/docs/foundation/programming-model.md`, 2.1), and the subset's checks
that matter next to the syscalls: no silent casts, overflow or panicking indexing. `unsafe_code` is
denied, so that the ring adapter can expect it in place; a crate here with no `unsafe` says
`#![forbid(unsafe_code)]` at its root.

## 3. Which crate takes which

- **`step`:** every crate held to the subset, and any crate under `testing/` or `tests/` held to the
  step crates' lints (a fake that runs as a step machine, `skein/docs/design/testing/README.md`, 6). A crate
  already on `step` stays there.
- **`harness`:** the worlds under `tests/`, the kits under `testing/`, the fakes under `crates/`, and
  the development tools: their crates under `crates/` and their binaries' packages under `bin/`
  (`temper/docs/design/testing/tools.md`, 3).
- **`binary`:** the crates that ship, are ordinary Rust, and hold no ring of their own: an
  application's binary and its client shells, its bundler, the browser parts, a playground it ships.
- **`ring`:** `skein-shell`, the binaries built directly on it that hold to its rules, and hearth's
  binary; `skein-fork`, a kit under `testing/` whose process boundary needs documented `unsafe`.

`binary` and `ring` differ by what a crate can meet: a client shell of thousands of lines and tests
indexes and counts as ordinary Rust does, and the ring's shell does neither next to a syscall.

## 4. The `clippy.toml` variants

Clippy reads the nearest `clippy.toml` above a crate's manifest and nothing beyond it, so a variant
applies to a directory's crates through one copy, or to a crate through its own.

- **`step`:** the tree's `clippy.toml`, the subset's bans.
- **`ordinary`:** a tree's `tests/clippy.toml`, and a copy in any other
  crate on `harness` or `binary` that `ring` doesn't take, and in a crate on `step` that is ordinary
  Rust, as `skein-world` and `skein-fail-stop` are. A run is one thread and one loop, and a seed
  replays. A step crate on `testing/` sits beside the harness kits there, so `testing/` has no
  `clippy.toml` of its own: each kit has a copy.
- **`ring`:** `ordinary`'s bans and no std handle that closes on drop, nor shared ownership
  (`skein/docs/foundation/programming-model.md`, 2.1). Copies sit in the crates on `ring`; in the
  crates that play the kernel: the simulator, the fake machines, hearth's fake git and fake
  mmdebstrap; and in five kits of skein's that hold neither: `skein-sim-world`, `skein-playground`,
  `skein-conformance`, `skein-heap` and `skein-scratch`.

**The process's arguments and environment,** in every variant: `disallowed-methods` lists `std::env`'s
`args`, `args_os`, `var`, `var_os`, `vars`, `vars_os`, `set_var` and `remove_var`, and
`skein_shell::arguments` and `skein_shell::environment`, which read them for a binary's crate, those two
`allow-invalid`, since most crates don't link the shell. Each with one reason, citing
`skein/docs/foundation/cli.md`, 6. A read is a scoped `#[expect(clippy::disallowed_methods, reason =
"...")]` whose reason starts with what it is: the binary's capture, a build script, a test harness, or
the round or milestone that removes it (`skein/docs/foundation/cli.md`, 9, check 4).

A crate's lint profile and its variant are separate choices: `skein-sim` is on `step` with `ring`'s
bans, since it holds to the step crates' lints and is ordinary Rust.

A copy whose nearest parent holds the same variant is deleted.

## 5. The check

```sh
python3 -I -B tools/lints/lints.py                       # the gate: 0 findings
python3 -I -B tools/lints/lints.py skein/tests hearth    # below these paths only
python3 -I -B tools/lints/lints.py --set harness PATH... # give these manifests a profile
python3 -I -B tools/lints/lints.py --write               # follow a changed definition
python3 -I -B tools/lints/test_lints.py
```

From the repository root, on the working tree, with Python's standard library alone.

- **Packages:** the root's `workspace.members` under the four trees. Each manifest's block is a
  profile's definition, byte for byte. A block's first line names its profile, but for `step`, whose
  block is its definition. A second block, or none, is a finding.
- **`clippy.toml`:** every file of that name git tracks under the four trees is a variant's
  definition, byte for byte. Its first line names its variant.
- **Findings:** one line each, `PATH: what is wrong`, then a count. Exit 1 on any finding, 0 on none.
- **`--set NAME PATH...`** replaces each manifest's block with the definition, and deletes the
  comment lines directly above it, which spoke of the block it replaces.
- **`--write`** rewrites each block and each `clippy.toml` from the definition its first line names.

The gate runs it (`.config/gate/README.md`).

## 6. Exceptions

A level or a ban that a crate cannot meet is a scoped `#[expect(lint, reason = "...")]` on the item,
or at the crate's root when the whole crate has one reason, as a step crate's
`#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]`
does: step code takes no closures (`skein/docs/foundation/programming-model.md`, 10.3). Where a crate's
only sites are in its test modules, the root says `#![cfg_attr(test, expect(...))]`.

## 7. The count of catches

Nothing catches a panic, tests included (`skein/docs/foundation/programming-model.md`, 10.4). Tests
on main still do, and a count holds them where they stand until each is converted: it lets no new
one in and lets none come back. It holds `#[should_panic]`, which no lint can ban, and its file
holds `tools/`' fixtures, which clippy never compiles.

```sh
python3 -I -B tools/lints/panics.py           # the gate: 0 findings
python3 -I -B tools/lints/panics.py --write   # lower the list to what the files hold
python3 -I -B tools/lints/test_panics.py
```

From the repository root, on the working tree, with Python's standard library alone.

### 7.1 What it counts

Per file, in every file git tracks whose name ends `.rs`, whatever its tree or directory:

| Pattern | What counts once |
|---|---|
| `catch_unwind` | a line naming `catch_unwind` as a whole word: a call, a `use` line, a comment |
| `resume_unwind` | a line naming `resume_unwind` as a whole word, the same way |
| `should_panic` | an attribute: each `#[should_panic`, bare or with its arguments, on a line that isn't a comment line |

A comment line is one whose first text after blanks is `//`, so a doc comment that quotes
`#[should_panic(...)]` doesn't count, and `clippy::should_panic_without_expect` in an `#[expect]`
names no pattern. On main at
`62b2474b27` the three come to 210 `catch_unwind` lines, 11 `resume_unwind` lines and 463
`#[should_panic]` attributes, the figures of `temper/docs/design/draft/panics.md`, 2, tree by
tree.

### 7.2 The list

`tools/lints/panics.txt`. A line starting `#` is a comment; every other line is

```
PATH PATTERN COUNT
```

a tracked path from the repository root, one of the three pattern names and a count of at least 1,
separated by single spaces. Lines are sorted by path, then by pattern in the table's order, with
at most one line for a path and a pattern. A path with no line for a pattern is listed at 0.

### 7.3 Findings

One line each, `PATH: what is wrong`, then their count and the rule they break
(`skein/docs/foundation/programming-model.md`, 10.4). Exit 1 on any finding, 0 on none.

- **A count above its line:** a file's count of a pattern above the list's, a new file's or a
  pattern the file had none of among them, since those are listed at 0.
- **A malformed list:** a line not of 7.2's form, out of order or repeated, or no list at all.

A count below its line, or a line whose file is gone, is no finding: the check prints it with
`--write` as the way to lower it, and exits 0 if nothing else is wrong. A change that lowers a count
lowers its line in the same change; another branch that only removes catches never fails the count.

### 7.4 Only falling

- **`--write`** rewrites each line to the count now, drops each line that reaches 0, and keeps the
  comment lines where they stand. It never raises a line and never adds one, but writes every
  file's counts when there is no list. With a finding standing it writes nothing and exits 1.
- **Once a pattern counts none,** it has no line left, and any count of it anywhere is above its
  line: the check refuses any, and keeps refusing.
- **A file that moves** takes its lines with it, in the same change: its old path's lines go and
  its new path's come, at the same counts. That is the one line added by hand; any other raise
  breaks the rule this count holds.
- **The gate runs it** beside `lints.py` (`.config/gate/README.md`, 8), and `fabrig-boundary` runs
  it and `test_panics.py` in its focused tests, so that both run on every change
  (`.config/gate/always.txt`).
- **What follows:** once `catch_unwind` and `resume_unwind` count none, `disallowed-methods` lists
  them in every `clippy.toml` the gate's clippy reads (section 4), and the count keeps
  `#[should_panic]` alone (`temper/docs/design/draft/panics.md`, 7).

## 8. Later

- smith's packages and the root's `tests/boundary` take a profile when this debt is taken up for
  them: they are outside the four trees, and no item of `smith/docs/development/debt/` holds it yet.
  smith's reads of the process's arguments and environment take section 4's ban with it, in smith's
  round of `skein/docs/foundation/cli.md`.
- The closure-ban lints could be set once in `[workspace.lints.clippy]`, which would retire each
  crate's root `manual_map` expectation.
- skein's `manual_map` exceptions could take section 6's one wording at each crate's root, as
  temper's and jig's do: the user's request of 2026-11-04 asked it of temper alone.
- **Four profiles or three, awaiting the user.** The request names three: `step`, `harness` and
  `binary`. Default: the four of section 2 stand, because `binary` held two regimes. Three by
  dropping `ring`'s further lints loosens `skein-shell`, hearth's binary and the two examples, and
  merges two definitions and four manifests. Three by raising the twelve crates on `binary` to them
  is a far larger plan.
- **Where the check runs, awaiting the user.** The request has it run in a step outside this
  repository, beside `tools/citations/citations.py`. Recommended: that step gains the line
  `python3 -I -B tools/lints/lints.py`; the other answer leaves the boundary test as its one run
  (section 5).
