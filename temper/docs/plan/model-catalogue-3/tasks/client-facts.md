+++
id = "client-facts"
part = "page"
kind = "interface"
needs = ["list-surface", "account-reads"]
files = ["jig/crates/jig-client-domain/src/contract/llm.rs", "jig/crates/jig-client-domain/src/contract/accounts.rs", "jig/crates/jig-client-domain/src/contract/asks.rs", "jig/crates/jig-client-domain/src/requests.rs", "jig/crates/jig-client-protocol-wire/schema/**", "jig/crates/jig-client-protocol-wire/src/generated/**", "jig/crates/jig-client-protocol-wire/golden/**", "jig/crates/jig-client-protocol-wire/src/tests/**", "jig/crates/jig-client-protocol-link/src/translate/**", "jig/crates/jig-client-protocol-link/src/tests/**", "jig/crates/jig-protocol-engine-clients/src/**", "jig/crates/jig-domain-core/src/documents/accounts.rs", "jig/crates/jig-domain-core/src/documents/deployment.rs", "jig/crates/jig-domain-core/src/documents/small.rs", "jig/crates/jig-domain-core/src/routing/party/account_requests.rs", "jig/testing/jig-client-scripted-engine/src/**", "jig/testing/jig-client-test-records/src/**", "temper/docs/design/client/journeys/*/recording/**", "jig/docs/design/client/domain/contract.status.md"]
effort = "high"
minutes = 140
+++

# `me`, the accounts document and the act carry it

## Goal

What a client needs to draw a list's state, and the act that reads one again, on the client's wire and in jig's client contract. It draws nothing: `models-read` does.

Acceptance:

- `contracts.md`, 8's `MeModel.new_until`, `AccountList`, `AccountListFailure`, `AccountLine.list` and `Ask::ReadLists` exist, as `jig/docs/design/client/domain/contract.md`, 3.2 and 5.1 say, and cross the wire, each with goldens at smallest and full, and through the scripted engine's independent encoding.
- A model's `new_until` is `first_read + NEW_FOR`, whatever the clock: a row of `jig-protocol-engine-clients/src/tests/catalogue.rs`, a list row and a stock one.
- A party's `accounts`, and an owner's deployment document, carry `list` for an account `has_list` says has one, whole or with its fields none, and none for one that doesn't, rows of `jig-domain-core/src/tests/documents/accounts_document/`.
- `me`'s `left_out` is the charter's count plus the largest `left_out` of the accounts whose rows that party may see, a row of its table.
- `Ask::ReadLists` is a keyed request, by a party on their accounts, answered as the child's `Reading { count }`, as `AccountAnswer::Reading`; its retry returns the same answer; it isn't among a compound request's operations; `jig-protocol-engine-clients/src/tests/model_offers.rs:138`'s row holds its digest, link and wire. Reads aren't kicked twice by a keyed retry.
- Journeys: `temper-client-journeys again` for `welcome-playground`, `welcome-set-up`, `project-landing` and `chat-draft`, since the wire's `me` changes. Its prints and the gallery change no words here. Recording flakes as `common.md` says.
- `contract.status.md`'s line goes.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 6, 7 and 8.
- `jig/docs/design/client/domain/contract.md`, 3.1, 3.2, 3.4, 5.1 and 7.1; `jig/docs/design/domain/accounts.md`, 3.7.
- `git show 987ff08f1a --stat`: `ChangeModel`'s footprint, the worked example.

## Owns and may change

Owns the header's files. The survey's checklist:

- **The fact path:** `jig-client-domain/src/contract/llm.rs:26`, `accounts.rs:29,43`; `schema/client-v19.schema:655`, `client-v19-part1.schema:17,20`; `jig-protocol-engine-clients/src/boundary.rs:218`, `translate/llm.rs:15,215`, `translate/accounts.rs:163,184`, `translate/owner.rs:141`; `jig-client-protocol-link/src/translate/llm.rs:54,144`, `translate/accounts.rs:142,235`; `documents/accounts.rs:16,28,58,100`, `documents/deployment.rs:323,332`, `documents/small.rs:123,132,153`; `jig-client-test-records/src/owner.rs:6`, `jig-client-scripted-engine/src/codec/values/{llm.rs:150,193,me.rs:59,101}`.
- **The act:** `contract/asks.rs:88`; `requests.rs:255`; `client-v19-part1.schema:107`; `jig-client-protocol-link/src/translate/{asks.rs:567,store.rs:71,compound.rs:45,103}`; `jig-protocol-engine-clients/src/{request.rs:141,331,386,translate/asks.rs:53}`, `translate/llm.rs:324`; `routing/party/account_requests.rs:27,116`, an arm in `account-reads` that here answers on the wire; `jig-client-scripted-engine/src/requests.rs:357,420,513,655`, `requests/projects.rs:18`.
- **Exhaustive matches of `Ask`, by the required-field exception:** `jig-client-view/src/state/activity/sending.rs:52`, `state/page/project/owner_form.rs:52,244,298,358`, `tree/sections/project/owner.rs:82,554,646,683,720,1158`; `jig/testing/jig-client-test-kit/src/check.rs:221`, `jig-client-kit/src/{observe.rs:112,harness/referee.rs:549,harness/referee/offers.rs:163,joint/component.rs:248}`, `jig-client-protocol-kit/src/{front/observe.rs:132,engine.rs:264,422}`, `jig-test-service/src/clients/translation.rs:143,206`; `temper-client-tools/src/jig/render/outcome.rs:63,1019`; `temper-service/src/clients/translation.rs:22,232`; `temper-engine-domain/src/{boundary.rs:629,admission.rs:37,route.rs:172}`.
- `temper/tools/journeys`'s tests, to read them as the regenerated recordings require.

Options, each pre-approved:

- `WallTime` is the form `made` takes on the wire in a task's document. If the contract's type for it is named otherwise, take that word.
- `new_until` for a model with no `first_read` is none.
- A compound request refuses `ReadLists`, as it does `ChangeModel` at `compound.rs:45`.
- A `Cut` on the accounts document that drops a row drops its `list`: nothing else is needed.

## Steps

Then `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and temper's; `heavy cargo clippy` naming every package you changed, `--all-targets -- -D warnings`; `heavy cargo nextest run -p jig-client-domain -p jig-client-protocol-wire -p jig-client-protocol-link -p jig-protocol-engine-clients -p jig-client-scripted-engine`; `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/document/)'`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-tools`; `heavy cargo nextest run -p temper-client-journeys`; `heavy cargo check --workspace --tests`. Commit and push.

## Report

As the implementer's text says.
