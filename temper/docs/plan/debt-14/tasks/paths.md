+++
id = "paths"
part = "paths"
kind = "feature"
needs = ["ratchet"]
files = ["hearth/tests/builder/tests/real.rs", "hearth/tests/builder/tests/real/**", "hearth/tests/builder/tests/suite/domain.rs", "hearth/tests/render/tests/suite/oracle.rs", "hearth/crates/hearth-domain-render/src/tests/packages.rs", "hearth/crates/hearth-protocol-builder/src/tests/translate.rs", "hearth/crates/hearth-protocol-builder/src/tests/translate_store.rs", "hearth/crates/hearth-protocol-builder/src/tests/translate/store_arrivals.rs", "hearth/testing/hearth-test-fixtures/**", "hearth/tests/render/src/fixtures/decoders/golden.rs", "hearth/tests/config/tests/suite/fuzzy/goldens.rs", "hearth/tests/template/tests/suite/fuzzy/goldens.rs", "hearth/tests/config/Cargo.toml", "hearth/tests/template/Cargo.toml", "Cargo.toml", "Cargo.lock", ".config/fixture-readers.toml", "hearth/scripts/old-forms.txt", "hearth/docs/design/domain/testing.status.md", "hearth/docs/development/debt/D008-path-modules.md", "hearth/docs/development/debt/D009-worlds-as-kits.md"]
effort = "medium"
minutes = 50
+++

# hearth's test modules are plain modules

## Goal

22 of hearth's 25 `#[path]` attributes go. Most are redundant, or need one file moved. The two that cross crate roots
compile the goldens' corpus reader: it moves into a new kit, `hearth-test-fixtures`, which `digits` also builds on.

The other three are in `tests/builder/tests/suite/memory.rs` and `tests/config/tests/suite/memory.rs`, which this plan
leaves alone (`notes/common.md`). They stay, and D008 stays open, narrowed to them.

Acceptance:

- `git grep -E '#\[path' -- 'hearth/*.rs'` finds exactly the three lines in those two `memory.rs` files.
- `hearth/tests/builder/tests/real.rs` is gone, `hearth/tests/builder/tests/real/main.rs` exists, and the `real` binary
  lists the same tests: compare `heavy cargo nextest list -p hearth-builder-world --test real` before and after.
- The same for each module moved or re-attached: `suite` of `hearth-builder-world` and `hearth-render-world`, and the
  `--lib` lists of `hearth-domain-render` and `hearth-protocol-builder`, are unchanged, name for name.
- `hearth-test-fixtures` builds on its own, with the harness profile: `python3 -I -B tools/lints/lints.py` reads 0
  findings.
- Both goldens tests pass in the fuzzy profile, in `-p hearth-config-world` and `-p hearth-template-world`.
- `python3 -I -B tools/citations/citations.py --all` reads 0.
- `path_attribute` in `hearth/scripts/old-forms.txt` reads 3 or less, by `--write`.
- D008 is narrowed as below, and `python3 -I -B tools/debt/debt.py hearth` exits 0.

## Inputs

- `temper/docs/plan/debt-14/notes/common.md`
- `hearth/docs/development/debt/D008-path-modules.md`, `hearth/docs/development/debt/D009-worlds-as-kits.md`
- `hearth/docs/design/domain/testing.md`, 2.9: names the kit
- `skein/docs/design/testing.md`, 6
- `tools/lints/design.md`, 5
- `tools/tiers/design.md`, 9

## Owns and may change

- **Owns:** every file in the spec's `files` list, for the changes below.
- **May change:**
  - root `Cargo.toml`: the kit's `workspace.members` line, in sorted place beside `hearth/testing/hearth-test-mirror`,
    and its `[workspace.dependencies]` line, beside `hearth-test-mirror`'s. `Cargo.lock`: what cargo writes.
  - `hearth/tests/config/Cargo.toml` and `hearth/tests/template/Cargo.toml`: one `[dev-dependencies]` line each.
  - `.config/fixture-readers.toml`: the entry `"hearth/tests/render/src/fixtures/decoders/*.rs"` goes, since its only
    file moves. The kit's path holds no `fixtures` directory, so it needs no entry.
  - `hearth/docs/design/domain/testing.status.md`: this file is yours for two edits: the entry `- **Not yet:** 2.9:
    `testing/hearth-test-fixtures`.` goes, and the citation `hearth/tests/builder/tests/real.rs` becomes
    `hearth/tests/builder/tests/real/main.rs`.
  - `hearth/docs/development/debt/D009-worlds-as-kits.md`: the sentence "The goldens' readers also cross by `#[path]`
    (D008)." goes. Nothing else.
- **Not to touch:** `hearth/tests/*/tests/suite/memory.rs`, `tests/builder/src/fixtures/bounds.rs`,
  `tests/config/src/fixtures/metered.rs`, `tests/config/src/fixtures/memory.rs`. `hearth/docs/plan/**`: `citations.py`
  leaves plans out, and they are a record.
- **Options:**
  - If a moved module's file reaches its old place by a relative path (`include_str!`, `file!`), repoint it and say so.
  - If `citations.py --all` finds another citation of a moved file, fix the path alone.
  - If a test's name changes, that is a mistake in the move, not a rename: put it back.

## Steps

1. **The `real` binary.** `git mv hearth/tests/builder/tests/real.rs hearth/tests/builder/tests/real/main.rs`. In
   `main.rs`, each `#[path = "real/x.rs"]` line goes and its `mod x;` stays. In `real/reserved.rs`,
   `#[path = "reserved/hearth.rs"]` goes: from `real/reserved.rs`, `mod hearth;` finds `real/reserved/hearth.rs`.
2. **Redundant attributes.** Each of these names the place the module would be found anyway: the attribute goes, the
   `mod` line stays.

   - `hearth/tests/builder/tests/suite/domain.rs`: `domain/naive.rs`
   - `hearth/tests/render/tests/suite/oracle.rs`: `oracle/engine.rs`
   - `hearth/crates/hearth-domain-render/src/tests/packages.rs`: `packages/preseeds.rs`
3. **`translate_store.rs`.** `git mv hearth/crates/hearth-protocol-builder/src/tests/translate_store.rs
   hearth/crates/hearth-protocol-builder/src/tests/translate/store_arrivals.rs`. In `tests/translate.rs` the attribute
   goes and `mod store_arrivals;` stays. The module's path doesn't change, so its `super::` lines don't.
4. **The kit.** `hearth/testing/hearth-test-fixtures/`:

   - `Cargo.toml`, as `hearth/testing/hearth-test-mirror/Cargo.toml` has it, with `[dependencies] skein-scratch.workspace
     = true`, and `python3 -I -B tools/lints/lints.py --set harness hearth/testing/hearth-test-fixtures/Cargo.toml`.
   - `clippy.toml`: a copy of `hearth/testing/hearth-test-mirror/clippy.toml`, byte for byte.
   - `src/lib.rs`:

     ```rust
     //! What more than one of hearth's test crates reads (hearth/docs/design/domain/testing.md, 2.9).
     pub mod golden;
     ```

   - `git mv hearth/tests/render/src/fixtures/decoders/golden.rs hearth/testing/hearth-test-fixtures/src/golden.rs`,
     unchanged: `paths::tree()` reads the running test's package, so `tests/render` stays the corpus's root.
5. **The readers.** In `hearth/tests/config/tests/suite/fuzzy/goldens.rs` and
   `hearth/tests/template/tests/suite/fuzzy/goldens.rs`, the two `#[path]` lines become
   `use hearth_test_fixtures::golden;`. Each world's `[dev-dependencies]` gains `hearth-test-fixtures.workspace = true`.
6. `.config/fixture-readers.toml`, `testing.status.md` and `D009`, as above.
7. **D008.** Rewrite the item, keeping its ID and file name:

   - Title: `# D008: Two memory suites reach their fixtures by `#[path]``
   - **Where:** `tests/builder/tests/suite/memory.rs`, `tests/config/tests/suite/memory.rs`,
     `tests/builder/src/fixtures/bounds.rs`, `tests/config/src/fixtures/metered.rs`,
     `tests/config/src/fixtures/memory.rs`
   - **Checked:** today, at your branch's base. **Paid when** stays. **Measure:** the same command, `: 3`.
   - **What:** only the item's third bullet stays, as a paragraph. **Why it matters** stays.
   - **What pays it back:** each fixture moves into its suite as a plain module, `suite/memory/`, or goes with what
     hearth's memory migration does to these suites. They wait for it: it holds both files.
8. `cd hearth && python3 -I -B scripts/old_forms.py --write`.
9. Lists compared, focused and fuzzy tests of the changed packages, `cargo fmt`, clippy on every package changed,
   `lints.py`, `citations.py --all`. Commit and push.

## Report

The lists' comparison, the three `#[path]` lines left, and anything `citations.py` found. No test here pins a new rule:
moves only, so no bite. Net lines.
