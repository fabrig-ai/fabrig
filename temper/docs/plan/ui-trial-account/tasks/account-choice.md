+++
id = "account-choice"
part = "choice"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-core/src/runs/account_llm.rs", "jig/crates/jig-domain-core/src/runs.rs", "jig/crates/jig-domain-core/src/runs/llm.rs", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/routing/mod.rs", "jig/crates/jig-domain-core/src/routing/task.rs", "jig/crates/jig-domain-core/src/effects/connector.rs", "jig/crates/jig-domain-core/src/tests/person_task.rs", "jig/tests/core/tests/suite/person_task.rs", "temper/crates/temper-engine-domain/src/route/trials.rs", "jig/docs/design/domain/accounts.status.md", "jig/docs/design/domain/authority.status.md"]
effort = "high"
minutes = 100
+++

# A person's task may be held to one account

## Goal

jig gives an application the choice that holds a new task to one account, `Core::account_llm`, and lets a person's task in no project carry it: `Event::PersonTask` gains `llm`, and `PersonSeed` says its `models()`. A task made with it runs on that account alone, in either order of two providers and for a second account of one. Nothing in temper uses it yet: its one constructor passes `llm: None`.

Acceptance:

- Sections 1 to 3 of `temper/docs/plan/ui-trial-account/notes/contracts.md`, exactly.
- `jig-domain-core`'s `tests/person_task.rs`, one table test, `a_task_held_to_an_account_runs_on_it_alone`, its rows (accounts as `(number, provider)`, the account held, the model it names):
  - `[(1, 1), (2, 2)]`, held 2, `second`: a later account of the other provider (Claude, then ChatGPT);
  - `[(1, 2), (2, 1)]`, held 2, `scripted`: the other order;
  - `[(1, 2), (2, 2)]`, held 2, `second`: a second account of one provider, where a task given no choice takes 1.

  For each: `account_llm(1, seed.models(), Payer::Party(PERSON), PROJECT, held)` names `account: Some(held)` and that model; a person's task made with it has `run_choice` choose that model and `held` first in its accounts; a claim keeps `held` (`task_document(..).kept_account`).
- In that test or one beside it, a table of what holds nothing or waits:
  - a key whose `uses` name project 1 alone (as `a_project_only_key_leaves_the_charters_first_model_waiting_without_an_account` adds it): `account_llm` is `None`;
  - an account among its provider's candidates but in no offer (its source not usable: see the options): the choice names its provider's first charter model and `run_choice` waits naming that account, `Err(RunWait::Account { account: Some(held), .. })`.
- Each existing test of `tests/person_task.rs` passes with `llm: None`, `a_persons_plan_and_key_choose_the_first_usable_model_and_keep_its_account` among them, unchanged but for the literal.
- The focused tests of `jig-domain-core` and `jig-core-world` (`jig/tests/core`) pass; `heavy cargo check --tests` for `temper-engine-domain`, `temper-service`, `jig-test-service` and `jig-client-protocol-kit`.
- `jig/docs/design/domain/accounts.status.md`: its **Not yet** entry on `Core::account_llm` (4.3) goes. `jig/docs/design/domain/authority.status.md`: its **Not yet** entry on 6.1 goes.

## Inputs

- `jig/docs/design/domain/accounts.md`, 4.3, the bullets *A task given no LLM choice*, *The task keeps it* and *Holding a new task to one account*.
- `jig/docs/design/domain/authority.md`, 6.1.
- `jig/docs/design/domain/engine.md`, 11.1, *The offer*.
- `temper/docs/plan/ui-trial-account/notes/contracts.md`, 1 to 3.
- `temper/docs/plan/ui-trial-account/notes/common.md`.

## Owns and may change

Owns the header's files. In them:

- `runs/account_llm.rs` is new, with its `mod` line in `runs.rs`.
- `runs/llm.rs`: only `pub(super)` on the private helpers the new module reuses (`candidates`, `defaults`, `charter_model`), no other line. `model-catalogue-3` changes this file: keep off its lines.
- `vocabulary.rs`: the one field and its doc line.
- `routing/mod.rs`: the `Event::PersonTask` arm's destructuring and call alone. `routing/task.rs`: `route_person_task`'s parameter and its `llm` in `tasks::New`.
- `effects/connector.rs`: `PersonSeed::models` alone.
- `temper/crates/temper-engine-domain/src/route/trials.rs`: `llm: None,` in its `core::Event::PersonTask` literal, nothing else (the next task decides it).
- `jig/tests/core/tests/suite/person_task.rs`: `llm: None,` in its literal.
- Any other literal the compiler names gains `llm: None`; arms written `{ .. }` need nothing.

Options, each pre-approved:

- **Reuse, don't copy.** `account_llm` follows `preview_llm` in `runs/llm.rs`: `charter_at` at `self.tasks.newest()`, `self.tasks.default_llm_at(charter, revision, models)`, its `candidates`, then `defaults` for the efforts and sub-agents. The fallback ("whose candidates hold it") follows `task_llm`'s `serving`: `self.accounts.candidates(coverage::payer(payer), provider, model.program.as_deref(), None, None, coverage)` holding `account`, the account's own provider being the model's.
- **The fixture.** `person_account` adds account 1 of provider 2 alone: give it the account's number and provider, keeping its existing callers' meaning. The fixture's limits hold two accounts; raise them in the fixture only if a row needs more, and say so.
- **A source not usable.** If not granting a key source's credential (`granted`) leaves it out of the offer but among its provider's candidates, that is the waiting row. If the fixture can't make one so without new helpers beyond this file, drop that row and say so in the report.
- **Efforts.** The fixture's models declare no efforts. If giving `second` the efforts `low` and `high`, its charter model naming `high`, takes no other change, add a row that the choice names effort `Some(1)`; otherwise leave it and say so.
- `event(seed)` in the test file becomes `event(seed, llm)`; its callers pass `None`.

## Steps

```rust
// runs/account_llm.rs
//! The choice that holds a new task to one account (jig/docs/design/domain/accounts.md, 4.3).
impl Core {
    #[must_use]
    pub fn account_llm(&self, charter: u32, models: tasks::Models, payer: tasks::Payer, project: u32, account: u32)
        -> Option<tasks::LlmChoice>;
}
```

```rust
// tests/person_task.rs
#[test]
fn a_task_held_to_an_account_runs_on_it_alone() {
    // Accounts as (number, provider), the account held, and the model its choice names.
    for (held_accounts, held, model) in [
        (&[(1, 1), (2, 2)][..], 2, b"second".as_slice()), // Claude, then ChatGPT
        (&[(1, 2), (2, 1)][..], 2, b"scripted".as_slice()), // ChatGPT, then Claude
        (&[(1, 2), (2, 2)][..], 2, b"second".as_slice()), // a second account of one provider
    ] {
        let (mut core, env) = fixture();
        // add each account, then the host once
        let seed = core.person_seed(PERSON).expect("room");
        let llm = core.account_llm(1, seed.models(), tasks::Payer::Party(PERSON), PROJECT, held).expect("it serves a model");
        // the choice's account and model; make the task with it; run_choice; claim; kept_account
    }
}
```

Each new test breaks its rule once by hand: take the offered branch out (the second-account row then takes account 1), and drop `account: Some(account)` from the choice. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `temper/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-core -p jig-core-world -p temper-engine-domain --all-targets -- -D warnings`; the focused tests of `jig-domain-core` and `jig-core-world`, then the `heavy cargo check --tests` the acceptance names, one command at a time. Commit and push.

## Report

As the implementer's text says, with the wall time of the new tests, and which optional rows you wrote or dropped.
