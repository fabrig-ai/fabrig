+++
id = "settings-wire"
part = "restore"
kind = "feature"
needs = ["policy-surface", "restore-surface"]
files = ["jig/crates/jig-client-protocol-wire/schema/client-v19.schema", "jig/crates/jig-client-protocol-wire/schema/client-v19-part1.schema", "jig/crates/jig-client-protocol-wire/src/generated/v19.rs", "jig/crates/jig-client-protocol-wire/golden/**", "jig/crates/jig-protocol-engine-clients/src/translate/settings.rs", "jig/crates/jig-protocol-engine-clients/src/translate/settings/change.rs", "jig/crates/jig-protocol-engine-clients/src/translate/refusals.rs", "jig/crates/jig-protocol-engine-clients/src/translate/asks.rs", "jig/crates/jig-protocol-engine-clients/src/translate/answers.rs", "jig/crates/jig-protocol-engine-clients/src/tests/settings_restore.rs", "jig/crates/jig-protocol-engine-clients/src/tests.rs", "jig/crates/jig-client-protocol-link/src/translate/settings.rs", "jig/crates/jig-client-protocol-link/src/translate/settings_proposals.rs", "jig/crates/jig-client-protocol-link/src/translate/asks.rs", "jig/crates/jig-client-protocol-link/src/translate/answers.rs", "jig/crates/jig-client-protocol-link/src/translate/store.rs", "jig/crates/jig-client-protocol-link/src/tests/settings_restore.rs", "jig/crates/jig-client-protocol-link/src/tests.rs", "jig/crates/jig-client-domain/src/contract/settings.rs", "jig/crates/jig-client-domain/src/contract/asks.rs", "jig/crates/jig-client-domain/src/contract/refusals.rs", "jig/crates/jig-client-view/src/words/**", "jig/crates/jig-client/src/words.rs", "jig/crates/jig-client/src/words/english.rs", "jig/testing/jig-client-scripted-engine/src/**", "jig/testing/jig-client-test-records/src/settings*.rs", "jig/docs/design/client/domain/contract.status.md"]
effort = "high"
minutes = 170
+++

# settings-wire: a project's policy, what each item replaced, and a restore on jig's client wire and contract

## Goal

jig's client wire `client-v19` (in place, as M3's `apps-wire`) and `jig-client-domain`'s
contract carry contract B5: a project's policy as a setting of a change's, a proposal's and a
review's items; each revision item's `replaced`; `Restore`, its outcome `Restored` and its
refusal `Forgotten`. The engine's translation (`jig-protocol-engine-clients`) and the client's
(`jig-client-protocol-link`) carry each both ways; the view and jig's testing crates speak them.

Acceptance:

- Each new wire arm and record round-trips in its own crate's tests: the engine side from the
  people and core vocabulary to the wire and back where the engine reads it (`Ask`, a change's
  items), the link from the contract to the wire and back (step 7).
- A revision read's items carry `replaced`, each value within the single read's bound or with
  its size as too large, as the item's set value is; an application's item its replaced size.
- No `unreachable!("a project's policy: policy-wire")` and no `unreachable!("a restore:
  settings-wire")` is left in jig (`rg 'policy-wire|settings-wire' jig` finds none).
- The client-wire world's drift, settings and fuzz tests pass, the goldens remade for the
  changed records only; the focused tests of every crate changed pass; `heavy cargo check
  --workspace --all-targets` is clean.
- The **Not yet** texts named in step 9 are gone from
  `jig/docs/design/client/domain/contract.status.md`.

## Inputs

- `jig/docs/design/client/domain/contract.md`, 3.2 (`ChangeSettings`' items, `Restore`'s row),
  5.1 (a pending proposal's review, its item of a project's own policy), 6 (`revision{n}`'s
  items and what each replaced).
- `jig/docs/design/domain/engine.md`, 17.2, 17.6 (**Reviewed against its own revision**),
  17.10 (**Each item records what it replaced**, **`Restore { revision, reason }`**).
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B1, B2, B4 and B5 (binding
  names); `notes/common.md`.
- `temper/docs/plan/configuring-p2-build/tasks/policy-surface.md` and `restore-surface.md`
  (their arms tables: what they leave in these crates).
- As the pattern: `temper/docs/plan/reference-m3/tasks/apps-wire.md` (the same mechanism for an
  application's setting; read its steps, do not redo them).

## Owns and may change

- Owned: the `files` above. `mod settings_restore;` beside `mod settings;` in each of
  `jig-protocol-engine-clients/src/tests.rs` and `jig-client-protocol-link/src/tests.rs`.
- Neighbour files, each with the edit allowed:
  - Every other match the new wire or contract arms reach in jig (`jig-protocol-engine-clients`'
    `request.rs` and `translate/llm.rs`; the link's `translate/compound.rs` and
    `limits.rs`; `jig-client-view/src/state/**`; `jig-client/src/thread/model.rs`;
    `jig/testing/jig-conformance`, `jig-client-test-kit`, `jig-test-domain`;
    `jig/examples/ops`): `Restore` takes `MoveToNewest`'s arm wherever a match classifies
    asks (the deployment's scope, no models, no carried value, not a compound operation);
    `Restored` takes `MovedToNewest`'s; `Forgotten` takes `Stale`'s, but in the view's
    `recovery` it sets `change_ask` (a later revision is restored instead, engine.md 17.10); a
    `Setting::ProjectPolicy` takes `Setting::Policy`'s. The view's outcome `route` gives
    `Restored` `Route::Proposed` (a restore made a proposal).
  - `jig/tests/client-wire/**`: a fixture's new arm or field; a golden of that world that fails
    only because a changed record's bytes moved, remade with its own tooling and named.
  - In temper, **compiler-forced arms and fields only** (the stated cross-tree allowance), each
    edited in that arm or field alone: a match on `jig_client_domain::contract::Setting` gets
    `Setting::ProjectPolicy { .. } => unreachable!("a project's policy: policy-ctl")`; on
    `contract::Ask`, `Outcome` or `Refusal`, `unreachable!("a restore: restore-ctl")`; a
    `contract::RevisionItem` literal gets `replaced: RevisionReplaced::NotOffered`.
- Versions: `client-v19` changes in place (the client wire's rules before the first release,
  as `apps-wire` did); if a drift or conformance check requires the version to move, stop and
  ask.
- Options, pre-approved:
  - If a test or limit bounds a revision document (`revision_document_items`) at a profile's
    bounds and fails only because replaced values were added, stop and ask with the sizes;
    a test's fixed byte price that fails only because a record grew: correct it, its comment
    naming the record and field.
  - If the codegen grammar refuses a shape below (a variant holds one record or nothing), keep
    the names, wrap the payload in the nearest existing record, and say so.
  - If `jig-client-view` has no phrase shape for a word below, add it in the phrase table's
    own pattern (an id, its holes, its English) and list it.

## Steps

1. Wire schema, in place (`client-v19.schema` where `Setting` is declared; the rest in
   `client-v19-part1.schema` beside their siblings), each new arm last of its enum:
   ```
   record ProjectPolicySetting { project: u32 }
   enum Setting { charter: SettingNumber, template: SettingNumber, policy: SettingNumber, rules, application: ApplicationSetting, project_policy: ProjectPolicySetting }
   enum RevisionReplaced { value: SettingSet, too_large: Size, default, removed, not_offered, application: ApplicationSize }
   record RevisionItem { setting: Setting edit: RevisionEdit replaced: RevisionReplaced }
   record Restore { revision: u32 reason: text 65536 }
   record OutcomeRestored { proposal: u64 to: u32 left_out: list 256 Setting }
   record RefusalForgotten { oldest: u32 }
   ```
   with `Ask.restore: Restore`, `Outcome.restored: OutcomeRestored`, `Refusal.forgotten:
   RefusalForgotten`. B5's `too_large: u32` and `application: u32` are the schema's existing
   `Size` and `ApplicationSize` records (a variant holds a record); the reason is `text 65536`
   as `Propose`'s is. A project's policy in a review needs no new record: its `ReviewItem` has
   `now` and `would_be` as `ReviewValue::Value` of the `policy` arm (or `TooLarge`), its
   `origin` `not_offered` (it has no default and no change) and no `application`.
2. Regenerate: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world --test
   suite -E 'test(/^drift::/)'`, then without the variable; `git status` shows only the client
   family moved.
3. Contract, `jig-client-domain`:
   - `settings.rs`: `Setting::ProjectPolicy { project: u32 }`, documented "A project's own
     policy, an item of its scope (domain/engine.md, 17.2)."; `pub enum RevisionReplaced {
     Value(SettingSet), TooLarge(Size), Default, Removed, NotOffered, Application { size: u32 } }`
     documented from engine.md 17.10; `RevisionItem { setting, edit, replaced: RevisionReplaced }`.
   - `asks.rs`: `Ask::Restore { revision: u32, reason: Text }` ("Propose the settings as a
     revision left them, against the newest (domain/engine.md, 17.10)."); `Outcome::Restored
     { proposal: u64, to: u32, left_out: List<Setting> }`. `refusals.rs`: `Refusal::Forgotten
     { oldest: u32 }`. B5's names and fields hold; struct variants, as B5 writes them.
4. Engine side, `jig-protocol-engine-clients`:
   - `translate/settings.rs` `setting` and `name`, `settings/change.rs` `setting`,
     `refusals.rs`' `encode_setting`: the `project_policy` arm both ways (policy-surface's arms
     go). A change's or a proposal's item of a project's policy decodes as a `Setting::Policy`
     item's value does (`setting_value`'s `Policy` arm), into `s::Item { setting:
     ProjectPolicy { project }, edit: Set(Value::Policy(..)) }`.
   - `translate/settings.rs` `revision`: each item's `replaced` as `RevisionReplaced`: a
     `Replaced::Value(v)` through `value` and the same too-large rule as the set value
     (`value_bytes` and `owned_record`); `Default`, `Removed`, `NotOffered` as named; each
     application item's `application::Replaced` (`Changed { size }` as `application`, the
     others as named). The review (`review`) needs no change for a project's policy (step 1).
   - `translate/asks.rs` `decode`: `w::Ask::Restore` into `p::Ask::Restore { revision, reason }`
     (the reason's bytes; the people child bounds it). `translate/answers.rs`:
     `p::Outcome::Restored` into `w::Final::Done` of `w::Outcome::Restored` (its `left_out`
     through `name`), as `MovedToNewest` goes; `refusals.rs`: `p::Refusal::Forgotten { oldest }`
     into `w::Refusal::Forgotten`. restore-surface's `unreachable!("a restore: settings-wire")`
     arms go.
5. Client side, `jig-client-protocol-link`, in the file's `wire_*`/`restore_*` pairs:
   `wire_setting`/`restore_setting`'s arm; `wire_revision_replaced`/`restore_revision_replaced`;
   `wire_revision_item`/`restore_revision_item` with `replaced`; `wire_ask_restore`/
   `restore_ask_restore` (`asks.rs` and `store.rs`' `restore_ask`, its reason checked as
   `Propose`'s is); `restore_outcome_restored` (`answers.rs` `outcome`) and
   `restore_refusal_forgotten` (`answers.rs`' refusal), each with its `wire_` twin as the file
   keeps them; `settings_proposals.rs`' review item for `ProjectPolicy` through the shared
   setting and value translation. In `limits.rs`, `restore_reason` and
   `outcome_restored_left_out` stay at their ceilings, as `propose_reason` does (no line).
6. Words and testing crates:
   - `jig-client-view/src/words/**`: three phrases in the jig phrase table's pattern, ids 7927 to
     7929 (free today): `SettingProjectPolicy` ("project N's policy", one number hole) in
     `malformed`'s setting match; `SettingsRestored` ("Proposed restoring as proposal N to role
     N", two number holes) for `Outcome::Restored`, each left-out setting after it in its own
     setting phrase; `SettingsForgotten` ("That revision is forgotten; the oldest kept is
     revision N", one number hole) for `Refusal::Forgotten`. `jig-client/src/words.rs` and its
     `english.rs`: `RefusalWord::Forgotten` beside `Stale`, the next free number of that table.
   - `jig-client-scripted-engine`: its codec (`codec/values/settings.rs`, `asks.rs`,
     `refusals.rs`, `outcome.rs`) translates the new contract arms to the wire as the link does
     (real arms, no `unreachable!`); `requests.rs` answers `Ask::Restore` by its scripted rule
     as `ChangeSettings` is, without the owner check (any party may ask, contract.md 3.2).
     `jig-client-test-records/src/settings*.rs`: the fixture revision items carry a
     `replaced` of each arm, and one item names `Setting::ProjectPolicy`.
7. Tests:
   - `jig-protocol-engine-clients/src/tests/settings_restore.rs`, in the style of
     `tests/settings.rs` (its `head`, `values` and charter fixtures):
     `a_project_policy_item_enters_the_core_from_a_change_and_a_proposal`;
     `a_project_policy_review_item_carries_the_policy_now_and_as_it_would_be` (a
     `d::SettingsReview` item of `ProjectPolicy` with `now` and `would_be`
     `Some(Value::Policy(..))`, `origin` `None`, reads `Value`, `not_offered`, `Value`; one past
     the document bound reads `TooLarge`);
     `revision_items_carry_what_each_replaced` (each `Replaced` arm, a value one past
     `value_bytes` as `TooLarge`, and an application item's each `application::Replaced`);
     `a_restore_ask_enters_the_people_vocabulary`;
     `a_restored_outcome_and_a_forgotten_refusal_reach_the_wire` (`left_out` of a
     `ProjectPolicy` and an `Application` setting).
   - `jig-client-protocol-link/src/tests/settings_restore.rs`, in the style of
     `tests/settings.rs`' `every_new_record_translates_in_both_directions`:
     `a_project_policy_setting_round_trips_in_items_reviews_and_refusals`;
     `revision_items_round_trip_every_replaced_arm`;
     `a_restore_ask_round_trips_through_the_wire_and_the_kept_table`;
     `restored_outcomes_and_forgotten_refusals_reach_the_contract`.
   - Narrow runs, each checking its count:
     `heavy cargo nextest run -p jig-protocol-engine-clients --lib -E 'test(/settings_restore::/)'`,
     then `-E 'test(/settings|answers|asks/)'`;
     `heavy cargo nextest run -p jig-client-protocol-link --lib -E 'test(/settings_restore::/)'`,
     then `-E 'test(/settings|store|compound/)'`;
     `heavy cargo nextest run -p jig-client-wire-world --test suite -E
     'test(/^(drift|settings|settings_proposals|fuzzy)::/)'`;
     `heavy cargo nextest run -p jig-client-view --lib -E 'test(/words/)'`;
     `heavy cargo nextest run -p jig-client --lib`;
     `heavy cargo nextest run -p jig-client-scripted-engine --lib`.
     A change of meaning (a revision read's items now carry a value whole beside the set one),
     so its readers run: `heavy cargo nextest run -p jig-core-world --test suite -E
     'test(/^settings_reads::/)'` and, in temper, `heavy cargo nextest run -p temper-client-tools
     --lib -E 'test(/settings|revision/)'`.
8. `cargo fmt`; `heavy cargo clippy -p <each package changed> --all-targets -- -D warnings`;
   `heavy cargo check --workspace --all-targets`.
9. Status: in `jig/docs/design/client/domain/contract.status.md`, from 3.2's entry remove
   "`Restore` of a settings revision" (the list becomes "`Hold`; `Move` of a live task as its new
   requester; `Succeed` (neither `contract::Ask` nor the client wire's v19 schema has them)")
   and "; a project's own policy among `ChangeSettings`' and `Propose`'s items
   (`contract::Setting` names none)"; from 5.1, "a settings review's item of a project's own
   policy (`contract::ReviewItem`). " (5.1 then reads "... no `resolved_default` preview; the
   accounts document's ..."); and the sentence "6: `revision{n}`'s items carry nothing of what
   each replaced (`contract::RevisionItem`)." (the other 6 entry stays). Then `python3 -I -B tools/citations/citations.py --base
   origin/main`. Commit and push.

## Report

The wire records and arms added; the goldens written and removed (a count, and any of another
family); the arms replaced and added, by file (a list), temper's among them; the phrase ids
and texts; any expectation corrected and why.
