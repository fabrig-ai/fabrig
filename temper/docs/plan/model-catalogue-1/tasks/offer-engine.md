+++
id = "offer-engine"
part = "offer-engine"
kind = "feature"
needs = ["offer-surface"]
files = [
  "jig/crates/jig-domain-core/src/documents.rs",
  "jig/crates/jig-domain-core/src/documents/small.rs",
  "jig/crates/jig-domain-core/src/documents/catalogue.rs",
  "jig/crates/jig-domain-core/src/documents/task_tree.rs",
  "jig/crates/jig-domain-core/src/runs/llm.rs",
  "jig/crates/jig-domain-core/src/settings/dispatch.rs",
  "jig/crates/jig-domain-core/src/tests/documents/offer.rs",
  "jig/crates/jig-protocol-engine-clients/src/translate/llm.rs",
  "jig/crates/jig-protocol-engine-clients/src/translate/owner.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/llm.rs",
]
effort = "high"
minutes = 150
+++

# The core fills the offer, and each template's preview

## Goal

`me`'s `models` becomes the offer: only what an account can run for the party now, each with those accounts, and each
template gets its `resolved_default`, so that no client guesses. A task given none that no account can run names no
model. An account changing changes `me`.

Acceptance, each a story in the new `jig-domain-core/src/tests/documents/offer.rs`:

1. Two providers, a key of one alone: the offer is that provider's models, in configuration's order, each with the
   account; the other gives none, though `me.providers` names it.
2. A key's model is offered with no host connected, and while a worker has yet to answer its logins. A program's
   model is offered once a connected worker reports its login usable, and leaves with that host or that login.
3. A held, limited or signed-out source's account isn't offered; with a second usable account of that provider, it is,
   on the second alone. Two usable accounts: both, in `candidates` order, the first what a run's selection takes.
4. A model the party's role doesn't allow isn't offered.
5. A key whose uses name project P is offered in P and not in another project (the playground's account).
6. The preview, with charter and catalogue orders opposed: the charter's first offered model at its default effort,
   `account` none; a template naming a choice previews it, offered or not; a chosen executor has no row; nothing
   offered, no row.
7. With a host able to run it, a task made with no choice takes at its check what the preview said (`next_llm`'s main).
8. A task's document, given none: with no account serving any charter model, `llm` is none and `run_wait` is still
   `Account { None, None, Empty }`; with a serving account held, it names that account's model; given a model with
   no account, it names that model.
9. Marks: an account's change, a program's login report, a host lost and a charter's change each mark the party's
   `me`. Follow `tests/reads/own_watches.rs`, `accounts_marked`, with its negatives.

## Inputs

- `temper/docs/plan/model-catalogue-1/notes/common.md`, `temper/docs/plan/model-catalogue-1/notes/contracts.md`
  (*The core*).
- `jig/docs/design/domain/engine.md`, 11.1; `jig/docs/design/domain/accounts.md`, 4.3;
  `jig/docs/design/client/domain/contract.md`, 5.1 and 7.2.

## Owns and may change

- Owns its `files`, with `documents/catalogue.rs`'s new functions beside `model_readiness`, which stays as it is:
  `next_llm` and the accounts document still read it.
- **Readers, and how:**
  - `jig/crates/jig-domain-core/src/tests/documents/frame_facts.rs`, `me_shows_the_models_a_partys_role_may_use_in_each_project`:
    give the fixture a usable key, or rename it to the offer's rule.
  - `jig/crates/jig-protocol-engine-clients/src/tests/llm.rs`, `me_lists_only_the_models_the_party_s_authority_allows_by_name`,
    and `src/tests/fixtures/documents.rs`, `jig/crates/jig-domain-core/src/tests.rs`: `FrameProjectRole`'s new fields.
  - `jig/crates/jig-domain-core/src/tests/classify_subjects.rs`: the `own` mask of `Touch::Accounts`.
  - `jig/crates/jig-domain-core/src/tests/inbox/inbox_selection.rs` (exact `document_bound`, `TEMPLATE_ENVELOPES`) and
    `tests/documents/frame_deployment_role.rs`: the bound's new terms.
  - `jig/tests/core/tests/suite/first_start.rs`: its world has no accounts, so read the role's models from
    `creating.within.authority.models` where it read `models`.
  - `jig/testing/jig-client-kit/src/joint/session/tests/llm.rs`: an edit only if the session's hello bounds drop a
    model it picks; a key's model needs no host, so expect none.
  - temper's worlds, fixtures only, each one line: a fixture listing models with no account gains one. `temper/tests/client`'s
    `restored_over` seeds a deployment key and needs no edit, so expect few.
- Options, pre-approved:
  - Where `document_bound` has no account count, put the offer's heap in `document_with_context_bound`.
  - `Subject::touched_by` (`documents.rs`): `Frame` answers `Touch::Accounts(None)` and `Touch::Accounts(Some(own))`. Every
    account event then delivers `me` whole; views coalesce, so accept it. `settings/dispatch.rs`: a change of charters
    or rules also touches frames, since a charter's order moves a preview.
  - If `project_policy::commit` moves a role's models without touching frames, add the touch and say so.
- Removes from `jig/docs/design/client/domain/contract.status.md` the 5.1 entries this closes, from
  `jig/docs/design/domain/engine.status.md` 11.1, and from `jig/docs/design/domain/testing.status.md` 4 the stories this
  writes. `jig/docs/design/STATUS.md`'s mention of `resolved_default` too.
- **A change of meaning runs its readers' tests.** Beyond the implementer's text, run
  `heavy cargo nextest run -p temper-client-world -p temper-client-tools -p temper-client-domain -p temper-client-view`.

## Steps

1. `catalogue.rs`: the walk the contracts note gives, returning the accounts that select `Chosen`. `Core::offer`,
   `Offered`. Coverage is `account_coverage(payer, Some(project))`, not the `None` `readiness` passes.
2. `runs/llm.rs`: `preview_llm`, sharing what it can with `next_llm` (`default_effort`, the candidates). A
   `document_llm(row)`: `next_llm` but `None` where a task given none found no kept, usable or serving candidate;
   `next_llm` keeps its fallback, as `chosen_main` and `chosen_policy` need one. `task_tree.rs` calls it.
3. `small.rs`: fill `offer` and `defaults`. `translate/llm.rs`, `models`: from the offer, by index, with `accounts`;
   `owner.rs`: `defaults`, through `translate::llm::choice`.
4. The marks, the bound. The stories, each broken once to see it bite.
5. Narrow tests, the readers' run above, then the implementer's text's closing checks. Commit and push.

## Report

As the implementer's text says; which temper fixtures needed an account, and whether any world found the offer empty
where it expected a model.

## Rebase (2026-11-06, step merge, foreman model-catalogue-1/offer-engine/foreman-1)

resolve translator conflicts by combining core offers and previews with the landed wire metadata
The merge conflicts with integration commit `42bd9ec419` (“Carry model offer facts and template previews across client boundaries”), merged by `38381ed5d1`.
Resolve both paths without taking either side wholesale:

- `jig/crates/jig-protocol-engine-clients/src/translate/llm.rs`: retain this branch’s `models(offer, models, limits)` signature, offer iteration, configuration-index lookup, and `accounts: offered.accounts.clone()`.
- Add the incoming `MeModelParts` fields: `effort: model.effort.clone()`, `shown: model.shown.clone()`, and `picker: model.picker`. Keep exactly one `accounts` field; discard the incoming empty placeholder.
- Preserve the incoming `fits` validation for shown names and default efforts.
- `jig/crates/jig-protocol-engine-clients/src/translate/owner.rs`, `project`: retain this branch’s translation of `value.defaults` through `llm::choice` into `MeDefault`, its populated `defaults` field, and `models(value.offer.as_slice(), models, limits)`. Discard the incoming empty defaults placeholder and authority-mask model call.

These combined values implement `notes/contracts.md` and `jig/docs/design/client/domain/contract.md`, 5.1.
Validate through heavy with `TEMPER_BUNDLE=none`: run `jig-protocol-engine-clients`’ lib tests filtered to `tests::llm::`, then the spec’s required temper reader tests.
