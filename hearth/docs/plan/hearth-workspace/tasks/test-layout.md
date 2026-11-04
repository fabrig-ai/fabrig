+++
id = "test-layout"
part = "layout"
kind = "migration"
needs = ["manifest-paths"]
files = [
  "hearth/tests/*/tests/**",
  "hearth/tests/render/src/fixtures/decoders.rs",
  "hearth/tests/*/Cargo.toml",
  ".config/nextest.toml",
  "hearth/docs/design/domain/testing.md",
  "hearth/docs/design/STATUS.md",
]
effort = "medium"
minutes = 120
+++

# hearth's worlds take the monorepo's test layout: one `suite`, with its `fuzzy` module

## Goal

hearth has 75 top-level test binaries and 20 `fuzzy_*.rs` across its 9 worlds. The rule is
`tests/suite/` with `tests/suite/fuzzy/` and one counting allocator per suite
(`notes/contracts.md`, 4), and the root's fuzzy profile selects a package's `suite` binary's `fuzzy`
module. Move them. This also cuts what hearth adds to the gate's link time. Tests move; they don't
change what they check.

It's one task, not one per world: `tests/render/tests/decoders.rs` reads every world's fuzzy files, so
the moves meet there.

Acceptance:

- In each world, `tests/` holds `suite/` and only the tier binaries: `conformance_*`, `recorded_*`,
  `real` (with `real/`). `hearth/crates/hearth/tests/` is as it was.
- **The same tests:** `heavy cargo nextest list -p 'hearth*'` under `default` and `fuzzy` give, at your
  base and after, the same two counts. The report gives them. A test's id changes only by its binary
  and module.
- Both profiles pass with `-p 'hearth*'`; the report gives each one's wall time, before and after,
  and the count of test binaries.
- `heavy cargo clippy -p 'hearth*' --all-targets -- -D warnings` passes, and
  `heavy cargo nextest run -p fabrig-boundary` passes.
- `python3 -I -B tools/citations/citations.py --all` reports 0.

## Inputs

- `notes/common.md` and `notes/contracts.md`, sections 2 and 4, in this plan's directory
  (`hearth/docs/plan/hearth-workspace/`).
- `skein/docs/design/testing.md`, 6, its last two lines; `skein/tests/oauth/tests/suite/main.rs`.
- `hearth/docs/design/domain/testing.md`, 2.9, 8.4 and 9.2.
- This cut wrote no design: the request changes none, and the status edits below are what it allows.

**Departures from the implementer's text:** you edit `.config/nextest.toml`, and
`hearth/docs/design/domain/testing.md` and `hearth/docs/design/STATUS.md` as step 5 says, and no other
design file.

## Owns and may change

**Owns:** the 9 worlds' `tests/` directories, but for the tier binaries' bodies.

**May change, only as the steps say:** `tests/render/src/fixtures/decoders.rs`, a world's `Cargo.toml`
(a dev-dependency on `skein-heap` if its suite gains the allocator and lacks one),
`.config/nextest.toml` (hearth's two clauses), the two design files.

Pre-approved, each with its default:

- **Name clashes in a merged binary:** a helper two files each defined stays in each as its own module's;
  nothing needs renaming. `tests/x.rs` and `tests/suite/fuzzy/x.rs` don't clash.
- **`tests/render/tests/oracle/engine.rs`,** which two binaries `#[path]`-include: it moves to
  `suite/oracle/engine.rs`, and the fuzzy file reaches it through `crate::oracle::engine`, if `oracle`'s
  module exposes it; otherwise it keeps a `#[path]`. Yours.
- **A world with nothing to count:** `hearth-conformance-world`'s suite holds `resolve_stanzas.rs` alone and
  no memory test: no allocator there, and no dependency for one.
- **A test that passed only because it had a process's binary alone:** nextest gives each test its own
  process, so none should. One that does is a finding: report it; don't weaken it.
- **Fuzzy over its slow period, or a suite slower than at your base:** report it. Budgets aren't yours.

## Steps

1. **Before moving:** record both counts and the wall times at your base.
2. **Move, world by world, with `git mv`,** one commit a world: accounts, builder, config, conformance
   (`resolve_stanzas.rs` alone), git, mmdebstrap, render, template, tree. The request says about 75
   moves, 24 `#[path]` or `mod` fixes and 9 `main.rs`.
   - `tests/x.rs` to `tests/suite/x.rs`, `tests/x/` to `tests/suite/x/`, `tests/fuzzy_x.rs` to
     `tests/suite/fuzzy/x.rs`.
   - `tests/suite/main.rs` as `skein/tests/oauth/tests/suite/main.rs`: each moved file's own
     `#[global_allocator]` goes, and the suite holds one.
   - `#[path]` and `include_*!` paths gain the two or three `../` a move costs, as in
     `tests/builder/tests/memory.rs`'s `../src/fixtures/bounds.rs`, and
     `tests/config/tests/fuzzy_goldens.rs`'s `../../render/src/fixtures/decoders/golden.rs`.
3. **The inventory reads the new layout.** `tests/render/src/fixtures/decoders.rs`: `source(world, file)`
   reads `tests/<world>/tests/suite/fuzzy/<file>`, and `fuzzy_files()` lists that directory, where a world
   has it. `tests/render/tests/decoders.rs`: each row's and `DOMAIN_REFERENCES`' `file` is the new name
   (`fuzzy_git.rs` becomes `git.rs`), and `row.file.starts_with("fuzzy_")` goes, because the directory says
   it now. `every_fuzzy_file_is_a_row_or_a_domain_reference` must still read files: it asserts so.
4. **`.config/nextest.toml`:** hearth's clauses for after `test-layout` (`notes/contracts.md`, 2). Do this
   when no `fuzzy_*` binary remains.
5. **Design, the two files only:**
   - `hearth/docs/design/domain/testing.md`, 2.9's layout listing: `tests/*/tests/*.rs` becomes
     `tests/*/tests/suite/*.rs`, and `tests/*/tests/fuzzy_*.rs` becomes `tests/*/tests/suite/fuzzy/*.rs`.
     Only those two lines' paths, with their column.
   - `hearth/docs/design/STATUS.md`: `Updated` takes today's date (`date -u +%F`), and below the table:

     ```markdown
     - **Departs:** `domain/testing.md`: 8.4: a world's memory tests are modules of its suite, not a binary of their own, and a process's heap is checked under scenario ceilings, with no worst case of the whole, because the root's rules are skein's (`skein/docs/foundation/programming-model.md`, 6.3). 9.2: the `conformance` profile is `conformance-hearth` and also runs `recorded_*`, and there is no `measure` profile, because the root's tier profiles are `<tier>-<tree>`.
     ```

     This file isn't a plan file: `tests/boundary/tests/outward.rs` reads it.
6. Fmt, old forms, clippy, tests, citations; commit and push.

## Report

The implementer's report, and: both counts and wall times, before and after; the test binaries before and
after; every test that failed after a move.
