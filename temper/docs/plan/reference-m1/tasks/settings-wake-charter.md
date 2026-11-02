+++
id = "settings-wake-charter"
part = "settings-wake"
kind = "feature"
needs = ["settings-wake-pace", "goal-jig-holdings", "goal-jig-config"]
files = ["jig/crates/jig-domain-core/src/settings/charters.rs", "jig/crates/jig-domain-core/src/settings/check.rs", "jig/crates/jig-domain-core/src/runs/delegation.rs", "jig/crates/jig-domain-core/src/routing/party/create.rs", "jig/crates/jig-domain-core/src/routing/run/delegation.rs", "jig/crates/jig-domain-core/src/admission/amendments.rs", "jig/crates/jig-domain-core/src/tests.rs", "jig/crates/jig-domain-core/src/settings/tests.rs", "jig/crates/jig-domain-people/src/batch.rs", "jig/crates/jig-domain-people/src/tests/fixtures.rs", "jig/crates/jig-protocol-store/schema/settings-v1.schema", "jig/crates/jig-protocol-store/schema/people-v14.schema", "jig/crates/jig-protocol-store/src/generated/**", "jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/src/codec/people/**", "jig/crates/jig-protocol-store/src/tests/settings.rs", "jig/crates/jig-protocol-store/golden/settings/**", "jig/crates/jig-protocol-store/golden/people/**", "jig/crates/jig-protocol-config/src/charters.rs", "jig/crates/jig-protocol-config/src/tests/charters.rs", "jig/crates/jig-protocol-config/src/tests/fixtures.rs", "jig/crates/jig-protocol-engine-clients/src/translate/operations.rs", "jig/crates/jig-protocol-engine-clients/src/translate/settings/change.rs", "jig/crates/jig-protocol-tools/src/nested.rs", "jig/tests/core/tests/suite/charters.rs", "temper/crates/temper-engine-smith/src/nested.rs", "temper/crates/temper-engine-domain/src/route/goal.rs", "temper/crates/temper-service/src/stock.rs", "jig/docs/design/domain/tasks.status.md", "jig/docs/design/protocol/configuration.status.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 110
+++

# settings-wake-charter: a charter's default wake policy

## Goal

A charter carries a default wake policy, kept with it in the settings and read from
configuration, and an agent task made under the charter by a batch entry that names no policy
takes it: a party's `Create` (a template's task among them, since the client fills none) and
an agent's `delegate` alike. Today every such entry gets the hard-coded
`WakePolicy::DEFAULT` in the protocol layer, before the charter is known.

Acceptance:

- `jig_domain_core::settings::Charter` has `pub wake: jig_domain_tasks::WakePolicy`, stored
  in the settings family (settings-v1, in place, with goldens) and restored.
- jig's configuration reads a charter's optional `wake` member, spelled as C7's line in
  `notes/contracts.md` says; absent, `WakePolicy::DEFAULT`. A rule out of shape (a zero
  count, age or span; an unknown kind or `results` word; an unknown member) refuses the start
  at its path (`charters[i].wake.news`).
- `jig_domain_people::Member.wake` and the core's `Delegate.wake` are `Option`; `None` is
  resolved in the core where the task is made: an `Executor::Agent { charter }` takes that
  charter's `wake` as the current settings hold it; every other executor, or a charter the
  settings do not hold, takes `WakePolicy::DEFAULT`. A policy named in the entry is kept as
  named.
- A change of settings, or a start's defaults, whose charter's `wake` fails
  `jig_domain_tasks::wake_valid` is refused, the charter's setting with `Fault::Shape`
  (`settings/check.rs`'s shape check, beside the charter's other checks).
- The protocol layer passes `None` through: the client's `NewTask.wake: None`
  (engine-clients' `member`), the `delegate` tool's absent `wake` (`jig-protocol-tools`
  and temper's `temper-engine-smith`).
- temper's stock charters 1, 2 and 4 name `WakePolicy::DEFAULT`, so no stock task's policy
  changes; temper's goal route (`route/goal.rs`) names `None`, taking its charter's.

## Inputs

- `jig/docs/design/domain/tasks.md`, 8.3 (the paragraph "Each charter brings a default
  policy"); `jig/docs/design/protocol/configuration.md`, section 3 (the charters row's
  default wake policy), 3.1 ("a task filled from it wakes by its charter's default wake
  policy") and section 10 (**A charter's default wake policy**: the member's spelling).
- `temper/docs/design/protocol/configuration.md`, 13.4 (the bullet **Each stock charter's
  default wake policy**).
- `jig/docs/design/domain/tasks.status.md` and `jig/docs/design/protocol/configuration.status.md`:
  their **Not yet** words for a charter's default wake policy, which this task removes
  (tasks.status: "a charter's default wake policy: jig's `Charter` carries none, ..."; the
  configuration status's "3, 3.1, 10: a charter's default wake policy" entry), and
  `temper/docs/design/protocol/configuration.status.md`'s "13.4: each stock charter's default
  wake policy" entry.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C7 and its line.

## Owns and may change

Owns the files in the header. The edits there:

- `settings/charters.rs`: the field, doc line "Its default wake policy, which a task made
  under it takes when its batch entry names none (domain/tasks.md, 8.3)."
- settings-v1: `record Charter { number: u32 policy: RunPolicy hosts: Kinds spend: u64
  wake: WakePolicy }`, with `WakeBatch`, `WakePace`, `WakeRule`, `ResultsWake` and
  `WakePolicy` records in that schema, shaped as tasks-v9's (after `settings-wake-pace`);
  `codec/settings.rs`'s `encode_charter`/`decode_charter` carry it. people-v14: `Member`'s
  `wake: option WakePolicy`, and its codec. Regenerate with `SKEIN_CODEGEN_WRITE=1 heavy
  cargo nextest run -p jig-store-codecs-world --test drift`; goldens as the generator asks.
- `jig-protocol-config/src/charters.rs`: `fn wake(...)` reading the member as section 10 and C7 spell it
  (durations as `age_ms` and `span_ms`, as the file's other `_ms` members); the existing
  `check::*` helpers and `What` refusals; the fixture's charter JSON stays without `wake`, and
  new tests add it.
- The core: `routing/party/create.rs` (`new_task`) and `routing/run/delegation.rs`
  (`delegate_validated`, `propose_batch`, `procedure_step`) resolve `None` through one
  function, `fn default_wake(core: &Core, executor: &tasks::Executor) -> tasks::WakePolicy`,
  in `admission/amendments.rs` beside `wake`. Sites that build a `Delegate` or a member with
  a policy of their own (`routing/task.rs`, `start_recurring`) keep `DEFAULT` as `Some` or
  as the `tasks::New` field they fill: they are procedures.
- `jig-protocol-engine-clients/src/translate/settings/change.rs`, `fn charter(value:
  w::Charter) -> c::Charter`: the client wire's charter carries no wake until
  `settings-wake-clients`; give `wake: WakePolicy::DEFAULT` there, with a comment saying a
  client's charter names no wake policy yet. `translate/settings.rs` (core to wire) drops it,
  the wire having no member for it.
- Literal fallout, each a new field or `None` alone: every `core::Charter` literal (the
  survey's list: `jig-domain-core` tests, `jig-protocol-store/src/tests/settings.rs`,
  `jig-protocol-engine-clients/src/tests/settings.rs`, `jig/testing/jig-test-profile`,
  `jig/testing/jig-test-domain`, `jig/tests/engine-clients`, temper's
  `temper-engine-domain/src/tests/resume_fixture.rs`, `temper/tests/engine/domain/src/walking.rs`,
  `temper/tests/system/src/world.rs`) and every `people::Member` and `Delegate` literal in
  jig's and temper's tests. Find them with `heavy cargo check --tests` per package.
- `temper-service/src/stock.rs`: `wake: WakePolicy::DEFAULT` in `fn charter` and
  `fn reviewer`.
- Other plans: reference-m7a changes `jig-domain-core`, `jig-protocol-store` and
  `jig-domain-tasks` on its trunk, and the goal family's tasks of this plan change
  `people::Member` beside `wake` (contract C2's `symbols`), `create.rs` and `delegation.rs`
  (`goal-jig-holdings`) and `jig-protocol-config`'s templates (`goal-jig-config`): this task
  starts from their merged work; keep every hunk to the field. In temper, its files are
  disjoint from the change family's (`change-call` owns `temper-engine-smith`'s `calls.rs`
  and `tests.rs` and `temper-engine-domain`'s `boundary.rs`, `tools.rs`, `host_route.rs`;
  `change-held-release` owns `forge_route.rs`): this task edits `nested.rs` and
  `route/goal.rs` alone there, and a `Delegate` literal elsewhere only in its `wake` field.
- A change of meaning: a task made with no named policy may wake otherwise once a charter
  names a policy; with every charter at `DEFAULT` nothing changes. Run the focused tests of
  `jig-domain-core`, `jig-core-world`, `jig-people-world`, `jig-engine-clients-world`,
  `jig-conformance-world`, and temper's `temper-engine-domain`, `temper-engine-domain-world`
  and `temper-service`.

## Steps

1. The field and its store; the configuration member; `Option` and its resolution; the
   pass-throughs; the literal fallout.
2. Tests:
   - `jig-protocol-config/src/tests/charters.rs`: `a_charter_names_its_default_wake_policy`
     (C7's coordinator policy read back), `a_charter_without_a_wake_member_takes_the_default`,
     `a_charter_wake_with_a_zero_span_is_refused_at_its_path`.
   - `jig-protocol-store/src/tests/settings.rs`: `a_charters_paced_wake_policy_round_trips`.
   - `jig-domain-core/src/settings/tests.rs`: `a_charter_with_a_zero_paced_span_is_refused_as_shape`.
   - `jig/tests/core/tests/suite/charters.rs`, a world whose charter 1 names C7's
     coordinator policy: `a_party_task_naming_no_wake_takes_its_charters_default`,
     `a_delegate_naming_no_wake_takes_its_charters_default`,
     `a_named_wake_policy_is_kept_over_the_charters`,
     `a_procedure_naming_no_wake_takes_the_default`. Each reads the made task's
     `TaskRecord::wake`.
   Narrow: `heavy cargo nextest run -p jig-protocol-config --lib -E 'test(/wake/)'`,
   `-p jig-protocol-store --lib -E 'test(/wake/)'`, `-p jig-core-world --test suite -E
   'test(/wake/)'` (check each count).
3. The implementer's text's checks: fmt, old forms, clippy on every package changed, and the
   focused tests the change of meaning names above.
4. Commit ("Give a charter a default wake policy") and push.

## Report

The implementer's text's report, plus: each literal site changed by file, and whether any
temper test's expected policy moved.

## Answers

2026-11-02. May this task add `pub use wake::valid as wake_valid;` to
`jig-domain-tasks/src/lib.rs`, or does `settings-wake-pace` supply it? `settings-wake-pace`
supplies it: its acceptance requires the root export, and its spec now owns `lib.rs` for that
one line (its `## Answers`). This task does not edit `jig-domain-tasks`. On resuming from
`acd3a1920c`, merge `origin/reference-m1/settings-wake-pace` again; once it carries
`jig_domain_tasks::wake_valid`, call it from `settings/check.rs` as the acceptance says. If the
merged branch still lacks the export, stop and report that, rather than adding it here: two
branches adding the export would collide.
