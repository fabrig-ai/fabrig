+++
id = "store-ceilings"
part = "jig-rows"
kind = "migration"
needs = ["tasks-pieces"]
files = ["jig/crates/jig-protocol-store/schema/tasks-v9.schema", "jig/crates/jig-protocol-store/schema/core-v6.schema", "jig/crates/jig-protocol-store/src/generated/tasks_v9.rs", "jig/crates/jig-protocol-store/src/generated/core_v6.rs", "jig/crates/jig-protocol-store/src/generated/layouts.rs", "jig/crates/jig-protocol-store/golden/tasks/**", "jig/crates/jig-protocol-store/golden/core/**", "jig/crates/jig-protocol-store/src/tests/**", "jig/tests/store-codecs/tests/suite/profiles.rs", "jig/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 75
+++

# tasks-v9's and core-v6's ceilings hold the draft's figures

## Goal

The store's schema ceilings admit jig's `standard` at the draft's figures (contracts K2): 80
live delegates, an inbox of 82 messages and a run's inbox of 82, a 128 KiB spec, a 256 KiB
message and result. Today tasks-v9 caps delegates at 64, the inbox at 62, the question lists at
64 and every spec, message and result field at 65,536 bytes, and core-v6 caps a run's inbox at
64 and a projection's words at 65,536. Each ceiling changes in place (no new version;
`notes/common.md`), with room above standard's figure; no configured figure changes here.

Acceptance:

- `jig/crates/jig-protocol-store/schema/tasks-v9.schema`, these ceilings and no others (the
  pieces `tasks-pieces` made keep theirs):

  | Fields | Becomes | Mapped from (`codec/tasks.rs`) |
  |---|---|---|
  | `TaskInterests.delegates` | `list 256` | `delegates` |
  | `TaskHistory.question_lines`, `TaskHistory.proposal_rows`, `TaskHistory.refused`, `TaskRun.replies` | `list 128` | `inbox_messages` |
  | `TaskInterests.inbox` | `list 130` | `inbox_messages + 2` |
  | `BytesParameter.value`, `JobSpec.bytes`, `Spec.words` | `bytes 262144` | `spec_bytes` |
  | `JobResult.bytes`, `FieldRule.name`, `ResultField.name`, `ResultField.value`, `Approval.state`, `Approved.state`, `Approved.words`, `ReportResult.words`, `VerdictResult.words`, `ChangeResult.title`, `ChangeResult.words`, `FailureResult.reason`, `Cancelled.reason`, `Grace.reason`, `ApprovalMessage.state`, `RejectedEscalation.reason` | `bytes 524288` | `result_bytes` |
  | `PlacedHold.reason`, `PersonProposal.reason`, `RejectedProposal.reason`, `Amendment.instructions`, `Amendment.reason`, `Proposal.reason`, `Settings.instructions`, `RejectedPersonProposal.reason`, `ActivityWords.words`, `TaskHistory.instructions` | `bytes 524288` | `message_bytes` |
  | `History.reason`, `Word.words` | `bytes 524288` | the larger of message and result |
  | `Verdict.fields`, `VerdictResult.fields`, `VerdictMessage.fields` | `bytes 1048576 pieces …` (their item type kept) | the result's bytes and `VERDICT_FIELDS` items' framing |

  If `codec/tasks.rs` maps a field this table does not name from `spec_bytes`, `result_bytes`
  or `message_bytes` and its ceiling is below the column's figure, raise it the same way and
  list it in the report.
- `jig/crates/jig-protocol-store/schema/core-v6.schema`: `ProjectionGoal.words` and
  `ProjectionTask.words` `bytes 262144` (from `spec_bytes`); `ProjectionMilestone.words`
  `bytes 524288` (the larger of message and result); `RunProof.outstanding` `list 128 Number`
  (from `run_inbox`). Nothing else of core-v6 (`transcript-ceilings` made its turn's).
- people-v14 changes nothing: none of its ceilings is mapped from a limit K2 raises past it
  (`people.operations` 81 within its 1,024; `people.batch` stays 4).
- Generated code, goldens and layouts by the writer (`notes/common.md`:
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world --test drift`);
  commit `tasks_v9.rs`, `core_v6.rs`, `layouts.rs` and any golden the writer changed (a ceiling
  raise changes no golden `.bin`; say so if one changed).
- A test holds the draft's figures in the codecs (below), and the configuration status's
  sentence "6: the store's ceilings cap what standard is to hold: ..." in
  `jig/docs/design/protocol/configuration.status.md` is removed, the rest of the line kept.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K2 (the table).
- `jig/docs/design/protocol/store.md`, 5.1 (a family's limits are made from the domain's and
  never past its ceilings) and 5.4 (the largest value).
- `jig/docs/design/protocol/configuration.md`, 6 (the bullet "Standard holds fabrig's own
  development"), for the figures the ceilings must admit.

## Owns and may change

- The schemas, generated files, goldens and layouts named above; `jig-protocol-store`'s tests
  where one pins a ceiling these change (change its figure); and
  `jig/tests/store-codecs/tests/suite/profiles.rs` for the new test.
- Every declaration's worst case at its ceilings stays under `u32` with these figures (the
  largest, `TaskRecord`, about 420 MB by hand). If skein's codec refuses a schema
  (`SchemaError::Large`), stop and ask with its words; do not lower a ceiling below the column.
- No configured limit, profile or `codec/` mapping changes.

## Steps

1. Both schemas; run the writer; `git diff --stat` shows only the files above.
2. The test, in `jig/tests/store-codecs/tests/suite/profiles.rs`,
   `the_codecs_hold_fabrig_scale`: small's core limits (`jig_protocol_config::profile(Profile::Small).core`)
   with `tasks` set to `tasks: 128, project_tasks: 128, project_goals: 80, tree_tasks: 384,
   depth: 32, delegates: 80, batch: 160, batch_bytes: 4_194_304, inbox_messages: 82,
   run_inbox: 82, inbox_bytes: 21_495_808, spec_bytes: 131_072, result_bytes: 262_144,
   message_bytes: 262_144, contract_choices: 4, authority_grants: 16, authority_segments: 8,
   hold_segments: 8, executor_kinds: 16` (the rest small's): `jig_protocol_store::codec::limits(&core)`
   is `Some`, and the test prints (`--no-capture`) `tasks_v9::Row::encoded_bound` at them. A
   second case at one past each ceiling's domain figure where a ceiling binds
   (`delegates: 257`, `inbox_messages: 129`, `spec_bytes: 262_145`, `result_bytes: 524_289`)
   is `None`.
3. Narrow: `heavy cargo nextest run -p jig-store-codecs-world --test suite -E 'test(=profiles::the_codecs_hold_fabrig_scale)' --no-capture`;
   then `heavy cargo nextest run -p jig-store-codecs-world --test drift`, the packages
   `-p jig-protocol-store`, `-p jig-store-codecs-world`, `-p jig-store-world`,
   `-p jig-protocol-config`, `-p jig-test-service`, and `heavy cargo check -p temper-service -p temper-twin --tests`.
4. The status sentence; `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo
   clippy -p jig-protocol-store -p jig-store-codecs-world --all-targets -- -D warnings`; the
   citations check; commit and push.

## Report

As the implementer's text asks, with the printed row bound at the draft's figures (the figure
`standard-scale` sizes standard's store by), and every generated file the writer changed.

## Answers

### 2026-11-03: Resume after tasks-pieces completes its codec translations (recommended), or expand this task to repair those mappings here? The required writer cannot compile the current prerequisite. (fabrig-scale/store-ceilings-1, foreman fabrig-scale/store-ceilings/foreman-1)

Resume store-ceilings after tasks-pieces completes its codec translations and verdict bounds; those repairs remain with tasks-pieces.
The store-ceilings spec declares `needs = ["tasks-pieces"]` and explicitly requires “No configured limit, profile or `codec/` mapping changes.”
The tasks-pieces acceptance owns the encoders and decoders in `codec/tasks/` and the verdict bounds in `codec/tasks.rs`, including `min(VERDICT_FIELDS, result_bytes)` framing plus result bytes.
`jig/docs/design/protocol/store.md`, 5.1, requires verdict pieces framed for “at most `VERDICT_FIELDS` items”, never as many items as result bytes.
The checkout confirms the old list translations and result-byte multiplication remain.
Keep store-ceilings parked until that prerequisite is complete, then resume generation, acceptance tests, status update and required checks.
The pushed checkpoint is not accepted as done: generated files, the row bound and acceptance checks remain outstanding.
