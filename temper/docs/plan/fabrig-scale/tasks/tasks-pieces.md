+++
id = "tasks-pieces"
part = "jig-rows"
kind = "migration"
needs = ["batch-bytes", "override-relations", "transcript-ceilings", "forge-profile"]
files = ["jig/crates/jig-protocol-store/schema/tasks-v9.schema", "jig/crates/jig-protocol-store/src/generated/tasks_v9.rs", "jig/crates/jig-protocol-store/src/generated/layouts.rs", "jig/crates/jig-protocol-store/golden/tasks/**", "jig/crates/jig-protocol-store/src/codec/tasks.rs", "jig/crates/jig-protocol-store/src/codec/tasks/**", "jig/crates/jig-protocol-store/src/tests/**", "jig/tests/store-codecs/tests/suite/**", "jig/crates/jig-protocol-config/src/domain.rs", "jig/crates/jig-protocol-config/src/problem.rs", "jig/crates/jig-protocol-config/src/tests/overrides.rs", "jig/testing/jig-test-service/src/tests.rs", "temper/crates/temper-service/src/tests/startup.rs", "jig/docs/design/protocol/store.status.md"]
effort = "high"
minutes = 180
+++

# tasks-v9 priced as store.md 5.1 says: one piece for what the domain bounds by its bytes

## Goal

tasks-v9's largest row is the sum of byte bounds the domain holds, linear in every limit:
the collections the domain bounds by their bytes in all become pieces whose bound is the
domain's sum plus their items' framing, and a batch (a proposal's creations, a verdict's
follow-ups, a recurring template's tasks) is one piece by the batch's bytes, with no batch of
its own inside each creation (`jig/docs/design/protocol/store.md`, 5.1 and 5.4). In place: the
family stays `jig-store-tasks 9`; a store of the old layout is refused at open (5.3), which the
user accepted (`notes/common.md`).

Acceptance:

- `jig/crates/jig-protocol-store/schema/tasks-v9.schema`, these fields and no others:

  | Record.field | Was | Becomes |
  |---|---|---|
  | `Spec.parameters` | `list 64 Parameter` | `bytes 1048576 pieces Parameter` |
  | `PartPath.segments` | `list 64 PartSegment` | `bytes 65536 pieces PartSegment` |
  | `Authority.grants` | `list 64 Grant` | `bytes 65536 pieces Grant` |
  | `Authority.note_resources` | `list 64 ResourceScope` | `bytes 65536 pieces ResourceScope` |
  | `Name.path` | `list 64 Segment` | `bytes 65536 pieces Segment` |
  | `SavedResource.path` | `list 64 AuthoritySegment` | `bytes 65536 pieces AuthoritySegment` |
  | `RecurringTemplate.batch` | `list 4 Member` | `bytes 67108864 pieces Member` |
  | `ProposedBatch.members` | `list 4 Creation` | `bytes 67108864 pieces Creation` |
  | `ResultFollowupsDelegates.value` | `list 4 Creation` | `bytes 67108864 pieces Creation` |
  | `RecurringState.last_batch` | `list 4 Number` | `list 1024 Number` |
  | `TaskInterests.result_followups` | `list 4 Number` | `list 1024 Number` |

  `Creation` keeps `recurring: option RecurringTemplate` (5.1, "No batch inside a batch": its
  template is within the creation's piece).
- `codec/tasks.rs`, `limits`, derives each new piece's bound from the domain's limits, with
  `EMPTY` a `tasks_v9::Limits` whose every list and bytes bound an item reaches is zero (as
  `codec/people/pieces.rs`'s `EMPTY`), so an item's `encoded_bound(&EMPTY)` is its fixed part:
  - `authority_grants`: `authority_grants × (Grant(EMPTY) + 4 + authority_segments × AuthoritySegment(EMPTY)) + authority_bytes`;
    `authority_note_resources` likewise with `ResourceScope`;
  - `name_path`: `hold_segments × (Segment(EMPTY) + 4) + hold_bytes`; `part_path_segments`
    likewise with `PartSegment`; `saved_resource_path`:
    `authority_segments × (AuthoritySegment(EMPTY) + 4) + authority_bytes`;
  - `spec_parameters`: `parameters × (Parameter(EMPTY) + 4) + spec_bytes`;
  - `verdict_fields`: `min(VERDICT_FIELDS, result_bytes) × (FieldRule(EMPTY) + 4) + result_bytes`;
    `verdict_result_fields` and `verdict_message_fields` likewise with `ResultField` (they were
    `result_bytes × 13`; at `result_bytes` 512 both rules give 6,656);
  - `proposed_batch_members` and `result_followups_delegates_value`:
    `people::pieces::piece_bound(batch_bytes, size_of::<jig_domain_tasks::New>(), Creation(EMPTY), true)`;
    `recurring_template_batch` the same with `Member(EMPTY)`;
  - `recurring_state_last_batch` and `task_interests_result_followups` stay `domain.batch`.
  Every product and sum is checked; overflow is `None`, as today. The `+ 4` is a piece's
  length prefix.
- The encoders and decoders in `codec/tasks/` write and read each new piece with
  `crate::codec::pieces::join` and `skein_codec::piece::split`, as `codec/tasks/pieces.rs`
  does for verdict fields: `value.rs` (`spec`/`read_spec`, `part`/`read_part`, `name`/`read_name`,
  `authority`/`read_authority`), `record.rs` (`saved`/`read_saved`), `phase.rs`
  (`new`/`read_new`, `template`/`read_template`), `proposals.rs` (`batch`/`read_batch`) and
  `carried.rs` (the follow-ups). An encoder refuses past the bound with the field's
  `wire::Path` and `Reason::Bound`, never truncates; `template` still refuses a recurring
  member.
- A batch the tasks child admitted always encodes: for every batch whose
  `jig_domain_tasks::batch_bytes` is within `batch_bytes` and whose members pass
  `jig_domain_tasks::bounded_batch`, the creations' piece is within its derived bound. The
  derivation rests on each item's encoded bytes past its fixed part being at most what the
  domain measures it to own past `size_of::<New>()`; the tests below hold it shape by shape.
- `jig-protocol-config`: `domain.rs`'s `batch_ceiling` checks `limits.tasks.batch` against
  `tasks_v9::CEILINGS.task_interests_result_followups`, naming the field
  `tasks-v9.TaskInterests.result_followups`; `problem.rs`'s `codec_ceiling_message` keys its
  batch words on that field name (the words otherwise unchanged).
- Generated code, goldens and the layout fingerprints regenerated by the writer
  (`notes/common.md`: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world --test drift`).
  Commit `src/generated/tasks_v9.rs`, `golden/tasks/**` and `src/generated/layouts.rs`; if
  `core_v6.rs` or `golden/core/**` change because core-v6 holds tasks records, commit them
  too and say so; any other family's diff means your tree is stale: say so.
- `jig/docs/design/protocol/store.status.md`: the **Not yet** entry "5.1, 5.4: tasks-v9 prices a
  batch by its count ..." removed, the rest of the line kept.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K2 (the last bullet).
- `jig/docs/design/protocol/store.md`, 5.1 (the bullets "A collection the domain bounds by its
  bytes in all is one piece", "The tasks family's pieces", "No batch inside a batch") and 5.4
  (the bullet "The largest value is the largest tasks row").
- `jig/docs/design/domain/tasks.md`, 11 (the bullet "a batch's bytes"; a verdict's fields).
- skein's `skein/docs/design/codec.md`, 3 (`bytes N pieces T`).

## Owns and may change

- `jig-protocol-store`: the schema, `codec/tasks.rs`, `codec/tasks/**`, the generated files
  and goldens named above, and its tests. `codec/people/pieces.rs`'s `piece_bound` is reused,
  not changed (it is `pub(in crate::codec)`). Byte ceilings other than those in the table stay
  (`store-ceilings` raises them).
- **Readers of the reshaped types:** `src/tests/codec.rs` and `src/tests/codec/phase.rs` build
  `tasks_v9` records by hand: change them to the pieced parts. `jig/tests/store-codecs/tests/suite/`
  (`profiles.rs`, `memory.rs`, `fuzzy/`): change what the shape change breaks, no expectation
  otherwise.
- **Pinned computed figures:** `jig/testing/jig-test-service/src/tests.rs`,
  `both_jig_profiles_have_usable_store_edges`, pins each profile's computed largest value
  (1,604,798 and 10,767,550): change each to the figure the run prints (the row is smaller
  now; the profiles' `STORE_KV` constants stay, `standard-scale` resizes standard's). Commit
  and ops do not move.
- **temper's pinned lines** (each edited in its figures; the one exception named below), in
  `temper/crates/temper-service/src/tests/startup.rs`, `capacity_overrides_name_the_failed_dependency`:
  - `(b"tasks.batch", 8, &["tasks.batch", "8", "ResultFollowupsDelegates.value", "4", "schema ceiling", "reduce"])`
    becomes `(b"tasks.batch", 1025, &["tasks.batch", "1025", "TaskInterests.result_followups", "1024", "schema ceiling", "reduce"])`;
  - `(b"tasks.batch", 2, &["tasks.batch", "2", "TaskRecord", "2391079", "2166708", "profile ceiling", "reduce"])`:
    the batch's count no longer prices the task record, so no count passes the profile's value.
    The row's member becomes `b"tasks.batch_bytes"` (the one edit beyond a figure; say so in
    the report) with the least power of two whose task record passes the init document's
    profile value, and its words `&["tasks.batch", "TaskRecord", "<needs>", "<has>", "profile ceiling", "reduce"]`
    with the figures the run prints (temper's words name `tasks.batch` for any task-record
    refusal; that is temper's to change, not yours).
  No other temper file.
- If skein's codec refuses the schema (`SchemaError::Large` at `jig/tests/codec-families.rs`),
  stop and ask with its words: the ceilings above are sized to keep every declaration under
  `u32`.
- If a test reads committed tasks-v9 bytes other than the generated goldens (a fixture store,
  a patch fixture), stop and ask, naming it.

## Steps

1. The schema; run the writer; `git diff --stat` shows only the files above.
2. `EMPTY` and `limits` in `codec/tasks.rs`; then each encoder and decoder, starting from
   `value.rs` (`spec`, `part`, `name`, `authority`), then `record.rs`, `phase.rs`,
   `proposals.rs`, `carried.rs`. Write one encode and decode pair per item type beside
   `codec/tasks/pieces.rs`'s, as it does for fields and rules: no generic helper (step code
   has no generics, `jig/AGENTS.md`).
3. Config: `batch_ceiling` and `codec_ceiling_message`; its test,
   `jig-protocol-config/src/tests/overrides.rs`, `batch_past_the_schema_keeps_its_binding_count`:
   `"tasks.batch":1025`, field `tasks-v9.TaskInterests.result_followups`, needed 1025, ceiling
   `CEILINGS.task_interests_result_followups`.
4. Tests in `jig-protocol-store`:
   - `src/tests/codec/pieces.rs`:
     `the_encoded_piece_bound_holds_a_verdict_at_the_domain_sum_exactly` keeps its 6,656 at 512;
     `verdict_fields_are_framed_for_at_most_verdict_fields_items` (at `result_bytes` 4,096,
     `verdict_fields` is `512 × (FieldRule(EMPTY) + 4) + 4096` and `verdict_result_fields` and
     `verdict_message_fields` are `512 × (ResultField(EMPTY) + 4) + 4096`, where the old rule
     gave `4096 × 13`);
     `an_authority_is_one_piece_by_its_bytes` (an authority with `authority_grants` grants and
     as many note resources, every pattern `authority_segments` deep, `authority_bytes` in
     all, round-trips; one byte more is refused at encode);
     `a_name_is_one_piece_by_the_hold_bytes`; `a_spec_parameters_piece_holds_the_spec_bytes`;
     `a_batch_at_its_bytes_encodes_in_every_shape`: with a domain fixture whose `batch_bytes`
     is 131,072 and whose counts are wide, batches each within `batch_bytes` and
     `bounded_batch`, built to spend their bytes on (a) one spec's words, (b) many one-byte
     byte parameters, (c) a verdict contract of many one-byte-name fields, (d) many grants of
     many one-byte segments, (e) holdings of many one-byte segments, (f) `batch` members of the
     smallest shape, (g) one recurring member whose template holds `batch` members: each
     round-trips as a proposal's batch, as a verdict's follow-ups and (but (g)) as a recurring
     template;
     `a_creations_piece_past_its_bound_is_refused` (decoding a forged piece one byte past
     `proposed_batch_members` is `Reason::Bound`).
   - `src/tests/sizes.rs`: `the_task_row_grows_with_the_batch_count_only_by_its_numbers`:
     `tasks_v9::TaskRecord::encoded_bound` at `batch` 160 less at `batch` 4, the same
     `batch_bytes`, is `2 × 156 × 8` (the two `Number` lists).
   - a test that `Creation::encoded_bound(&EMPTY)` is under 1,024 (a field left at its
     ceiling in `EMPTY` would make it far larger).
5. Narrow runs: `heavy cargo nextest run -p jig-protocol-store --lib -E 'test(/codec::pieces::|sizes::/)'`;
   `heavy cargo nextest run -p jig-protocol-config --lib -E 'test(=tests::overrides::batch_past_the_schema_keeps_its_binding_count)'`;
   `heavy cargo nextest run -p jig-test-service --lib -E 'test(=tests::both_jig_profiles_have_usable_store_edges)'`;
   `heavy cargo nextest run -p temper-service --lib -E 'test(=tests::startup::capacity_overrides_name_the_failed_dependency)'`.
   Then `heavy cargo nextest run -p jig-store-codecs-world --test drift` (without the writer),
   the packages `-p jig-protocol-store`, `-p jig-store-codecs-world`, `-p jig-store-world`,
   `-p jig-protocol-config`, `-p jig-test-service`, `-p jig-core-world`, and
   `heavy cargo check -p temper-service -p temper-twin --tests`.
6. The status line; `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy
   -p jig-protocol-store -p jig-protocol-config -p jig-test-service -p jig-store-codecs-world
   --all-targets -- -D warnings`; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`;
   the citations check; commit and push.

## Report

As the implementer's text asks, with: `size_of::<New>()`, `Member(EMPTY)` and
`Creation(EMPTY)`; small's and standard's largest task row before and after (the jig-test-service
figures); the temper row's new member and figures; and every generated file the writer changed.

## Answers

### 2026-11-03: Resume tasks-pieces after batch-bytes pushes its required domain API (recommended), or assign that dependency's implementation to this task? The current dependency branch lacks Limits.batch_bytes and the public jig_domain_tasks::batch_bytes function. (fabrig-scale/tasks-pieces-1, foreman fabrig-scale/tasks-pieces/foreman-1)

Resume tasks-pieces after batch-bytes pushes its required domain API; that implementation remains with batch-bytes.
The tasks-pieces spec declares `needs = ["batch-bytes", "override-relations", "transcript-ceilings", "forge-profile"]`.
Its acceptance requires “`jig_domain_tasks::batch_bytes`” and derives batch piece bounds from `batch_bytes`.
The batch-bytes spec explicitly requires “`jig_domain_tasks::Limits` has `pub batch_bytes: u32`” and “`jig_domain_tasks::batch_bytes(batch: &[New]) -> Option<u64>` is public (exported from `lib.rs`)”.
Those domain files belong to batch-bytes and are outside tasks-pieces’ permitted files.
The checkout confirms the field and public export are absent; `owned.rs`’s function remains private.
Keep tasks-pieces parked until that dependency is available, then resume its remaining implementation and required checks.
The pushed schema checkpoint does not meet the task’s acceptance and is not accepted as done.
