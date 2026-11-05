+++
id = "link-encoders"
part = "link-translate"
kind = "migration"
needs = ["link-loops", "piece-others"]
files = ["jig/crates/jig-client-protocol-link/src/translate/settings.rs", "jig/crates/jig-client-protocol-link/src/translate/settings_proposals.rs", "jig/crates/jig-client-protocol-link/src/translate/answers.rs", "jig/crates/jig-client-protocol-link/src/tests.rs", "jig/crates/jig-client-protocol-link/src/tests/**", "jig/testing/jig-client-scripted-engine/src/lib.rs", "jig/testing/jig-client-scripted-engine/src/codec/**", "jig/tests/conformance/tests/suite/client_settings.rs", "jig/tests/conformance/tests/suite/client_settings_proposals.rs", "jig/tests/conformance/Cargo.toml", "jig/docs/development/debt/D008-link-translate-split-by-number.md"]
effort = "high"
minutes = 60
+++

# jig: the link's production code holds no test-only encoders

## Goal

The last step of `jig/docs/development/debt/D008-link-translate-split-by-number.md`: **Paid when** holds, and the item goes. 39 `wire_`, `encode_` and `restore_read_` functions in `translate/` (426 lines) only tests reach. They move under the link's `src/tests/`, and conformance reads the scripted engine's own projections.

Acceptance:

- `rg -n 'fn (NAME)\b' jig/crates/jig-client-protocol-link/src/translate`, for each name below, finds nothing outside `#[cfg(test)]`. The item's count of 85 is wrong, and this list is the check: your report gives the count of `wire_` and `encode_` functions left in production, each reached from a production path.
- `find jig -name '*_part[0-9]*.rs' | wc -l` reads 0. `rg -o 'piece::split\(&mut reader\)' jig/crates jig/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**' | wc -l` reads at most 5, each named in `link-loops`'s or `piece-others`' report.
- `heavy cargo nextest run -p jig-client-protocol-link -p jig-conformance-world -p jig-client-scripted-engine` passes, assertions unchanged, with the same tests. `heavy cargo check -p jig-client-protocol-link` compiles with no test code. `.config/gate/wasm32.txt` lists the link: `heavy cargo check -p jig-client-protocol-link --target wasm32-unknown-unknown` too.
- Production falls by about 425 lines (at most -350), and test lines rise by no more. The part's whole net is near -800: your report gives it. The item is deleted.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `jig/docs/development/debt/D008-link-translate-split-by-number.md`
- `jig/docs/design/client/domain/testing.md`, 4 (the scripted engine's wire projections)

## Owns and may change

- **Owns:** the files in `files`.
- **The 39, at this plan's base.** A name a production path reaches stays, and your report names it.
  - `translate/settings.rs`: `encode_revision_item_pieces`, `restore_read_revision`, `restore_read_revision_answer`, `restore_read_revisions`, `restore_read_setting`, `wire_held_revision`, `wire_malformed_settings`, `wire_outcome_settings_changed`, `wire_read_revision_answer`, `wire_refusal_full`, `wire_refusal_stale`, `wire_revision_answer`, `wire_revision_decider`, `wire_revision_document`, `wire_revision_edit`, `wire_revision_forgotten`, `wire_revision_head`, `wire_revision_item`, `wire_revision_party`, `wire_revision_replaced`, `wire_revision_row`, `wire_revisions_page`, `wire_setting_document`, `wire_setting_fault`, `wire_setting_follows`, `wire_setting_line`, `wire_settings_document`, `wire_settings_proposal`.
  - `translate/settings_proposals.rs`: `encode_heads`, `encode_review_pieces`, `wire_origin_revision`, `wire_owner_proposal_action_settings`, `wire_owner_retained_action_settings`, `wire_review_item`, `wire_review_value`, `wire_setting_origin`, `wire_settings_review`.
  - `translate/answers.rs`: `wire_outcome_restored`, `wire_refusal_forgotten`.
- **Their home:** `src/tests/encoders.rs` (a child of `tests`, `#[cfg(test)]` by being there), with a sub-file per family if it reads better. Link tests (`tests/settings.rs`, `settings_proposals.rs`, `settings_restore.rs`, 74 lines) read them there: spelling only. A helper they call that production also calls stays, `pub(crate)` if it must open.
- **`translate/answers.rs`' inline test** (`original_nonzero_recipient_survives_wire_decoding_and_contract_translation`, about lines 637 to 718) moves to `src/tests/answers.rs` or the like, keeping its name, and tiers filter by name: check `.config/` if the module path changes.
- **Conformance reads 7 names** (`wire_malformed_settings`, `wire_settings_document`, `wire_setting_document`, `wire_revision_answer`, `wire_revisions_page`, `wire_owner_proposal_action_settings`, `wire_owner_retained_action_settings`), and a test crate can't read another crate's `#[cfg(test)]`.
  `jig-client-scripted-engine/src/codec/values/settings.rs` already defines most of these, independently (`pub(in crate::codec)`), by the design's own word. Publish those conformance needs (`pub use` from the crate's root, or a `pub mod wire`) and read them there; add the dependency to `jig/tests/conformance/Cargo.toml`.
  - If the engine lacks a name, or its output differs so that a conformance assertion would change, that name moves to `jig/tests/conformance/tests/suite/` as a helper, from the link's text, and your report says which and why. **No conformance assertion changes.**
- **The link's tests keep their own copy** under `src/tests/`, though they duplicate the engine's. A dev-dependency from the link on the engine would remove them, and is worth a look: it isn't this item's. Your report says what you saw.
- **`wire_setting_value` and `wire_change_settings` stay:** they're production's, and `jig-protocol-engine-clients`' and temper's tests call them.
- **The item's text corrects nothing,** being deleted; your report gives the true counts for the record. Any citation of it goes (`D007`, and `D025` if it names it), by the citations check.

## Steps

1. `src/tests/encoders.rs`, by moves, with the inline test.
2. Conformance on the engine's.
3. Delete the item.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-client-protocol-link -p jig-client-scripted-engine -p jig-conformance-world --all-targets -- -D warnings`; the tests above; `heavy cargo check --tests -p jig-protocol-engine-clients -p jig-client-world -p temper-client-tools -p temper-client-domain`. The citations check. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items, the true counts of `wire_` and `encode_` functions before and after, what conformance reads and from where, what you saw of the two copies, and the part's net production against the plan's base.
