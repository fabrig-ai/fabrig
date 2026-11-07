+++
id = "surface"
part = "surface"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-tasks/src/reasons.rs", "jig/crates/jig-domain-tasks/src/lib.rs", "jig/crates/jig-domain-tasks/src/llm.rs", "jig/crates/jig-domain-tasks/src/boundary.rs", "jig/crates/jig-domain-tasks/src/limits.rs", "jig/crates/jig-domain-tasks/src/run.rs", "jig/crates/jig-domain-tasks/src/tests/reasons.rs", "jig/crates/jig-domain-core/src/settings/charters.rs", "jig/crates/jig-domain-core/src/limits.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-domain-people/src/llm.rs", "jig/crates/jig-domain-people/src/lib.rs", "jig/crates/jig-protocol-config/src/names.rs", "jig/crates/jig-protocol-config/src/profile/*.rs"]
effort = "high"
minutes = 120
+++

# per-wake: the surface: `Reasons`, `CharterPart`, `LlmLater`, the record's and the claim's fields

## Goal

Give jig the types and fields every other task of this plan builds on, so the store, the
configuration, the client wire, the tasks child's offering, the core's parts and the later
choice are built in parallel: `contracts.md`, sections 1, 2, 3 (the type and
`parts_owned_bytes`) and 5 (the types only). This task carries the surface and its forced
readers; it changes no behaviour, but for the claim keeping what it is given.

Acceptance:

- Every name `temper/docs/plan/per-wake/notes/contracts.md` gives in sections 1, 2, 3 and
  5's type block exists as it gives it, with its doc comment citing its design section at
  type or module level: `Reasons` and its constants and functions, `REASON_NAMES`,
  `ALWAYS_OFFERED_NAMES`, `offered`; `TaskRecord.reasons` and `.offered`;
  `RunContext.reasons`; `Event::Claim`'s `reasons` and `offered`; `CharterModels.resumes`;
  `Limits.charter_parts`; `CharterPart` and `Charter.parts`; `parts_owned_bytes`;
  `jig_domain_tasks::LlmLater` and `LlmChoice.later`; `jig_domain_people::LlmLater` and its
  `LlmChoice.later`.
- `Reasons::of`, `Reasons::named` and `offered` are whole, by `contracts.md`, section 1.
- A task is made with `reasons: Reasons::FIRST` and `offered: Reasons::ALL`; `run_context`
  gives `reasons: row.reasons` (the offering task adds the waking kinds); a successful claim
  stores the event's `reasons` and `offered` on the record in its own commit.
- The core's claim (`jig-domain-core/src/routing/run/preparation.rs`, the `Event::Claim`
  it pushes, and `routing/run/jobs.rs`'s) passes `reasons: context.reasons` (a job's
  `Reasons::FIRST`) and `offered: Reasons::ALL` (the parts-core task computes it).
- The core fills `CharterModels.resumes` from the charter version's `policy.resume` wherever
  it builds one.
- `charter_owned_bytes` adds `parts_owned_bytes(&charter.parts)`.
- `tasks.charter_parts` is a named limit (`names.rs`, its line "Bounds parts per charter."),
  8 in the small and standard profiles.
- Every forced reader compiles with the empty or default value and nothing else changes:
  `parts: Box::new([])`, `later: None`, `reasons: Reasons::FIRST`, `offered: Reasons::ALL`,
  `resumes: <the charter's resume>` or `false` in a fixture, `charter_parts: 8`.
- Unit tests in `jig/crates/jig-domain-tasks/src/tests/reasons.rs`, one table: a row per
  `MessageKind` variant (twenty), each with its `Reasons::of` and whether `offered` passes it
  under `Reasons::EMPTY` and under its own bit, with a `Words` row from a person and one from
  a task; `REASON_NAMES` round-trips through `Reasons::named` to each bit;
  `ALWAYS_OFFERED_NAMES` and an unknown name give `None`; `is_known` false for bit 14. And in
  `jig-domain-core`, beside the documents' size test (`src/tests/settings/documents.rs`): a
  charter with two parts owns `parts_owned_bytes` more than with none, which grows by each
  part's instructions plus two.
- `heavy cargo check -p <pkg> --tests` is clean for every package the compiler names, every
  tree's; the lib tests of `jig-domain-tasks`, `jig-domain-core`, `jig-domain-people`,
  `jig-protocol-config` and `jig-protocol-store` pass, and `jig-protocol-store`'s goldens are
  unchanged (no schema changes here).

## Inputs

- `temper/docs/plan/per-wake/notes/common.md`; `notes/contracts.md`, sections 1, 2, 3 and 5.
- `jig/docs/design/domain/tasks.md`, section 3 (its reasons, its LLM choice's `later`) and
  8.6 (the reasons, the kinds, what a run is offered); `jig/docs/design/protocol/store.md`,
  section 5 (the bits); `jig/docs/design/protocol/configuration.md`, section 6 (a charter's
  parts' limit).

## Owns and may change

- Owns the files in the header.
- Forced readers, each edited only by the new field's name and its default value, or the
  new event fields at a construction or a match (`..` where it destructures):
  - `Charter` literals: `jig/crates/jig-protocol-config/src/charters.rs`,
    `jig/crates/jig-protocol-store/src/codec/settings.rs` (`decode_charter`),
    `jig/crates/jig-protocol-store/src/tests/settings.rs`,
    `jig/crates/jig-protocol-engine-clients/src/translate/settings/change.rs` and its tests,
    `jig/crates/jig-domain-core/src/tests.rs` and `src/tests/settings/documents.rs`,
    `jig/testing/jig-test-domain/src/tests/fixtures.rs`, `jig/testing/jig-test-profile/src/lib.rs`,
    `jig/testing/jig-engine-clients-kit/src/fixtures/receiving.rs`,
    `temper/tests/engine/domain/src/walking.rs`,
    `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`,
    `temper/crates/temper-service/src/stock.rs`.
  - tasks' `LlmChoice` literals: `jig/crates/jig-protocol-config/src/templates.rs`,
    `jig/crates/jig-protocol-store/src/codec/tasks/value.rs` (`read_llm`) and the settings
    codec's, `jig/crates/jig-protocol-tools` (its `llm` parse), 
    `jig/crates/jig-protocol-engine-clients/src/translate/llm.rs`,
    `jig/crates/jig-domain-core/src/routing/party/create.rs` and `runs/llm.rs`, the tasks,
    core, engine-clients, client-protocol, channel and joint-client worlds' and kits'
    builders, `temper/crates/temper-engine-smith/src/calls.rs` and `nested.rs`,
    `temper/crates/temper-engine-domain/src/route/forge_route/change_llm.rs` and its tests,
    `temper/crates/temper-protocol-store/src/records/change.rs` and its tests,
    `temper/tests/engine/domain/tests/suite/forge_routes/*.rs`,
    `temper/tests/real-loop/src/fixtures.rs`.
  - people's `LlmChoice` literals: `jig/crates/jig-domain-people/src/tests/*.rs`, the store's
    people codec, the engine's people translation, `jig/tests/people/tests/suite/simulation.rs`.
  - `TaskRecord`, `RunContext`, `Event::Claim` and `CharterModels` constructions and matches,
    in any tree.
  - `tasks::Limits` literals: `jig/crates/jig-domain-core/src/tests.rs`,
    `jig/crates/jig-domain-tasks/src/tests/revisions.rs`, `jig/crates/jig-protocol-config/src/overrides.rs`
    (its destructure only), `jig/crates/jig-protocol-store/src/tests/fixtures.rs`,
    `jig/testing/jig-test-profile/src/lib.rs`, `jig/tests/store/src/root.rs`,
    `jig/tests/tasks/src/world.rs`, `temper/crates/temper-engine-domain-forge/src/tests/reconcile.rs`,
    `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`,
    `temper/tests/engine/domain/src/walking.rs`.
- Pre-approved: any other site of these types the compiler names, in any tree, edited the
  same way and listed in the report; a memory figure or pin that moves only because a type
  grew, updated where it is pinned (`SKEIN_CODEGEN_WRITE=1` where the test offers it).
  If `Reasons` needs a derive or a `const fn` the contract leaves out, add it and say so.
- Not yours: any codec's or schema's body beyond the forced default (the store task), the
  configuration's `parts` and `later` members (the config task), the client wire (the wire
  task), the offering, the parts' check and assembly, and the later choice's checks and use.

## Steps

1. The table of `src/tests/reasons.rs`, failing; then `reasons.rs` until it passes; break
   one bit of `of` by hand, see its row fail, restore.
2. The fields and types of `contracts.md`, sections 2, 3 and 5; `parts_owned_bytes`; the
   claim's store; the core's fills; the limit's name and profiles.
3. The forced readers, `heavy cargo check -p <each package> --tests` until clean.
4. The core's size test; the lib tests above.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and temper's;
   `heavy cargo clippy -p jig-domain-tasks -p jig-domain-core -p jig-domain-people
   -p jig-protocol-config -p jig-protocol-store -p jig-protocol-engine-clients
   -p jig-protocol-tools -p temper-engine-domain -p temper-engine-smith
   -p temper-protocol-store -p temper-service --all-targets -- -D warnings`, adding each
   other package you changed; the focused tests of those packages.
6. Commit (why: the plan's types exist, so its store, configuration, wire, offering, parts
   and later choice build in parallel); push your branch.

## Report

As the implementer's text says, and also: each forced site edited, by file; each moved pin
with its old and new value; anything of the contract you had to add.

## Answers

2026-11-07, to run `per-wake/surface-1`.

- **The override arm for `tasks.charter_parts`:** yes. In
  `jig/crates/jig-protocol-config/src/overrides.rs`, bind `charter_parts` in the
  destructure and add `b"charter_parts" => *charter_parts = count(value)?,` beside
  `charter_documents`: a named limit takes an override, and this task's acceptance
  (the config lib tests pass) needs it. Nothing else in that file.
- **`large_enum_variant` on people's `Event`:** yes, one scoped
  `#[expect(clippy::large_enum_variant, reason = "...")]` on `Event` in
  `jig/crates/jig-domain-people/src/boundary.rs`, as the crate and its neighbours already
  do for inline requests. No boxing: `LlmChoice.later` stays `Option<LlmLater>` as the
  contract gives it.
- **`Default` on `Reasons`:** neither the derive nor a handwritten impl. `Default` is
  outside skein's `programming-model.md`, 10.2's derives, and no task of this plan calls
  it; `Reasons::EMPTY` is the empty value. `notes/contracts.md`, section 1 now derives
  `Clone, Copy, PartialEq, Eq, Hash, Debug` only. Drop `Default` from `reasons.rs`.
- Resume from the pushed branch; the three changes above are the only additions to scope.

2026-11-07, to run `per-wake/surface-2`.

- **`large_enum_variant` on temper's `WorkerInput`:** yes, one scoped expectation on
  `WorkerInput` in `temper/crates/temper-engine-domain/src/boundary.rs`. It is a forced
  consequence of `LlmChoice.later` reaching the decoded call, and the task can't meet its
  clippy acceptance without it. Write it the way the same file's `ProposedAction` does,
  under `cfg_attr(not(target_arch = "wasm32"), expect(clippy::large_enum_variant,
  reason = "the worker boundary carries the decoded call whole, its later choice with
  it"))`, because the variants' sizes differ on wasm32 and a bare `expect` could go
  unfulfilled there. No boxing, and no change to its fields.
- A further `large_enum_variant` that clippy names on a temper or jig enum, if it grows
  only because it carries `LlmChoice`, `CharterPart` or the claim's fields, is covered by
  the same rule. Put the expectation on that enum, written the same way, and list it in
  the report. Anything else is a question.
- Resume from `e9cd8f27dd`, then finish the verification the run lists as not run.
