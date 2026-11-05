+++
id = "retry-reason"
part = "retry-reason"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-tasks/src/**",
  "jig/crates/jig-protocol-store/schema/tasks-v9.schema",
  "jig/crates/jig-protocol-store/src/generated/tasks_v9.rs",
  "jig/crates/jig-protocol-store/src/codec/tasks/**",
  "jig/crates/jig-protocol-store/src/tests/codec/**",
  "jig/crates/jig-protocol-store/src/tests/fixtures/record.rs",
  "jig/crates/jig-protocol-store/golden/tasks/**",
  "jig/tests/tasks/**",
]
effort = "high"
minutes = 120
+++

# A task keeps why its preparation failed, and can count it

## Goal

The tasks child can only pause a preparing task, unboundedly and without a word (`run.rs`, `pause`). Give it the two events of `notes/contracts.md`, 2: a pause that says why, and a counted failure that ends in `Hold::Failures(class)` naming the reason. Nothing sends them yet: `brief-counts` does. `PreparationFailed { task }` stays, unchanged, until then.

Acceptance:

- `PreparationPaused`: as `PreparationFailed` is today, a `Preparing` task only, no try spent, with `RetryNotice::Refused { count, reason: Some(..) }`.
- `PreparationCounted`: a `Preparing` task's failure of `class`, within `limits.retries.of(class)`, with `RetryNotice::Failed { .., reason: Some(..) }`; past the allowance `Phase::Held { why: Hold::Failures(class), source: Some(HoldSource::Preparation { words }) }`, the notice kept.
- A reason is cut to `REASON_BYTES` at a character's edge as the child takes it; a stored one past it is refused at restore, by name.
- A run's failure keeps `reason: None` and `source: None`: its path doesn't change.
- A held task's document says the reason: `HoldLine.words` (`jig-domain-core`, `documents/task_tree.rs`, `hold`).
- `tasks-v9` round-trips both, goldens regenerated from `skein-codegen`'s output, with the full fixture carrying a reason.
- The reason survives a restart: a held task's and a backing-off task's come back whole.
- `jig/docs/design/domain/tasks.status.md`: its "5.2, 5.5" **Not yet** entry stays: `brief-counts` closes it.

## Inputs

- `temper/docs/plan/brief-failure/notes/common.md`, `temper/docs/plan/brief-failure/notes/contracts.md`, 2.
- `jig/docs/design/domain/tasks.md`, 5.2 and 5.5; `jig/docs/design/domain/engine.md`, 9.1.
- `jig/AGENTS.md`.

## Owns and may change

Owns the files in `files`.

May also change, each only as the compiler forces it:

- `RetryNotice` loses `Copy`: `jig/crates/jig-domain-core/src/documents/chip_phase.rs` (`retry: row.retry_notice.clone()`), `jig/crates/jig-domain-core/src/tests/documents/chip_phase.rs`, `jig/crates/jig-domain-core/src/tests/runs/notes_room.rs`, `jig/crates/jig-protocol-engine-clients/src/translate/chips.rs`. `chips.rs`'s `retry` matches with `..` and takes a reference: the wire gains the reason in `reason-shown`.
- `HoldSource::Preparation`'s arms: `jig/crates/jig-domain-core/src/documents/task_tree.rs` (`hold_source` and `hold_account` give `None`, as for no source; `hold` gives it as `words`), `jig/crates/jig-domain-core/src/reads/run_task.rs`, `jig/crates/jig-domain-core/src/effects.rs`, `jig/crates/jig-domain-core/src/effects/connector.rs`, `jig/crates/jig-domain-core/src/runs/delegation.rs`, if any matches every source.
- The new events' arms: `jig/crates/jig-domain-core/src/routing/flight.rs` (with `PreparationFailed`), `jig/tests/tasks/src/world.rs`.
- temper's lines naming these, a departure from "a branch stays in its tree" for a required field: `temper/crates/temper-engine-domain/src/route/forge_route/tests.rs`, `temper/crates/temper-engine-domain/src/route/asks.rs`, `temper/tests/system/src/composed/review.rs`, if a pattern needs it. No behaviour of temper's changes.
- `jig/tests/store-codecs/**`, `jig/tests/codec-families.rs`: literals and generated sizes.

Options, pre-approved:

- `stored/failure.rs`'s `RestoreBound` gains one variant, `PreparationReasonBytes`, for both the notice's reason and the hold's words; if an arm elsewhere needs it, add it.
- `owned.rs` counts a notice's reason among a task's owned bytes, as it does an account's words. If `retry_notice`'s bytes aren't counted by a seam you can find, count them in `phase_bytes`'s neighbour and say which.
- A byte-count or size fixture that moves with the schema is regenerated (the implementer's text).
- `valid` in `hold_origin.rs` takes `Preparation` only with `Hold::Failures(_)`, non-empty, within `REASON_BYTES`.

Leave `PreparationFailed`, `PreparationUnread`, `preparation_failed` and `preparation_unread`'s behaviour alone. `jig/crates/jig-protocol-store/schema/core-v6.schema` moves on another draft: this task doesn't need it.

## Steps

1. Build `notes/contracts.md`, 2 in the tasks child: `pause` and `failure` take `reason: Option<Box<[u8]>>`; add `preparation_paused` and `preparation_counted` beside `preparation_failed`, under the same `domain.ready()` and `Preparing` guards. A failing test first, in `jig/tests/tasks/tests/suite/lifecycle.rs`: a preparing task counted past its allowance ends `Held` with `Hold::Failures(Permanent)` and the words; on today's code it does not compile, so the first red is its assertion once the types are in.
2. The schema of `notes/contracts.md`, 2, the codec, `skein-codegen`, goldens, `read_retry_notice`.
3. `HoldLine.words`, and a test in `jig/crates/jig-domain-core/src/tests/documents/task_specification.rs`.
4. A restart test: `jig/tests/tasks/tests/suite/` for a held and a backing-off reason read back, with the restore bound's refusal.
5. A test for the cut: 200 bytes of multi-byte words keep at most `REASON_BYTES`, whole characters.
6. `cargo fmt`; old forms for jig; `heavy cargo clippy -p jig-domain-tasks -p jig-protocol-store -p jig-domain-core -p jig-protocol-engine-clients --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-tasks -p jig-protocol-store -p jig-tasks-world -p jig-store-codecs-world`, then `-p jig-domain-core --lib`, and `heavy cargo check -p temper-engine-domain -p temper-service --tests`. Name the worlds as their manifests have them. Check each count.
7. Commit, push.

## Report

As the implementer's text has it, with the rule each new test bit on.
