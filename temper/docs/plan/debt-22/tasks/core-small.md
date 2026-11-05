+++
id = "core-small"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/settings/models.rs", "jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/acts/objects/task.rs", "jig/crates/jig-protocol-store/src/codec/settings.rs"]
effort = "medium"
minutes = 30
+++

# jig-domain-core: `settings::remap` takes its `moved`, and `control` takes its row

## Goal

Findings 5 and 6 (second sentence) of `temper/docs/plan/debt-22/inputs/debt20-review.md`. A decode's loop builds one `moved` where it built one per value, and `control` loses an `.expect`. Behaviour stays.

Acceptance:

- `jig-domain-core`'s `settings` exports `pub fn moves(kept: &[ModelName], names: &[ModelName]) -> [u64; 64]`, and `pub fn remap(value: &mut Value, moved: &[u64; 64], kept: &[ModelName], names: &[ModelName])`.
- `jig-protocol-store/src/codec/settings.rs`'s two loops call `settings::moves` once, before the loop.
- `acts/objects/task.rs`'s `control` takes `row` from its caller `admit`, which holds it, and has no `.expect`.
- `heavy cargo nextest run -p jig-domain-core --lib -E 'test(remap)'` and `heavy cargo nextest run -p jig-protocol-store --lib` pass, `remap_template_choices_follow_model_and_effort_names_and_mark_departures`, `remap_named_masks_follow_names_and_drop_departures_while_all_stays` and `live_settings_remap_efforts_by_name_and_refuse_missing_efforts` among them, unchanged but for their calls. `heavy cargo nextest run -p jig-core-world` passes.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/inputs/debt20-review.md`, findings 5, 6
- `jig/docs/design/protocol/store.md`, 5.1

## Owns and may change

- **Owns:** the files in the header. `settings/tests.rs`, only the two `remap` calls: `core-tables` holds another test there, and the two don't meet.
- **Readers of `settings::remap`:** the five calls in `codec/settings.rs` and the two in `settings/tests.rs`. `remap` is public, and this plan's request names this change; a call's spelling is all that moves.
- **Stays:** `codec/tasks/models.rs`'s `moves` and `codec/people.rs`'s `model_moves`: they read wire names, a different type, and no finding names them.

## Steps

1. `moves`, split out of `remap`; the calls. `control`'s row.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-core -p jig-protocol-store --all-targets -- -D warnings`; the tests above; `heavy cargo check -p jig-store-world --tests`; the citations check. Commit and push.

## Report

The common note's items. No new test: this moves code, and the existing tests hold it.
