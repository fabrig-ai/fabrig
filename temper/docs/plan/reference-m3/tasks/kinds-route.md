+++
id = "kinds-route"
part = "kinds-root"
kind = "feature"
needs = ["kinds-held", "kinds-codec", "apps-surface", "change-call-llm"]
files = ["temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/tools.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/tests/kinds.rs", "temper/crates/temper-engine-domain/src/tests.rs", "temper/crates/temper-engine-smith/src/settings.rs", "temper/crates/temper-service/src/clients/translation.rs", "temper/crates/temper-engine-domain/src/admission.rs", "temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-smith/src/tests/settings.rs", "temper/crates/temper-service/src/tests/fixtures.rs", "temper/tests/engine/domain/tests/suite/settings.rs", "temper/tests/system/tests/suite/running_revision.rs"]
effort = "high"
minutes = 120
+++

# kinds-route: the root routes C2's asks for kinds 1 to 3 to the forge, typed

## Goal

The root gives the forge each ask jig's core makes of an item of temper's kinds (contract
C2), as typed values, and carries the forge's answers back within the same decision
(`root.md`, 5.6): judging a change's items, keeping a committed change's, keeping and dropping
a proposal's. It keeps, for the request's decision, the typed values its entrance decoded; the
entrances that fill them (a party's change or proposal, a run's `propose_settings`) come with
the settings file (`kinds-ctl`), so here they carry none, and every route is held by tests that
hand the root its asks.

Acceptance:

- `temper_engine_domain::PartyInput::{ChangeSettings, ProposeSettings}` and
  `Tool::ProposeSettings` gain `kinds: Box<[KindValue]>`, a new boundary type
  `pub struct KindValue { pub kind: u32, pub scope: u32, pub number: u32, pub value:
  temper_engine_domain_forge::settings::SizedValue }` (exported as `PartyInput` is): the typed
  value of each `Set` item of kinds 1 to 3 the request carries, decoded at the entrance. Their
  producers (`temper-service`'s `clients/translation.rs`, `temper-engine-smith`'s
  `settings::proposal`) give `Box::default()`; their consumers in the root pass it on.
- A new module `route/kinds.rs`, module doc citing `root.md`, 5.6 and
  `protocol/configuration.md`, 13.2:
  - The root keeps the routed request's `kinds` (`Domain` field, one slot) from the event that
    carries it until that event's decision completes, then clears it.
  - `Ask::JudgeSettings { owner, revision: _, proposal, items }` (the field set
    `apps-surface` gives; `proposal` where it has it) for the forge's connector: each item to
    a `forge::settings::Asked`: `Edit::Set(_)` takes the slot's typed value for its kind,
    scope and number (with `proposal`, the forge's kept edit, `proposal_settings`); `Remove`
    and `Reset` as they are. `forge.judge_settings(&bounds, &asked)`, translated one for one:
    `Accept { size, charter }` to `Judged::Accept { size, charters }` (`charters` the one
    charter, or empty); `Same` to `Same`; `Refuse(r)` to `Refuse { why: r.words() }`. A `Set`
    with no typed value is `Refuse { why: Refusal::Value.words() }`. Pushed as
    `connector::Event::SettingsJudged { owner, judged }` to the core in the same step, as a
    requirement's `Verdict` is.
  - `Ask::KeepSettings { revision: _, proposal, items }`: the same translation, then
    `forge.keep_settings(env, proposal, &asked, out)`, its saves and erases becoming the root's
    writes as the forge's others do.
  - `Ask::KeepProposalSettings { proposal, items }` and `Ask::DropProposalSettings { proposal
    }`: `keep_proposal_settings` and `drop_proposal_settings`.
  - The bounds: `forge::settings::Bounds { landing_rules: forge.landing_rules, gates:
    forge.change_policy.gates, roles: forge.roles, segments: authority.segments,
    segment_bytes: authority.segment_bytes.min(forge.name_bytes), charters: tasks.charters }`
    from the root's limits (the smaller segment bound, so that every rule the forge keeps
    fits the store's segment, whose bound is `forge.name_bytes`, `kinds-store`).
  - `Ask::SaveSettingsDefaults` stays as `apps-surface` left it (`kinds-start`).
- `forge_route.rs`: `apps-surface`'s `unreachable!` arm for the asks above becomes a call
  into `route/kinds.rs`; nothing else.
- Tests in `src/tests/kinds.rs` pass (Steps); the crate's focused tests pass.

## Inputs

- `temper/docs/design/domain/root.md`, 5.6 (all four bullets); `temper/docs/design/protocol/
  configuration.md`, 13.2 (**Judged by the forge**, **Kept by the forge**).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C1 and C2 (with the
  placement `apps-surface` gives: `jig_domain_core::settings::application::{Item, Edit}`).
- `temper/docs/plan/reference-m3/tasks/apps-surface.md` (the asks' fields as merged).

## Owns and may change

- Owns `route/kinds.rs` and `src/tests/kinds.rs`, new. `route.rs`, `route/tools.rs`,
  `boundary.rs`, `domain.rs`: the field, the slot, the pass-through, one `mod` line; nothing
  else. `forge_route.rs`: the one arm. `temper-engine-smith/src/settings.rs` and
  `temper-service/src/clients/translation.rs`: `kinds: Box::default()` at each literal; the
  other header files (`admission.rs`, `translate.rs`, the tests and worlds that build or match
  these variants): the field alone, as the compiler names them.
- Options: if the asks' `owner` token or another field `apps-surface` added must be echoed,
  echo it as C2 says and name it in the report. If the core asks in a later step than the
  event's (the slot would be cleared), keep the slot to the decision's end instead, say where.
- Not this task's: the stock, the start's defaults and their save (`kinds-start`); the
  entrances' decoding (`kinds-ctl`); the root's gates and requires (`kinds-switch`).
- Other plans: reference-m2, -m5, -m6, -m7b change `forge_route.rs`: the one arm only.

## Steps

1. The boundary type and fields; the slot; the module; the arm.
2. Tests in `src/tests/kinds.rs`, a root built as `src/tests/release.rs` builds one, with the
   forge given the stock of `kinds-held`'s tests, asks pushed as `Request::Ask` are there:
   - `a_judge_ask_is_answered_in_the_same_step`: a project rule `Set` with its typed value in
     the slot: `SettingsJudged` with one `Accept` and its size;
   - `a_refusal_carries_the_forges_words`: kind 3 charter 0: `Refuse` with `Charter`'s words;
   - `a_set_without_its_typed_value_is_refused_as_another_kind`;
   - `a_keep_ask_writes_the_forges_rows`: the `Change` row saved in the decision's writes;
   - `a_proposal_is_kept_judged_from_its_kept_values_and_dropped`.
   Narrow: `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/kinds::/)'`
   (check the count: five).
3. fmt, old forms, clippy on the three crates; their focused tests; `heavy cargo check
   -p temper-engine-domain-world -p temper --tests`.
4. Commit ("Route the forge's kinds' asks through the root, typed") and push.

## Report

The implementer's text's report, plus: where the slot is cleared, and the asks' fields as
`apps-surface` gave them.
