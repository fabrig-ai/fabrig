+++
id = "map-busy"
part = "manual-map"
kind = "migration"
needs = ["map-quiet", "temper-clippy"]
files = ["temper/crates/temper-service/src/**", "temper/crates/temper-engine-domain/src/**", "temper/crates/temper-client-domain/src/**", "temper/crates/temper-client-view/src/**", "temper/crates/temper-worker-domain/src/**", "temper/tools/gallery/src/worlds.rs"]
effort = "medium"
minutes = 45
+++

# temper: `manual_map` said once per crate, the crates other drafts are changing

## Goal

Finish what `map-quiet` began, in the crates other drafts touch, which is why this comes late: it cuts the chance of
meeting them. After this D011's **Paid when** holds across temper. `lint-gate` deletes the item, with the others'.

Acceptance:

- D011's measure reads nothing or 0:
  `rg -U -c --type rust '#\[expect\(\s*clippy::manual_map' temper | awk -F: '{s+=$2} END {print s+0}'`.
- `rg -U --type rust 'clippy::manual_map' temper` finds only crate roots (`src/lib.rs`, or a crate's `main.rs`), one
  each, all reading `reason = "step code matches optional fields without closures"`.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes for each package you changed, and
  `heavy cargo clippy --target wasm32-unknown-unknown -p temper-client-domain -p temper-client-view -- -D warnings`.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/development/debt/D011-manual-map-expectations.md`
- `tools/lints/design.md`, 6
- `temper/docs/plan/debt-11/tasks/map-quiet.md`, for what was done
- `skein/docs/foundation/programming-model.md`, 10.3

## Owns and may change

- **Owns:** the `manual_map` attributes in `temper-service`, `temper-engine-domain`, `temper-client-domain`,
  `temper-client-view`, `temper-worker-domain`, the crate roots there, and the one match under the attribute in
  `temper/tools/gallery/src/worlds.rs`.
- **May change:** nothing else in those crates.
- **Overlap:** `debt-10`, `serve-without-box` and `look-round-3` are changing these crates, among them
  `temper-service/src/engine.rs`, `system.rs`, `clients.rs`, `temper-engine-domain/src/domain.rs`, `route/forge_route.rs`,
  `temper-client-domain/src/route.rs` and `temper/tools/gallery/src/worlds.rs`. Change attribute lines alone there, and
  don't reformat what surrounds them. New sites a merge brought in go the same way.
- **Options:**
  - Where an attribute lists other lints beside `clippy::manual_map`, keep them and its reason, if the reason still
    says why.
  - If a crate's root `#![expect]` is unfulfilled in a target that compiles none of its sites, use
    `#![cfg_attr(test, expect(clippy::manual_map, reason = "..."))]`, with the same reason.
  - If the root expectation is unfulfilled in every target, the attribute was stale: delete it, and say so.
  - If an attribute on `manual_map` is left in a crate this spec doesn't name, because a merge brought it, do the same
    there: it is the same one-line change.

## Steps

1. For each crate, put the attribute at the crate root, beside its other inner attributes, and delete the item-level
   ones:

   ```rust
   #![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
   ```

   | Crate | Sites at the base |
   |---|---:|
   | `temper-service` | 22 |
   | `temper-client-domain` | 10 |
   | `temper-engine-domain` | 8 |
   | `temper-client-view` | 6 |
   | `temper-worker-domain` | 3 |

2. `temper/tools/gallery` is ordinary Rust: take clippy's suggestion for the match in `src/worlds.rs` and drop the
   attribute. The value it returns stays the same.
3. Fmt, old forms, citations. Clippy as above. Then `heavy cargo nextest run -p temper-client-gallery`, whose code
   changed; an attribute changes no behaviour, so the others take clippy alone.
4. Commit and push.

## Report

The report of your implementer text, and: the measure's reading; the count of attributes removed and added, by crate;
each crate that took the `cfg_attr` form or an option; the branch's net lines.
