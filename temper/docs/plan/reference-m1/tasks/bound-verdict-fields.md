+++
id = "bound-verdict-fields"
part = "bound"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/limits.rs", "jig/crates/jig-domain-tasks/src/verdict.rs", "jig/crates/jig-domain-tasks/src/activity.rs", "jig/crates/jig-domain-tasks/src/lib.rs", "jig/crates/jig-domain-tasks/src/tests.rs", "jig/crates/jig-domain-core/src/limits.rs", "jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/src/tests/open.rs", "jig/tests/tasks/tests/suite/memory.rs", "jig/tests/tasks/tests/suite/verdict_fields.rs", "jig/docs/design/domain/tasks.status.md"]
effort = "medium"
minutes = 45
+++

# bound-verdict-fields: a verdict's fields counted apart from a result's bytes

## Goal

A deployment whose `tasks.result_bytes` is past 512 opens its store and admits verdicts: a
verdict's fields are counted apart, at most `VERDICT_FIELDS = 512`, the settings schema's
`list 512` ceiling, never from a result's bytes. Today the settings codec sizes
`limits.verdict_fields` as `result_bytes` (`jig/crates/jig-protocol-store/src/codec/settings.rs`,
`limits`), so any profile raising a result's bytes past 512 (a producer's report in a
change's result: contract C6) fails at store open.

Acceptance:

- `jig_domain_tasks::VERDICT_FIELDS: u32 = 512`, exported from the crate's root, equal to
  `settings_v1::CEILINGS.verdict_fields` (a test says so).
- The settings codec sizes `verdict_fields` as `VERDICT_FIELDS.min(result_bytes)`;
  `field_rule_name` and `approval_state` keep `result_bytes`.
- A verdict contract and a verdict result admit 512 fields and refuse 513 with
  `Refusal::Contract`, with `result_bytes = 2048`.
- Verdict arrays are priced at `VERDICT_FIELDS.min(result_bytes)` fields wherever the code
  prices them by `result_bytes` today; a field rule's name keeps the whole byte allowance.
- A store opens with `result_bytes = 2048`.
- The memory story for saturated verdict arrays holds at the stock limits and at
  `result_bytes = 2048`.

## Inputs

- `jig/docs/design/domain/tasks.md`, section 11 (the bullet "a spec's and a result's bytes
  ... a verdict's fields are counted apart, at most 512 (`VERDICT_FIELDS`, the store's
  schema's ceiling)"): the design this builds.
- `jig/docs/design/domain/tasks.status.md`: its **Not yet** entry for section 11, which this
  task removes.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C8.

## Owns and may change

Owns the files in the header. The edits there:

- `jig-domain-tasks/src/limits.rs`: the constant, with a doc line ("Maximum fields in one
  verdict, equal to the settings-v1 schema's `list 512` ceiling (domain/tasks.md, section
  11)."); `lib.rs`: its export beside `Limits`.
- `jig-domain-tasks/src/verdict.rs`: `fields_bytes` returns `None` past `VERDICT_FIELDS`
  fields; `rules_valid` bounds `choice.fields.len()` by `VERDICT_FIELDS.min(limit)`.
- Pricing: `jig-domain-tasks/src/activity.rs` (near line 129) and
  `jig-domain-core/src/limits.rs` (near line 119) price verdict field arrays by
  `result_bytes` today: count `VERDICT_FIELDS.min(result_bytes)` fields there, keeping the
  name bytes at `result_bytes`. If another site prices verdict field arrays by
  `result_bytes` (`rg "FieldRule>\(\)|ResultField>\(\)" jig/crates`), change it the same
  way and name it in the report. `jig-domain-core/src/limits.rs` is changed on plan
  reference-m7a's trunk too: keep the hunk to these lines.
- `jig-protocol-store/src/codec/settings.rs`, `limits`: the one line
  `limits.verdict_fields = tasks::VERDICT_FIELDS.min(domain.tasks.result_bytes);` (the
  crate's name for `jig_domain_tasks` as the file has it).
- `jig/docs/design/domain/tasks.status.md`: remove the words this task closes (common.md's
  stated departure); if the status has no such entry, leave it.
- If a memory figure or a frame bound in another jig test moves with the new pricing (a
  golden of limits, a `Measured` figure), update that figure alone and name it in the
  report. A stored format does not change: the schema's ceiling is already 512.

## Steps

1. The constant and its export; `fields_bytes` and `rules_valid`; the pricing sites; the
   codec line.
2. Tests, all in existing files:
   - `jig-domain-tasks/src/tests.rs`: `verdict_field_bytes_refuse_more_than_the_schema_count`:
     513 fields named by `name.to_be_bytes()` give `fields_bytes(..) == None`.
   - `jig-protocol-store/src/tests/open.rs`, with `const LARGE_RESULT_BYTES: u32 = 2048;`:
     `a_store_opens_with_result_bytes_above_the_verdict_field_ceiling` (the open fixture's
     limits with `core.tasks.result_bytes = 2048` open with no problem, as the file's other
     open tests assert); `settings_size_verdict_fields_apart_from_result_byte_bounds` (for
     `(32, 32)` and `(2048, 512)`: `codec::limits(&domain)`'s `settings.verdict_fields` is the
     second, `field_rule_name` and `approval_state` the first);
     `the_verdict_field_limit_equals_the_settings_schema_ceiling`.
   - `jig/tests/tasks/tests/suite/verdict_fields.rs`, a world at `result_bytes: 2048`
     (`World::new(<seed>, tasks::Limits { result_bytes: 2048, ..LIMITS })`), rules named by
     two-byte big-endian numbers: `a_verdict_contract_admits_the_schema_field_ceiling`,
     `a_verdict_contract_refuses_one_field_past_the_schema_ceiling` (refused with
     `Refusal::Contract`, no live record made), `a_verdict_result_admits_the_schema_field_ceiling`,
     `a_verdict_result_refuses_one_field_past_the_schema_ceiling` (refused, no person
     answer kept).
   - `jig/tests/tasks/tests/suite/memory.rs`: turn
     `saturated_verdict_machine_field_arrays_fit_their_declared_owner_bound`'s body into
     `fn check_verdict_arrays(l: &Limits)` over `VERDICT_FIELDS.min(l.result_bytes)` rules
     (one-byte names up to 256, two-byte past it), called at `LIMITS` by the existing test
     and by `verdict_field_arrays_fit_with_result_bytes_above_the_schema_ceiling` at
     `Limits { tasks: 1, project_tasks: 1, tree_tasks: 1, batch: 1, contract_choices: 1,
     result_bytes: 2048, ..LIMITS }`.
3. Narrow tests: `heavy cargo nextest run -p jig-domain-tasks --lib`, `-p jig-protocol-store
   --lib -E 'test(/verdict|result_bytes/)'`, and the tasks world's suite binary filtered to
   `verdict_field|verdict_arrays` (find its package and binary in `jig/tests/tasks/Cargo.toml`).
4. Then the implementer's text's checks: fmt, old forms, clippy on `jig-domain-tasks`,
   `jig-domain-core`, `jig-protocol-store` and the tasks world, and the focused tests of
   those packages. A change of meaning: `jig-domain-core`'s and `jig-protocol-store`'s
   focused tests run whole.
5. Commit ("Count a verdict's fields apart from a result's bytes") and push.

## Report

The implementer's text's report, plus: each pricing site changed, and any memory figure
moved, with its old and new value.
