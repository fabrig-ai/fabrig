+++
id = "map-quiet"
part = "manual-map"
kind = "migration"
needs = []
files = ["temper/crates/temper-protocol-store/src/**", "temper/crates/temper-engine-domain-forge/src/**", "temper/crates/temper-engine-domain-forge-change/src/**", "temper/crates/temper-engine-domain-forge-landing/src/**", "temper/crates/temper-client-tools/src/**", "temper/crates/temper-box/src/**", "temper/crates/temper-settings-form/src/**", "temper/crates/temper-engine-smith/src/**", "temper/crates/temper-worker-protocol-git/src/**", "temper/crates/temper-fake/src/**", "temper/tests/engine/forge/src/translate.rs", "temper/testing/temper-fixtures/src/first_run.rs"]
effort = "medium"
minutes = 45
+++

# temper: `manual_map` said once per crate, the crates no draft is changing

## Goal

Step code takes no closures, so it writes `match` where `clippy::manual_map` asks for `Option::map`. Replace
the per-item `#[expect(clippy::manual_map, ...)]` in the crates below with one crate-root expectation in
jig's wording. The rest of temper's sites are `map-busy`'s, a later task, because other drafts are changing
those files. temper's item stays open until then: don't touch
`temper/docs/development/debt/D011-manual-map-expectations.md`.

Acceptance, for each crate of the table in `## Steps`:

- `rg -U --type rust 'clippy::manual_map' CRATE` finds one attribute, at the crate's root, reading exactly
  `reason = "step code matches optional fields without closures"`, or, in the two worlds of the table,
  nothing at all.
- `heavy cargo clippy -p PKG --all-targets -- -D warnings` passes for each package you changed.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/development/debt/D011-manual-map-expectations.md`
- `tools/lints/design.md`, 6
- `skein/docs/foundation/programming-model.md`, 10.3
- `jig/crates/jig-domain-accounts/src/lib.rs` for the attribute's place

## Owns and may change

- **Owns:** the `manual_map` attributes in the files of this spec's header, and the crate roots there.
- **May change:** in `temper/tests/engine/forge/src/translate.rs` and `temper/testing/temper-fixtures/src/first_run.rs`,
  only the matches under those attributes. Nothing else in those two crates: `temper-profiles` moves their
  lints, in parallel.
- **Options:**
  - Where an attribute lists other lints beside `clippy::manual_map`, keep them and its reason, if the reason
    still says why.
  - If a crate's root `#![expect]` is unfulfilled in a target that compiles none of its sites, use
    `#![cfg_attr(test, expect(clippy::manual_map, reason = "..."))]`, with the same reason. The table says
    where to expect that.
  - If the root expectation is unfulfilled in every target, the attribute was stale: delete it, and say so.

## Steps

1. For each crate, put the attribute at the crate root, beside its other inner attributes, and delete the
   item-level ones. `temper-protocol-store/src/records/issues.rs` opens with a module-level `#![expect]`: it
   goes too.

   ```rust
   #![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
   ```

   | Crate | Sites at the base | Form |
   |---|---:|---|
   | `temper/crates/temper-protocol-store` | 36 and the module's | root |
   | `temper/crates/temper-engine-domain-forge` | 18 | root |
   | `temper/crates/temper-client-tools` | 10 | root |
   | `temper/crates/temper-box` | 2 | root |
   | `temper/crates/temper-settings-form` | 2 | root |
   | `temper/crates/temper-engine-domain-forge-change` | 1 | root |
   | `temper/crates/temper-engine-domain-forge-landing` | 1 | root |
   | `temper/crates/temper-fake` | 1 | root |
   | `temper/crates/temper-engine-smith` | 1, in `src/tests.rs` | `cfg_attr(test, ...)` |
   | `temper/crates/temper-worker-protocol-git` | 1, in `src/tests/program.rs` | `cfg_attr(test, ...)` |
   | `temper/tests/engine/forge` | 2, in `src/translate.rs` | no attribute: `Option::map` |
   | `temper/testing/temper-fixtures` | 2, in `src/first_run.rs` | no attribute: `Option::map` |

2. The two worlds are ordinary Rust, where closures are in (`skein/docs/foundation/programming-model.md`,
   10.2): take clippy's suggestion for each match and drop the attribute. The value each returns stays the
   same.
3. Fmt, old forms, citations. Clippy as above. Then the focused tests of the packages you changed:
   `heavy cargo nextest run -p PKG` for `temper-fixtures` and the world's package, whose code changed; an
   attribute changes no behaviour, so the others take clippy alone.
4. Commit and push.

## Report

The report of your implementer text, and: the count of attributes removed and added, by crate; each crate that
took the `cfg_attr` form or an option; the branch's net lines.
