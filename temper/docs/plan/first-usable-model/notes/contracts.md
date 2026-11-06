# first-usable-model: the shapes that cross tasks

The run's binding calls. Each side builds to these, and neither needs the other's task.

## The engine (`first-usable-choice`)

- **A task given no choice stores none.** `tasks::TaskRecord.llm` is `None` for it; `Some` means named,
  by its creator, its template or an amendment. No stored or wire format changes.
- `Core::next_llm(&self, row: &tasks::TaskRecord) -> Option<tasks::LlmChoice>`, `pub(crate)`, in
  `jig-domain-core/src/runs/llm.rs`: what the next run takes, by `jig/docs/design/domain/accounts.md`, 4.3.
- A Task document's `llm` is `next_llm`. Its `kept_account` and `run_wait` keep their meaning.
- `RunWait::Account { account: Some(_), source, why: WaitWhy::Host }`, so
  `TaskRunWait::Account { why: TaskWaitWhy::Host, .. }`, is now reached: a program login no host holds
  usable. `TaskRunWait::Host` stays: no host with room, or a busy one holds the login.
- `account: None` with `why: Empty`: no account serves the main's provider. The document's `llm.main`
  names that model.

## The status (`run-wait-status`)

In `jig-client/src/facts.rs`:

```rust
/// What a run that can't start waits for (client/domain/client.md, 19.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunWaitKind { Account, Credential, Host }

pub enum ChatStatus { /* ... */ RunWaiting(RunWaitKind), /* ... */ }
```

`jig_client::words::Word::status` gives `StatusWord::{WaitingForAccount, WaitingForCredential,
WaitingForHost}`, English "Waiting for an account", "Waiting for a credential", "Waiting for a host".

## The printed command (`printed-command`)

- `temper-client-protocol-wire`: `versioned record DeploymentAnswer { welcome: bool owned: bool
  program_login: option bytes 8192 }`.
- `temper-client-domain`'s `forge::deployment::Answer` gains `program_login: Option<Box<[u8]>>`, loses
  `Copy`; `root.forge().deployment()` gives `Option<&Answer>`.
- `temper-client-view`'s `pages/connect_model.rs`:

  ```rust
  /// The whole command for NAME (client/README.md, 4.6, *The printed command*).
  pub(crate) fn printed_command(root: &Domain, w: &Words, name: &[u8]) -> Box<[u8]>
  /// The same with the words before it, "On this machine, run:" or "On each host, with its directories:".
  pub(crate) fn command_line(root: &Domain, w: &Words, name: &[u8]) -> Box<[u8]>
  pub(crate) fn command(b: &mut Builder, root: &Domain, w: &Words, name: &[u8], welcome: bool)
  ```
