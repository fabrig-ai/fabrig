+++
id = "offer-narrows"
part = "reads"
kind = "feature"
needs = ["list-surface", "account-reads"]
files = ["jig/crates/jig-domain-core/src/documents/catalogue.rs", "jig/crates/jig-domain-core/src/runs/llm.rs", "jig/crates/jig-domain-core/src/runs/run_choice.rs", "jig/crates/jig-domain-core/src/tests/documents/offer.rs", "jig/crates/jig-domain-core/src/tests/runs/first_usable.rs", "jig/docs/design/domain/accounts.status.md", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 90
+++

# An account rules a model out

## Goal

`jig/docs/design/domain/accounts.md`, 3.7's *What it decides*, where it's read: the offer, a template's preview, the readiness the document shows, and a task given no LLM choice. So every page draws the same list.

Acceptance:

- `Core::account_allows(account, model)`, finding the model's number and kind in the catalogue by its name, reads `accounts.allows`. A model with no catalogue row, a task's held one, is allowed.
- Rows of `tests/documents/offer.rs`'s table, on one provider's two accounts: an account with no list offers the stock and a person's, not a list's; one with a good read offers what `names` holds and a person's, not a stock name it lacks, and not a list's it lacks; one whose only attempts failed offers the stock and a person's; one with a good read and a later failure keeps its `names`; two accounts offer the union, each with its accounts; a provider whose every account rules a model out doesn't offer it; `Model.picker` off still appears, in the full offer.
- Rows of `tests/runs/first_usable.rs`: a task given no choice passes over a model no usable account allows, going on to the next, so a plan without Astra falls on to Sol, the draft's section 3; the preview agrees with its task (`jig/docs/design/client/domain/contract.md`, 5.1); a task's explicit choice runs on though its account's list has lost the name; a kept account stays, by `accounts.md`, 4.3's first rule.
- `run_choice`'s candidates take the filter only when the task has no LLM choice and no kept account.
- `engine.status.md`'s Not yet line, and `accounts.status.md`'s, go, once `core-reads`' report and this one's have no gap left; if `core-reads` hasn't yet landed, leave what is its and say so.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 6.
- `jig/docs/design/domain/accounts.md`, 3.7 and 4.3; `jig/docs/design/domain/engine.md`, 11.1; `temper/docs/design/draft/model-catalogue.md`, section 3; `temper/docs/design/draft/model-catalogue.status.md`, *Design departs*.

## Owns and may change

Owns the header's files. `documents/catalogue.rs:197,206,266` (`offered_accounts`, `model_readiness`) and `runs/llm.rs:125-150`'s `candidates` take the filter; `runs/run_choice.rs:59` takes it under the condition above. `accounts.select` and `accounts.candidates` keep their signatures: the filter is the core's.

This changes what a shared item means (the offer, and a task's first choice): run its readers' tests below.

Options, each pre-approved:

- Tests set a record with whatever `Stored::Account` restore path `tests/fixtures.rs` already has. If none, restore the child with a `list`: a helper there.
- `ModelKind` maps from `ModelSource` in one `match`, in `documents/catalogue.rs`.

## Steps

```rust
// documents/catalogue.rs
pub(crate) fn account_allows(&self, account: u32, model: &crate::Model) -> bool { .. }
```

Then `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-core -p jig-core-world -p jig-accounts-world`; `heavy cargo nextest run -p temper-engine-domain -p temper-engine-domain-world -p temper-service -E 'test(/first_model|configuring|models|offer/)'`; `heavy cargo nextest run -p temper-client-world -E 'test(/model|accounts|choice/)'`. Commit and push.

## Report

As the implementer's text says.
