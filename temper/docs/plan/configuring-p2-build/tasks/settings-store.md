+++
id = "settings-store"
part = "restore"
kind = "feature"
needs = ["policy-surface", "restore-surface"]
files = ["jig/crates/jig-protocol-store/schema/settings-v1.schema", "jig/crates/jig-protocol-store/src/generated/settings_v1.rs", "jig/crates/jig-protocol-store/schema/people-v14.schema", "jig/crates/jig-protocol-store/src/generated/people_v14.rs", "jig/crates/jig-protocol-store/src/generated/layouts.rs", "jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/src/codec/people.rs", "jig/crates/jig-protocol-store/src/codec/people/**", "jig/crates/jig-protocol-store/src/keys/settings.rs", "jig/crates/jig-protocol-store/golden/settings/**", "jig/crates/jig-protocol-store/golden/people/**", "jig/crates/jig-protocol-store/src/tests/**"]
effort = "high"
minutes = 120
+++

# settings-store: the store keeps a project's policy item, what each item replaced, and a restore's answer

## Goal

jig's store keeps, in place in `settings-v1` and `people-v14`, what the core now holds of the
two parts: a project's policy as a setting of a revision's and a proposal's items, each
revision item's `replaced` (contract B2) and each revision application item's state and size
replaced, and a restore's stored answer (its ask, its outcome and its *forgotten* refusal,
B4). A format change in place, as M3's `apps-store` made one (commit f6717e2103).

Acceptance:

- `settings::Revision` with `RevisionItem`s of each `Replaced` state (jig's four kinds' values
  and a project's policy), `application::RevisionRecorded` of each `application::Replaced`
  state, `Setting::ProjectPolicy { project }` in a revision's and a proposal's items, and the
  keys `Key::Default`/`Key::Changed` of a project's policy round-trip through the store's
  codecs at their bounds; a replaced value follows model names after a catalogue reorder as a
  set value does.
- `people::Ask::Restore`, `Outcome::Restored`, `Refusal::Forgotten` and a settings refusal
  naming `Setting::ProjectPolicy` round-trip as a stored answer; a reason or a `left_out` one
  past its bound is refused `Bound`.
- No `unreachable!("a project's policy: policy-store")` and no `unreachable!("a restore:
  settings-store")` is left in `jig-protocol-store`, nor restore-surface's placeholder line
  `// The store keeps no replaced state until settings-store.` (Step 6 says which other arm
  this task also replaces.)
- The crate's tests, the store-codecs world (drift, layouts, goldens' memory and fuzz), the
  store world and the core world's settings stories pass; the goldens remade are those of the
  changed records only.
- No `docs/design/` edit: the engine status's 17.10 line, which `policy-change` edits beside
  it at the same time, is `restore-request`'s to update (a stated change by the lead).

## Inputs

- `jig/docs/design/domain/engine.md`, 17.2 (a project's own policy: "the revision's record
  keeps the item as any other, its value whole and the project's policy it replaced"), 17.5
  (**History**), 17.7 (**What the store keeps**), 17.10 (**Each item records what it
  replaced**, **Bounded**).
- `jig/docs/design/protocol/store.md`, 4.4 (the settings rows), 5.3 (no version moves before
  the first release: changes in place). 4.4 needs nothing: the live row keeps "pending
  proposals' sets" and the history row already says items keep "the values they set and what
  each replaced". Leave `store.md` unedited (it is read by tests:
  `jig-protocol-store/src/tests/words.rs` reads its refusal texts only).
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B1, B2 and B4 (binding names);
  `notes/common.md`.
- `temper/docs/plan/configuring-p2-build/tasks/restore-surface.md` (what it leaves in this
  crate) and `policy-surface.md` (its arms table).
- As the pattern: `temper/docs/plan/reference-m3/tasks/apps-store.md` and its answers; `git
  show --stat f6717e2103` for what such a change touches.
- Old code, from jig's `configuring-revisions/replaced` (unreachable from your slot, quoted):
  in `settings-v1`, `enum Replaced { value: boxed Set, default, removed, not_offered }`,
  `record RevisionItem { setting: Setting edit: Edit replaced: Replaced }`.

## Owns and may change

- Owned: the `files` above. In `src/tests/**`: the new file `src/tests/settings/replaced.rs`
  and its `mod replaced;` line beside `mod applications;` in `src/tests/settings.rs`; new tests
  in `src/tests/codec/settings_proposals.rs`; fixture literals of `Revision`, `RevisionItem` and
  `RevisionRecorded` anywhere in the crate's tests (restore-surface gave them
  `Replaced::NotOffered`: keep or vary them as a test needs).
- Neighbour files, each with the edit allowed:
  - `jig/crates/jig-protocol-config/src/domain.rs`, `people_bound`: one row per new people-v14
    limit path (`AskRestoreReason` as `b"people.words"`, `OutcomeRestoredLeftOut` naming its
    bound as step 3 sets it), in the table's one-line style; nothing else.
  - `jig/tests/store/**` and `jig/tests/store-codecs/**`: a fixture's new field or arm; a golden
    of `jig/tests/store/golden/**` that fails only because a changed record's bytes moved,
    remade with that world's own golden tooling and named in the report.
  - Compiler-forced arms and fields in any crate the new schema types reach (generated
    `Path` variants, `Limits` fields) are always allowed.
- **Versions:** `settings-v1` and `people-v14` change in place, their generated code, layouts
  and goldens made again with the repository's generator (step 7). If a test or a layout check
  requires a version to move, move only that family's version by one and its format row's
  expectation, and say so in the report.
- Options, pre-approved:
  - If restore-surface's arms in this crate name another task than `settings-store` for
    `people::Ask::Restore` (its table says `restore-request` where a codec cannot refuse),
    replace them here too: the stored answer keeps the ask, so this task's `people-v14` carries
    it and `restore-request` changes no schema. Say so in the report.
  - If the store's value bound (`src/sizes.rs`, `largest`, which takes
    `core::settings::revision_bound`) or a sizes test fails because a revision row with replaced
    values passes it: stop and ask (the price is restore-surface's `revision_bound`). A test's
    fixed byte price that fails only because a record grew: correct it, its comment naming the
    record and field.
  - If the codegen grammar refuses a shape below (a variant holds one record or nothing), keep
    the names and wrap the payload in the nearest existing record, and say so.

## Steps

1. `schema/settings-v1.schema`, in place:
   ```
   record ProjectPolicySetting { project: u32 }
   enum Setting { charter: CharterNumber, template: TemplateNumber, policy: PolicyNumber, rules, application: ApplicationSetting, project_policy: ProjectPolicySetting }
   enum Replaced { value: Set, default, removed, not_offered }
   record RevisionItem { setting: Setting edit: Edit replaced: Replaced }
   enum SizedReplaced { changed: SizedSet, default, removed, not_offered }
   record RevisionRecorded { kind: u32 scope: u32 number: u32 edit: SizedEdit replaced: SizedReplaced }
   versioned record Revision { models: list 64 ModelName head: Head items: list 128 RevisionItem applications: list 128 RevisionRecorded }
   ```
   (`Replaced` and `RevisionItem` after `Item`; `SizedReplaced` and `RevisionRecorded` after
   `Recorded`.) `Item`, `Recorded` and `Proposal` stay as they are: a proposal's items are a
   party's `Item`s and its applications `Recorded` (B2). The limits' field names stay
   (`revision_items`, `revision_applications`).
2. `codec/settings.rs`:
   - `encode_setting`/`decode_setting`: the `project_policy` arm both ways (replaces
     policy-surface's arm).
   - `encode_replaced`/`decode_replaced` (core `settings::Replaced` <-> `wire::Replaced`,
     `Value(v)` through `encode_value`/`decode_value` in a `wire::Set`); `encode_revision_item`/
     `decode_revision_item`; `encode_revision_recorded`/`decode_revision_recorded`
     (`application::Replaced::Changed { size }` <-> `SizedReplaced::Changed(SizedSet { size })`).
     `encode_revision` and `decode_revision` use them in place of restore-surface's drop and
     `NotOffered` fill, and the placeholder line goes. `decode_revision`'s model remap applies
     `remap_value` to a `Replaced::Value` as to an `Edit::Set` value.
   - `encode`'s `Record::Application` check: `Setting::ProjectPolicy { .. }` joins jig's kinds'
     `return Err(bound())`. `decode`'s `Key::Changed(setting)`: `ProjectPolicy` decodes as jig's
     kinds do (`Record::Changed`): the core writes no row under it (B1), the codec stays total.
3. `schema/people-v14.schema`, in place, each new arm last of its enum:
   ```
   record ProjectPolicySetting { project: u32 }
   record AskRestore { revision: u32 reason: bytes 8192 }
   record OutcomeRestored { proposal: u64 to: u32 left_out: list 256 Setting }
   record RefusalForgotten { oldest: u32 }
   ```
   with the arms `Setting.project_policy: ProjectPolicySetting`, `Ask.restore: AskRestore`,
   `Outcome.restored: OutcomeRestored` and `Refusal.forgotten: RefusalForgotten`, each declared
   where the enum's existing records sit (`OutcomeRestored` after `Setting`).
   `codec/people.rs`'s `derived_limits`: `ask_restore_reason: people.words`;
   `outcome_restored_left_out: jig_domain_tasks::SETTINGS_ITEMS.checked_add(1)?`, with the line
   `// A restore leaves out at most a proposal's application items and one project's policy
   (engine.md, 17.10, Bounded).` `codec/people/asks.rs` (`encode_ask`/`decode_ask`),
   `codec/people/outcomes.rs` (`Outcome::Restored`, its `left_out` through `encode_setting`/
   `decode_setting`), `codec/people/outcomes/facts.rs` (`Refusal::Forgotten`; the
   `project_policy` arm of `encode_setting`/`decode_setting`).
4. `keys/settings.rs`: a project's policy where keys name settings, tag 5 after an
   application's 4: `setting_write` writes `tag(5)` then `u64(project)`; `setting_read` reads
   tag 5 as `Setting::ProjectPolicy { project }` (the `u32::try_from` as the others). No row is
   written under it today (B1); the key codec stays total.
5. Tests:
   - `src/tests/settings/replaced.rs`, in the style of `applications.rs` beside it (its
     `round_trip` and `head` helpers, copied or shared):
     `revision_items_round_trip_every_replaced_state` (for each of `charter()`, `template()`,
     `policy()`, `rules()` of `src/tests/settings.rs`, items with `Replaced::Value(v)`,
     `Default`, `Removed`, `NotOffered`, held and history; a truncated row refused);
     `a_project_policy_item_round_trips_in_a_revision_and_a_proposal`
     (`Setting::ProjectPolicy { project: u32::MAX }`, `Edit::Set(Value::Policy(..))`,
     `Replaced::Value(Value::Policy(..))`; and in a `Proposal`'s items);
     `revision_application_items_round_trip_every_replaced_state` (`Changed { size: 0 }`,
     `Changed { size: u32::MAX }`, `Default`, `Removed`, `NotOffered`, at 0 and
     `SETTINGS_ITEMS` items; one past refused `Bound`);
     `replaced_values_follow_model_names_after_a_catalogue_reorder` (as
     `frozen_model_masks_and_template_choices_follow_names_after_a_catalogue_reorder`, a
     revision whose item replaced `Value::Rules` with `Models::Named(1)` reads back `Named(2)`);
     `a_project_policy_key_round_trips_and_sorts_after_application_settings` (`Key::Default` and
     `Key::Changed` of `ProjectPolicy { project }` for 0, 1, `u32::MAX`, after the last
     `Application` key of each subfamily, as `application_keys_sort_after_rules_...` checks).
   - `src/tests/codec/settings_proposals.rs`:
     `restore_asks_outcomes_and_forgotten_refusals_round_trip_at_their_bounds` (`ask_round_trip`
     of `Ask::Restore { revision: 0 | u32::MAX, reason: 0 | people.words bytes }`; a stored
     answer with `Outcome::Restored { proposal: u64::MAX, to: u32::MAX, left_out }` at 0 and at
     `outcome_restored_left_out` settings mixing `ProjectPolicy` and `Application`; one with
     `Outcome::Refused(Refusal::Forgotten { oldest: u32::MAX })`; one with
     `Malformed::Settings { setting: ProjectPolicy { .. }, fault: Shape }`);
     `a_restore_reason_and_left_out_one_past_their_bounds_are_refused` (as
     `judged_refusal_words_use_the_people_words_bound`).
   - Extend `each_settings_record_round_trips_its_whole_domain_value` only if it needs a
     `replaced` other than its fixture's; no other test's meaning moves.
6. Arms: every `unreachable!("a project's policy: policy-store")` (policy-surface) and
   `unreachable!("a restore: settings-store")` (restore-surface) in `jig-protocol-store` gets
   its body above; `rg 'policy-store|settings-store' jig/crates/jig-protocol-store` finds none.
7. Regenerate, then the narrow tests, each checking its count:
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world --test drift`
   (writes the generated code, the goldens and `layouts.rs`; `git status` shows only this
   task's families moved), then without the variable;
   `heavy cargo nextest run -p jig-protocol-store --lib -E 'test(/settings::replaced::/)'`, then
   `-E 'test(/settings|codec::settings_proposals|keys|format|settings_layout/)'`, then
   `heavy cargo nextest run -p jig-protocol-store --lib`;
   `heavy cargo nextest run -p jig-store-codecs-world` (memory, fuzz);
   `heavy cargo nextest run -p jig-store-world` (whole, about 176; its goldens per the
   neighbour rule).
   A change of meaning (stored revisions now keep their replaced state across a restart), so
   the core world's readers run: `heavy cargo nextest run -p jig-core-world --test suite -E
   'test(/^(start_settings|settings_reads|settings_proposals|settings_proposal_edges|application_settings|restart)::/)'`.
8. No status edit (the acceptance's last line). Then `cargo fmt`; `heavy cargo clippy -p jig-protocol-store
   -p jig-protocol-config --all-targets -- -D warnings`; `heavy cargo check --workspace
   --all-targets`; `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and
   push.

## Report

The schemas changed and whether a version moved; the goldens written and removed, by name; the
arms replaced, by file, and whether restore-surface's `Ask::Restore` arm here named another
task; any test expectation corrected and why; whether `jig/tests/store` goldens moved.

## Answers

2026-11-03. **Q: the prescribed `Revision` (128 `RevisionItem`s, each an edit and a replaced
value whole) exceeds the generator's `u32` encoded worst case: revise the representation or
lower ceilings?** Revise the representation; lower no ceiling. A revision's items are bounded
by their bytes in all (`core::settings::revision_bound`, engine.md 17.10, **Bounded**), not by
their count times their largest item, which is the case skein's codec gives one form for
(`skein/docs/design/codec.md`, 3, "A sum crosses as one piece"), the form `people-v14` already
uses (`Pattern.segments`, `Authority.grants`, `PolicyValue.permissions`). This replaces step
1's `Revision` line:

```
versioned record Revision { models: list 64 ModelName head: Head items: bytes 3221225472 pieces RevisionItem applications: list 128 RevisionRecorded }
```

- `RevisionItem`, `Replaced`, `SizedReplaced`, `RevisionRecorded` stay as step 1 has them,
  declared before `Revision`. `Item`, `Recorded` and `Proposal` stay unchanged (its worst
  case fits).
- `codec/settings.rs`: the items are written and split with the piece helpers, each item
  after its `u32` length, as `codec/people/pieces.rs` does; widen that module's helpers'
  visibility (`pub(in crate::codec)`) so the settings codec shares them rather than copying
  them. A piece that does not end exactly at its end is malformed; more items than
  `settings::capacity(domain) * 2` is `Bound`, as the list was.
- `derived_limits`: the items' field limit (whatever the generator names it) is the byte sum
  derived from `core::settings::revision_bound(domain)` with `piece_bound` (owned size
  `size_of::<settings::RevisionItem>()`, item `wire::RevisionItem::encoded_bound(..)`), as
  `derive_shared` does; `None` past the ceiling, as the other derivations. `proposal_items`
  becomes `settings::capacity(domain)?.checked_mul(2)?` itself, no longer copied from the
  revision's limit.
- Tests: `revision_items_round_trip_every_replaced_state` also covers a revision at the
  derived sum and one byte past it (`Bound`), and one item past the count (`Bound`).
- The step's stop rule stands: if `revision_bound` (restore-surface's price) is below what a
  revision with replaced values encodes to, stop and ask; do not raise it here.
- No skein change, no ceiling of another record moves, no version moves.
