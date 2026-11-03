+++
id = "kinds-switch-charters"
part = "kinds-root"
kind = "feature"
needs = ["kinds-switch-rules", "change-root", "change-story"]
files = ["temper/crates/temper-engine-domain/src/translate.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs", "temper/crates/temper-engine-domain/src/route/asks.rs", "temper/crates/temper-engine-domain/src/route/policy_translate.rs", "temper/crates/temper-engine-domain/src/tests/**", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/tests/**", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/tests/**", "temper/tests/engine/domain/src/walking.rs", "temper/tests/engine/domain/src/oracle/config.rs", "temper/tests/engine/domain/src/forge_routes.rs", "temper/tests/engine/domain/tests/suite/forge_routes/reviews.rs", "temper/tests/engine/domain/tests/suite/forge_routes/kind_charters.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/tests/suite/settings/b2.rs", "temper/tests/system/src/composed/agreement.rs", "temper/docs/design/domain/root.status.md", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 120
+++

# kinds-switch-charters: a change's charters read from kind 3, and held when unoffered

## Goal

The change's charters come from the forge's kind 3 as it holds it (C3; `configuration.md`,
13.2, **The forge's charters are read when a change is made**): `Config.forge_charter` and the
review charter fixed at 4 go. And a change whose charter of a role the newest settings no
longer offer, or whose LLM choice for a role its charter's models no longer allow, is held
when it next needs a delegate of that role, saying so (`forge.md`, 8.9; this folds in the
change family's uncut `change-held`).

Acceptance:

- `temper_engine_domain::Config.forge_charter` and `RootConfig.forge_charter` go. Every reader
  moves to `domain.forge.charters()` (C3): the change's start in `route/forge_route.rs`
  (`start_change`, as `change-root` left it: the producing charter is the one `change` names,
  else `charters().change`; the review charter `charters().review`, no longer the literal 4),
  the procedure's authority's executors and the run reserve floors that read
  `config.forge_charter` today, and any other the compiler names. The bridge stock of
  `kinds-switch-rules` (`policy_translate::forge_stock`) takes kind 3 from its own arguments,
  `change` and `review`, which a world gives where it gave `forge_charter`.
- Worlds: every `Config` literal and builder that set `forge_charter` gives the forge's kind 3
  through `kinds_stock` (or the bridge's arguments) instead, keeping its value and review
  charter 4 (the domain world's `walking.rs`, `oracle/config.rs`, `forge_routes.rs`'
  `restart_with_charter`, `reviews.rs`' `StoryLimits.forge_charter`, `settings/b2.rs`;
  `resume_fixture.rs`; `tests/system/src/composed/agreement.rs`; temper-service's `system.rs`,
  `startup/checks.rs`, `tests/welcome_policy.rs`). The startup check that the forge's charter
  is one of the core's goes: jig's reference check of kind 3 refuses it now (`apps-start`).
- A charter 0 (kind 3 unset) refuses a change's start as an unoffered charter does.
- **Held:** a delegate's step of a change refused by jig because its charter is not offered by
  the newest settings, or because its LLM choice names a model its charter's models do not
  allow (`jig_domain_tasks::Refusal::Llm { why: LlmFault::Charter, .. }`, C5), holds the change
  (`tasks::Event::Hold`, as `refused_step` holds one today), its hold's words the forge's:
  `the change's ROLE charter N is no longer offered; cancel the change or make it again`, or
  `the change's ROLE choice names model M, which charter N no longer allows; cancel the change
  or make it again` (ROLE `producer` or `review`). In `route/asks.rs`' `StepRefused` arm,
  beside its `Denied` branch; a refusal of any other cause steps again as today.
- Status: `root.status.md` 6 loses "the producing charter is the configured forge charter and
  the review charter is fixed at 4 (`temper-engine-domain/src/route/forge_route.rs`)" and
  whatever of "a change's delegates under the charters it was made with" `change-root` left;
  its first entry loses "and the forge's charter out of `Config`" and "They are `Config`'s
  `landing` and `forge_charter`, read at start,"; `forge.status.md` 8.3 loses "the review
  charter, kind 3's number 2, is fixed at charter 4".
- Tests: a new domain world story file `suite/forge_routes/kind_charters.rs` (`mod` line in
  `suite/forge_routes.rs`): `a_change_reads_the_forges_charters_when_made` (kind 3 number 2
  changed to another charter: the next change's reviews run under it, a live one's keep
  theirs); `a_change_whose_charter_is_no_longer_offered_is_held` (its producer's charter
  dropped from the rules' delegation by a change of settings: its next producing delegate
  holds it, the words above). The world's suite passes: `heavy cargo nextest run -p
  temper-engine-domain-world --test suite`; `-p temper-engine-domain --lib`; `-p
  temper-service --lib`; `heavy cargo check --workspace --tests`.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.2 (**The forge's charters are read when a
  change is made**) and 13.6 step 2; `temper/docs/design/domain/forge.md`, 8.1 and 8.9 (**A
  change keeps its charters and its LLM choices**, as the change family wrote it);
  `temper/docs/design/domain/root.md`, 6 (**A change's delegates**).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C3, C4 and C5.

## Owns and may change

- Owns the files in the header for the edits above; in `forge_route.rs`, the charter reads
  alone.
- Options: if jig refuses an unoffered charter as `StepRefusal::Denied` (an authority finding
  on the delegation) rather than `Refused`, hold from that branch with the same words and say
  so. If a world's story asserts review charter 4 by literal, keep its value through kind 3
  number 2.
- Not this task's: the document's `charter` member (`kinds-document`).

## Steps

1. The field's removal and the reads; the bridge's arguments; the worlds.
2. The hold.
3. The story file; the narrow tests first (`heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E 'test(/kind_charters::/)'`, check the count:
   two); then the packages in Goal.
4. fmt, old forms, clippy over every package changed.
5. Commit ("Read a change's charters from the forge's kind 3") and push.

## Report

The implementer's text's report, plus: which jig refusal an unoffered charter gives.
