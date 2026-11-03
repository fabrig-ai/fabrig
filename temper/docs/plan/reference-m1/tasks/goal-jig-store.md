+++
id = "goal-jig-store"
part = "goal-jig"
kind = "feature"
needs = ["goal-jig-surface", "bound-verdict-fields"]
files = ["jig/crates/jig-protocol-store/schema/settings-v1.schema", "jig/crates/jig-protocol-store/schema/people-v14.schema", "jig/crates/jig-protocol-store/src/generated/settings_v1.rs", "jig/crates/jig-protocol-store/src/generated/people_v14.rs", "jig/crates/jig-protocol-store/src/generated/layouts.rs", "jig/crates/jig-protocol-store/src/codec/people.rs", "jig/crates/jig-protocol-store/golden/**", "jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/src/codec/people/batch.rs", "jig/crates/jig-protocol-store/src/tests/**", "jig/tests/store-codecs/**"]
effort = "medium"
minutes = 60
+++

# goal-jig-store: the store keeps a template's resources and a member's symbols

## Goal

A template's `resources` and a stored batch member's `symbols` (contract C2) survive the
store: the settings family's `Template` record and the people family's `Member` record carry
them, encoded and decoded whole, where `goal-jig-surface` left them empty on decode.

Acceptance:

- `schema/settings-v1.schema`: a `record Symbol { connector: u16 symbol: u16 }`, and
  `Template` gains, after `llm`, `resources: list 32 Symbol` (32, the most holdings a task
  takes in any profile, `jig-protocol-config/src/profile/standard.rs`: check
  `jig_domain_tasks::Limits::holdings` across the profiles and use the largest if it is more,
  saying so).
- `schema/people-v14.schema`: `Member` gains, after `named`, `symbols: list 32 Symbol` (the
  same record, declared in this schema too, as the schemas declare their own records).
- The schemas change in place, with their generated code and one set of golden bytes,
  as jig's `protocol/store.md`, section 5 says before the first release (the settings'
  version stays 1, the people's 14).
- `codec/settings.rs` encodes and decodes `Template.resources`; `codec/people/batch.rs`
  encodes and decodes `Member.symbols`; each refuses a list past its bound as the codec
  refuses any.
- Tests (`src/tests/`, beside the settings and people codec tests):
  `a_template_with_resources_round_trips_through_the_settings_store` and
  `a_stored_member_keeps_its_symbols`; and the golden drift check passes.

## Inputs

- `jig/docs/design/protocol/configuration.md`, 3.3 ("Served in `me` with the rest of the
  template, and kept by the settings with it").
- `jig/docs/design/protocol/store.md`, section 5 (the paragraph "Settings ... Until the first
  release, each family's version keeps its current number, and its schema, generated code
  and one set of golden bytes change in place").
- No status edit: `goal-jig-client` removes the 3.3 **Not yet** lines once the whole path
  holds.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C2.

## Owns and may change

- Owns the files in the header.
- **Neighbours:** the settings family of this plan changes `settings-v1.schema`'s `Charter`
  record (a charter's default wake policy) on this trunk, and plan reference-m7a changes
  `jig-protocol-store` on its own: keep the hunks to `Symbol`, `Template` and `Member`. If
  the schema or generated file has moved under you at your base, regenerate from the schema
  as it is, never by hand.
- **Options:** if `list 32` does not fit the generator's ceilings for these records, use the
  nearest bound it accepts and say so. If `Member` is also kept in the tasks family
  (`tasks-v9.schema`'s `Member` is the tasks child's `New`, which carries holdings, not
  symbols), leave tasks-v9 alone.

## Steps

1. Edit the two schemas; regenerate code and goldens with `SKEIN_CODEGEN_WRITE=1 heavy cargo
   nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'`, then
   run the same without the variable: it must pass.
2. The codec arms and the two tests: `heavy cargo nextest run -p jig-protocol-store --lib -E
   'test(a_template_with_resources_round_trips_through_the_settings_store) |
   test(a_stored_member_keeps_its_symbols)'` (check two selected).
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-store --all-targets -- -D warnings`; `heavy cargo nextest run -p
   jig-protocol-store -p jig-store-codecs-world`.
4. Commit ("jig store: a template's resources and a member's symbols"), push.

## Report

The schema lines, the bounds chosen, the goldens regenerated, the tests and counts.

## Answers

- 2026-11-02. May this task regenerate `src/generated/layouts.rs`, whose settings and people
  fingerprints its schema changes make stale? Yes: the file is generated from the schemas this
  task changes, and the task's full checks (`the_stores_layouts_have_no_drift`,
  `current_layouts_are_the_eight_distinct_runtime_schema_fingerprints`) cannot pass without
  it; it is now in the header, with `codec/people.rs` (the `Limits.member_symbols` line the
  compiler required). Regenerate it, never by hand: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest
  run -p jig-store-codecs-world --test drift -E 'test(=the_stores_layouts_have_no_drift)'`,
  then the same without the variable; only the `settings` and `people` lines may change (the
  settings family's `Charter` change on this trunk moves the same `settings` line: if it has
  landed at your base, regenerate from the schema as it is). Then both formerly failing tests
  alone, and step 3's checks.

## Rebase (2026-11-02, step merge, foreman reference-m1/goal-jig-store/foreman-2)

rebase onto the landing trunk, retain job support and symbols/resources, and regenerate combined store artifacts.
The merge conflicts with main commit `97159dc54e` (“jig: persist the job task surface while refusing admission”), brought into reference-m1 by `8305f532de`.
Conflicting paths under `jig/crates/jig-protocol-store/`:
- `src/generated/layouts.rs`
- `src/generated/people_v14.rs`
- `src/generated/settings_v1.rs`
- `src/tests/codec.rs`
- `src/tests/settings.rs`

Keep the merged schemas’ `ExecutorJob` record and `Executor::job` arm alongside this task’s `Symbol`, `Template.resources`, and `Member.symbols` additions. Keep versions 1 and 14 and bounds 32.
Regenerate `people_v14.rs` and `settings_v1.rs` from those combined schemas; never resolve generated hunks by hand.
In `src/tests/codec.rs`, retain all incoming job tests and this task’s `a_stored_member_keeps_its_symbols`; retain the incoming `job: None` fixture initializer.
In `src/tests/settings.rs`, retain both `job_delegation_kinds_round_trip_in_settings` and `a_template_with_resources_round_trips_through_the_settings_store`.
Verify the automatically merged `src/codec/settings.rs` keeps both job executor translations and resource encoding/decoding.
Run the spec’s drift regeneration to refresh generated codecs and goldens, then repeat without `SKEIN_CODEGEN_WRITE`.
Regenerate layouts with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world --test drift -E 'test(=the_stores_layouts_have_no_drift)'`, then repeat without the variable.
Relative to the landing base, only settings and people fingerprints should change; preserve its tasks fingerprint and existing tasks artifacts.
Run both acceptance tests (two selected), the layout fingerprint checks, and step 3’s formatting, old-form, clippy and package checks.
