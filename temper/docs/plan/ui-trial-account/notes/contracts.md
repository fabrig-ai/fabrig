# ui-trial-account: the shapes that cross tasks

Binding for `account-choice`, which builds them, and `trial-account`, which uses them. The design is `jig/docs/design/domain/accounts.md`, 4.3, `jig/docs/design/domain/authority.md`, 6.1, and `temper/docs/design/domain/trial.md`, 2 and 4.

## 1. A person's task carries an LLM choice

`jig-domain-core`'s `vocabulary.rs`, the field after `executor`:

```rust
/// A person's task in no project (jig/docs/design/domain/authority.md, 6.1).
PersonTask {
    seed: Box<crate::PersonSeed>,
    executor: tasks::Executor,
    /// None takes what a chat of theirs takes; `Core::account_llm` holds it to one account.
    llm: Option<tasks::LlmChoice>,
    spec: tasks::Spec,
    contract: tasks::Contract,
},
```

`routing/task.rs`'s `route_person_task` puts it in its `tasks::New`'s `llm`, where `None` stands today. The batch's checks (`resolve_llm`, tasks' own) stay as they are.

## 2. A seed's models

`effects/connector.rs`:

```rust
impl PersonSeed {
    /// The models its authority allows (jig/docs/design/domain/authority.md, 6.1).
    #[must_use]
    pub const fn models(&self) -> tasks::Models;
}
```

`const` may go if the compiler refuses it.

## 3. The choice that holds a new task to one account

`jig-domain-core`, a new module `runs/account_llm.rs`:

```rust
impl Core {
    /// The choice that holds a new task of `charter` to `account`
    /// (jig/docs/design/domain/accounts.md, 4.3): the charter's first model, in its order
    /// within `models`, that `offer(payer, project, models)` holds with `account`; else the
    /// first whose LLM provider is the account's and whose candidates for `payer` hold it.
    /// Each model at its default effort, as a task given no choice takes it, and
    /// `account: Some(account)`. None where the account serves none of them.
    #[must_use]
    pub fn account_llm(
        &self,
        charter: u32,
        models: tasks::Models,
        payer: tasks::Payer,
        project: u32,
        account: u32,
    ) -> Option<tasks::LlmChoice>;
}
```

Its sub-agents are the charter's other models within `models`, each at its default effort, as `preview_llm` builds them.

## 4. The trial's choice (temper)

`temper-engine-domain`'s `route/trials.rs`, at `trials::Request::Make { account, person }`, after `Core::person_seed`:

- `Core::offer(tasks::Payer::Party(person), people::DEPLOYMENT, seed.models())` holds no account but `account`: `llm: None`, the first usable account's trial, as today.
- It holds any other: `Core::account_llm(TRIAL_CHARTER, seed.models(), tasks::Payer::Party(person), people::DEPLOYMENT, account)`; `Some` is the `llm`; `None` routes `trials::Event::Refused { account }` and makes no task.

Each `Make` judges it again, an `again: true` included.
