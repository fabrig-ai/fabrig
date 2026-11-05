+++
id = "ways-kind"
part = "ways-kind"
kind = "feature"
needs = ["project-templates"]
files = [
  "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema",
  "temper/crates/temper-client-protocol-wire/src/generated/v1.rs",
  "temper/crates/temper-client-protocol-wire/golden/**",
  "temper/crates/temper-protocol-store/schema/store-v11.schema",
  "temper/crates/temper-protocol-store/src/generated/**",
  "temper/crates/temper-protocol-store/golden/**",
  "temper/crates/temper-protocol-store/src/records/settings.rs",
  "temper/crates/temper-protocol-store/src/tests/settings.rs",
  "temper/crates/temper-engine-domain-forge/src/settings.rs",
  "temper/crates/temper-engine-domain-forge/src/kinds.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/settings.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/kinds.rs",
  "temper/crates/temper-engine-smith/src/kinds.rs",
  "temper/crates/temper-engine-smith/src/tests/kinds.rs",
  "temper/crates/temper-engine-domain/src/route/kinds.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs",
  "temper/crates/temper-engine-domain/src/tests/kinds.rs",
  "temper/crates/temper-service/src/stock/kinds.rs",
  "temper/tests/engine/domain/tests/suite/settings/kinds.rs",
]
effort = "high"
minutes = 150
+++

# Kinds 6 and 7, and a project's read of its own kinds

## Goal

temper's forge declares kind 6, a project's questions, and kind 7, a project's ways: judged, kept, erased
with their project. A `change` naming no route takes kind 6. A party with a role in a project reads that
project's kinds. Nothing in the client reads them yet: `ways-session` does.

Acceptance:

- **Declared:** `temper-service/src/stock/kinds.rs` declares 6 and 7, `KindScope::Project`, `per_scope` 1,
  `charters` 0, `value_bytes` each its encoding's size. There is no stock row for either
  (`protocol/configuration.md`, 13.4): `forge::setting` answers none.
- **Judged by the forge:** scope 0 is `Refusal::Scope`; a number other than 1 is `Refusal::Number`; a value
  of another kind is `Refusal::Value`; a removal or a reset is good.
- **Judged by the root:** in `route/kinds.rs`' `judge`, beside `approval_refusal`, a `Ways` with a `way`
  neither 0 nor a number in `domain.core.project_templates(scope)` is refused with
  `settings::Refusal::Way.words()`, in the proposal's and the direct change's path alike.
- **Same:** an item leaving its value as stored answers `Same`, as every kind's does.
- **Kept and erased:** `Domain::keep_settings`, the replaced slot and a restore work as for kind 2;
  `erase_project_settings` erases kinds 2, 6 and 7 of the project.
- **The route:** in `forge_route.rs`, the change's route is `change.questions`, or with none, kind 6 of its
  repository's project: `Domain::project_questions`. The foreman's task count, its delegation kind and
  parameter 14 all follow that one value, so a project's `foreman` behaves as a `change` naming it. A
  change keeps it on its row: a later change of kind 6 reaches only later changes.
- **The read:** `read_refusal` takes the query. In the deployment's scope, a role in the deployment, as
  today. In a project's scope, a role in that project, and only a `ForgeRead::Setting` or `RevisionSetting`
  of a kind 2, 6 or 7 whose `scope` is that project; anything else answers `Why::Unknown`.
- `protocol/configuration.status.md`: this closes its Not yet on kinds 6 and 7 (but its *13.9*, which
  `ways-ctl` closes); the sentence on 4 and 5 says the forge declares neither.
- The tests below pass and bite.

## Inputs

- `temper/docs/plan/ui-settings/notes/common.md`; `temper/docs/plan/ui-settings/notes/contracts.md`, *Kinds 6
  and 7*.
- `temper/docs/design/protocol/configuration.md`, 13.2, 13.4; `temper/docs/design/protocol/store.md`, section 2,
  family 20; `temper/docs/design/client/forge.md`, section 9, the row of temper's settings.
- `temper/docs/design/domain/agent.md`, 4.3.1, the `questions` member.

## Owns and may change

Owns `files`. May also change:

- **Compiler-forced arms and fields of the new `Value` variants,** each edited only for that:
  `temper-engine-domain-forge/src/lib.rs` (`row_bytes`: no heap), `restore.rs`, `tests/*`;
  `temper-client-tools/src/settings.rs` and `settings/value.rs`, whose `valid` and render read a kind's value
  (decode kinds 6 and 7 there too, so `settings show` doesn't call them malformed); `temper-settings-form/src/kinds.rs`
  only if the compiler forces it. Their names in ctl are `ways-ctl`'s.
- **The bound:** `settings::rows` in `temper-engine-domain-forge/src/settings.rs` counts each project's two
  values in `values`. If a fit check on the settings document or the forge's rows trips, with literals in
  `temper-service/src/limits.rs`, `temper-service/src/tests/startup.rs`, `temper-engine-domain-forge/src/tests/restore_fit.rs`
  or `temper/docs/help/reference/limits.md`: follow the check's formula and regenerate; a limit's figure that
  must rise is named in the report. No budget or timeout moves.
- **The world's arms:** `temper/tests/codecs/**`, `temper/crates/temper-fake/src/check.rs`,
  `temper/tests/engine/forge/src/world.rs`, where a `SettingValue` or `Value` is matched.
- **Design status:** `temper/docs/design/protocol/configuration.status.md`, `protocol/store.status.md`, for the
  lines this closes.

Pre-approved:

- If `route/kinds.rs`' `read_refusal` needs the `ForgeRead` by reference, change its signature; it has one caller.
- If a restore's `RestoredEdit::Set` of kind 7 names a template no longer offered: judge it as a set is judged.
  A way that stops being offered after it was set stays stored: the client reads it as the first (`ways-session`).
- If `temper-service`'s family rejects a `Setting` read in a project's scope before the root: it passes `scope`
  through at `family.rs`, `Read` arm, today. If that has changed, make it pass, and say so.

## Steps

1. **Schemas:** by `notes/contracts.md`, then regenerate (`notes/common.md`, *Codecs*).
2. **Store:** `records/settings.rs` both ways; `tests/settings.rs`' round trip takes both new values.
3. **Forge:** `settings.rs` (`Kind`, `Value`, `ProjectWays`, `Refusal`, `judge`, `rows`), `kinds.rs`
   (`project_questions`, `erase_project_settings`, the arms).
4. **Smith:** `temper-engine-smith/src/kinds.rs`' `decode`, `encode` and `size`. Decode through the wire's
   records, as a charter's.
5. **Root:** `route/kinds.rs` (`judge`, `read_refusal`), `forge_route.rs`. `temper-engine-domain` holds `ChangeQuestions`
   and the forge `Route`: map at the one place the route is taken, before line `let foreman = ...`, and make
   the two later readers of `change.questions` read that value. `change.questions.is_some()` in the plan's gate
   (`plan` in `forge_route.rs`) still asks of what the maker named.
6. **Service:** `stock/kinds.rs`.
7. **Tests:**
   - Forge, `tests/settings.rs`: `a_projects_kinds_are_judged`: table of scope, number, value, removal.
   - Forge, `tests/kinds.rs`: `a_deleted_project_loses_its_questions_and_ways`;
     `project_questions_reads_none_until_set_then_the_route`.
   - Root, `tests/kinds.rs`: `a_way_the_project_does_not_offer_is_refused` (0 good, offered good, other refused
     naming it); `a_project_reads_its_own_kinds_with_a_role_there` (member reads 6 and 7 in its scope; no role;
     another project's scope; kind 1 and kind 3 in a project's scope; `ProposalItems` in a project's scope, all
     `Why::Unknown`; a deployment role in the deployment's scope, as today).
   - Root, `route/forge_route/tests.rs`: `a_change_naming_no_route_takes_its_projects_questions`;
     `a_named_route_wins_over_the_projects`; `a_later_change_of_the_questions_reaches_only_later_changes`;
     a project's `foreman` grants the foreman's charter.
   - World, `temper/tests/engine/domain/tests/suite/settings/kinds.rs`:
     `a_projects_ways_are_changed_by_its_owner_and_read_by_its_member` (by `apply`, then a read as a member; a
     member who is no owner is refused by jig); `a_proposal_of_a_projects_ways_is_kept_then_accepted`, by `propose`.
8. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy and nextest for every package you changed and their
   direct dependents, `temper-settings-form` and `temper-client-tools` among them, `heavy cargo check -p temper --tests`.
   Commit and push.

## Report

As the implementer's text says, with the limit figures that moved, if any.
