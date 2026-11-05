+++
id = "person-facts"
part = "person-facts"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/src/dismissed.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/views.rs",
  "temper/crates/temper-engine-domain-forge/src/limits.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/dismissed.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/views.rs",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/route/applications.rs",
  "temper/crates/temper-engine-domain/src/tests/dismiss.rs",
  "temper/crates/temper-engine-domain/src/tests/documents.rs",
  "temper/crates/temper-protocol-store/schema/store-v11.schema",
  "temper/crates/temper-protocol-store/src/generated/v11.rs",
  "temper/crates/temper-protocol-store/src/records.rs",
  "temper/crates/temper-protocol-store/src/keys.rs",
  "temper/crates/temper-protocol-store/src/tests/**",
  "temper/crates/temper-protocol-store/golden/**",
  "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema",
  "temper/crates/temper-client-protocol-wire/src/generated/v1.rs",
  "temper/crates/temper-client-protocol-wire/src/lib.rs",
  "temper/crates/temper-client-protocol-wire/src/tests.rs",
  "temper/crates/temper-client-protocol-wire/golden/**",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/startup/checks.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/crates/temper-service/src/tests/root_checks.rs",
  "temper/crates/temper-client-domain/src/forge/dismissed.rs",
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests/dismissed.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 85
+++

# A person's facts

## Goal

The per-person family of `Dismissal` becomes `PersonFact`, and holds a second kind: that the person started a goal from a chat's document. Same request, same watch, a larger bound.

Acceptance, as `notes/contracts.md`, *A person's facts*, temper's `domain/forge.md`, 5.5, and `client/README.md`, 4.6:

- `PersonFact` in the engine's domain, the store, the wire and the client's domain; `Dismiss`, `Dismissals`, the store's `Dismissed` and the subject `dismissed` keep their names, so their other readers needn't change. Both schemas change in place.
- `erase_project` matches both kinds; `domain.rs`' range for "is the party's To begin held" still starts at `ToBegin` first; `route/applications.rs` scopes `GoalFrom` to its `project`, as `ToBegin`.
- **The bound:** `limits.dismissed()` is 8 times what it was; `DISMISSED` 8,192 on the wire; the start's check on `judge_projects` stays at 1,024, an eighth of that (`startup/checks.rs`). `dismiss` answers `TooLarge` at `dismissed.len() == capacity`, or at the party's own count at `DISMISSED`, counted over the party's key range. `family.rs`' document is always whole: `Dismissals` of a party's rows fits.
- **The client's side:** `forge/dismissed.rs`: `dismissed(fact)` as it is, `goals_from(project, chat)` as the note has it; `Home`'s `skip` path and its `skip_refused` stay `ToBegin`'s: a `TooLarge` answer to a `GoalFrom` changes no page and says nothing.
- `temper/docs/design/domain/forge.status.md`'s 5.5 entry goes.
- The tests below, each having bitten.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/domain/forge.md`, 5.5; `temper/docs/design/client/README.md`, 4.6, *`Dismiss`* and *A person's facts*; `temper/docs/design/protocol/store.md`, section 2.
- `temper/crates/temper-client-domain/src/pages/home.rs`, `Facts::to_begin`, `skip`.

## Owns and may change

Owns, as `files` lists them.

May also change, for the type's new name or kind alone: `temper/tests/client/src/front/dismissed.rs`, `tests/client/tests/suite/dismissed.rs`, `suite/to_begin.rs`; `temper/tests/codecs/tests/suite/fuzzy/store.rs`, whose generator gains `GoalFrom`; `temper/crates/temper-client-view/src/tests/fixtures.rs` and `tests/home.rs`; `temper/tools/gallery/src/samples.rs`.

Generated: the codecs' and their goldens with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`.

Options, each pre-approved:

- **Eagerly allocated rows:** `domain.rs` allocates `limits.dismissed()` at start. If a heap pin or a worst-case figure moves, change its number, name it.
- **`GoalFrom`'s `chat` and `goal`** aren't checked to be real or the party's: they're hints the party keeps about itself, read back only by that party, and a page reads each through its own reads. Say nothing else about them.
- **A second `Dismiss` of the same fact** answers dismissed with nothing written, as today.

## Steps

1. The domain and its tests: `goal_from_is_saved_and_marked`, `a_second_goal_from_writes_nothing`, `too_large_at_the_family_bound`, `too_large_at_the_partys_own_bound`, `erase_project_erases_both_kinds`, `another_partys_facts_are_not_answered`.
2. The store and the wire, regenerated, with key-order tests (`keys.rs`: `ToBegin` before `GoalFrom`, party first).
3. The client's domain: `goals_from_reads_the_partys_rows`, `a_refused_goal_from_changes_no_page`.
4. A story in `temper/tests/client/tests/suite/dismissed.rs`: `a_goal_from_reaches_another_client_of_the_party`.
5. `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain -p temper-protocol-store -p temper-client-protocol-wire -p temper-client-domain -p temper-codecs-world`; `-p temper-service --lib`; the client world's `dismissed` and `to_begin` suites; `heavy cargo check --tests -p temper-client-view -p temper-client-gallery -p temper-client-tools -p temper-fake`.
6. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.

## Rebase (2026-11-05, step merge, foreman ui-chat-draft/person-facts/foreman-1)

resolve both document conflicts, preserving the new `at` design and the eightfold bound.
The merge conflicts with integration commit `d9fa8d0a036610acceb43d4621f9ee669463ed55` (“ui-chat-draft: the client's batch”).
In `temper/docs/design/domain/forge.status.md`, remove only the completed 5.5 gap for `PersonFact`, `GoalFrom` and its bound; retain integration's separate `5.5: GoalFrom's at` gap.
In `temper/docs/design/protocol/store.md`, section 2, combine both edits to family 34: keep integration's `GoalFrom { project, chat, goal, at }` and this branch's bound of `8 times` the forge's `bindings` times `judge_projects`.
Preserve integration's key-order description ending “the project, the chat, the goal and when.”
Keep the remaining document content unchanged. The `at` implementation remains a recorded gap; this rebase needs no schema or code extension.
