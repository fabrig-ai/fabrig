+++
id = "first-usable-choice"
part = "first-usable-choice"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/runs/llm.rs",
  "jig/crates/jig-domain-core/src/runs/run_choice.rs",
  "jig/crates/jig-domain-core/src/documents/catalogue.rs",
  "jig/crates/jig-domain-core/src/documents/task_tree.rs",
  "jig/crates/jig-domain-core/src/tests/runs/first_usable.rs",
  "jig/crates/jig-domain-core/src/tests/runs/run_choice.rs",
  "jig/crates/jig-domain-core/src/tests/runs/default_llm.rs",
  "jig/crates/jig-domain-tasks/src/llm.rs",
  "jig/crates/jig-domain-tasks/src/control.rs",
  "jig/crates/jig-domain-tasks/src/settings_moves.rs",
  "jig/crates/jig-domain-tasks/src/domain.rs",
  "jig/testing/jig-core-kit/src/oracle.rs",
]
effort = "high"
minutes = 150
+++

# A task given no LLM takes the first model it can use, at its run's check

## Goal

On a box whose only account is a Claude Code login, every task waits for ever, for OpenAI's model.
`Core::default_llm` picks a model at Create with empty host facts, so a program source never counts
(`Domain::select`, `host_allows`), the default falls to the charter's first and is recorded for good.

Make the choice at each run's check instead, from current facts, and keep what a task is using. A
named choice is unchanged. Also make an unsigned login wait as its account's, not as a host's.

Acceptance, each with a story in `jig-domain-core/src/tests/runs/first_usable.rs`, on charter models in
the order `temper init` writes them: OpenAI's two, then a program-endpoint Anthropic model, then a third
provider's:

1. Only a usable Claude login: `run_choice` is `Ok`, `policy.model` is the Anthropic model, its
   alternatives the others that bind. The stored row's `llm` is `None`.
2. Only an OpenAI key: main is the charter's first.
3. Claude's login claimed, so the task keeps that account, then an OpenAI key added: its next check still
   takes the Anthropic model. **A task keeps what it's using.**
4. No account, the wait `Account { None, None, Empty }`; a Claude login added: the same task runs on it.
5. A task naming OpenAI's model, only Claude usable: it waits, `Account { None, None, Empty }`, and stays
   named, whatever is added for another provider.
6. A kept account removed, or no longer serving the payer: the next check chooses again.
7. A kept account limited, another provider's usable: it waits for its own.
8. A declared login not signed in on the one host: `Err(RunWait::Account { account: Some(_), source:
   Some(_), why: WaitWhy::Host })`, and its document's `run_wait` says so. No host at all, or the host
   holding the usable login full: `Err(RunWait::Host)`.
9. A Task document's `llm` is what `next_llm` gives, for an unnamed and a named task alike.
10. Refused, naming the charter's first with `LlmFault::Authority`: a Create whose authority allows none of
    the charter's models, as today; an amendment leaving an unnamed task such an authority; and a move
    to a revision whose charter gives such a task none (`settings_moves`, and the incompatible count in
    `domain.rs`).

## Inputs

- `temper/docs/plan/first-usable-model/notes/common.md`, `temper/docs/plan/first-usable-model/notes/contracts.md`.
- `jig/docs/design/domain/accounts.md`, 4.3 and 7.
- `jig/docs/design/domain/tasks.md`, section 3, *Its LLM choice*.
- `jig/docs/design/domain/engine.md`, 7.1, step 1, and 11.1, *A resolved default beside intent*.
- `jig/docs/design/client/domain/contract.md`, 5.1, *a task*.

## Owns and may change

Owns, as `files` lists them, and removes the **Not yet** entries this closes: `accounts.status.md`'s 4.3,
`tasks.status.md`'s 3, `engine.status.md`'s 7.1 step 1, and `contract.status.md`'s 5.1 (each in
`jig/docs/design/domain/` or `jig/docs/design/client/domain/`).

May also change, expected results and fixtures only:

- `jig/crates/jig-domain-core/src/tests/runs.rs`, its `mod` line.
- `jig/tests/core/tests/suite/charters.rs`, `a_task_created_without_an_llm_choice_records_its_default_and_its_run_uses_it`:
  rename it for what it now says; its assignment half stands.
- `jig/tests/core/tests/suite/settings.rs`, `a_held_versions_documents_show_its_charter_and_nothing_once_released`, about
  lines 1960 to 2043, which compares the stored row's `llm` with the document's.
- `jig/tests/engine-clients/tests/requests/llm.rs`, `an_amend_naming_a_choice_changes_the_one_the_task_s_next_run_takes`.
- `temper/tests/engine/domain/src/oracle.rs`, for the same `durable` edit as `jig-core-kit`'s.
- Comments that say the default is recorded: `jig-domain-tasks/src/llm.rs`, about lines 190 and 200;
  `jig-domain-core/src/tests/runs/default_llm.rs`'s module doc.

Options, each pre-approved:

- **An ended task's document:** `next_llm` reads the charter by `charter_at` and may find none once its
  revision is released, and `payer` may give none: then `None`, or the charter's order with no account's
  view. It never panics on an ended row; `chosen_policy`'s `expect`s are for live tasks.
- **The restart oracle:** `jig-core-kit/src/oracle.rs`, `durable`, strips `run_wait` because it derives
  from facts a restart hasn't restored; `llm` of an unnamed task now does too. Default: strip `llm` there
  with the same comment, and, if the oracle compares no stored `llm` elsewhere, compare the stored row's
  field directly so a named choice is still held across a restart.
- **`default_llm`'s revision:** `tasks::Domain::default_llm` reads the newest charter. Add `default_llm_at(charter,
  revision, models)` beside it for a task's own revision (`row.settings.run.unwrap_or(row.settings.task)`), and
  keep the old name for Create and `jig/tests/tasks`.
- **`resolve_llm`:** it stays, for its refusal and for recurring templates' members. It stops assigning
  `new.llm`.

Not here: `resolved_default` (still **Not yet**), and any change to the Task document's or the store's format.

## Steps

1. Write the stories first and see them fail. `run_choice.rs`' helpers serve: `ready`, `placement_fixture`,
   `placement_with_accounts`, `host_with_logins`, `host_with_endpoints`, `login`, `inventory`, `account`,
   `granted`, `new_row`, `fixtures::make_batch`.
2. `documents/catalogue.rs`: draw the host join out of `readiness`, per model, taking its coverage,
   so the run's check and the accounts document read one rule:

   ```rust
   impl Core {
       /// Whether `model` has an account usable for `payer`, on a connected host (accounts.md, 4.3).
       pub(crate) fn model_readiness(&self, payer: tasks::Payer, coverage: &[accounts::Coverage], model: &Model) -> (Readiness, bool)
   }
   ```

   `readiness` keeps its results: its tests stand unchanged.
3. `runs/llm.rs`:

   ```rust
   impl Core {
       /// What a task's next run takes (domain/accounts.md, 4.3): the choice it was given, or, given
       /// none, its charter's models within its authority, the first it can use its main.
       pub(crate) fn next_llm(&self, row: &tasks::TaskRecord) -> Option<tasks::LlmChoice>
   }
   ```

   Step 1 of 4.3 reads `row.kept_account`, the account's provider and `account_coverage(payer,
   Some(row.project))`; step 2 `model_readiness == Usable`; step 3 a non-empty `accounts.candidates`; step
   4 the first. Efforts by `default_effort`, as Create does today. `chosen_policy` and `chosen_main` read it,
   each keeping its fallback to the charter's own. `task_tree.rs`, about line 955: `llm: core.next_llm(row)`.
4. `run_choice.rs`: remember the first `Selection::Wait { why: WaitWhy::Host, .. }` as `RunWait::Account {
   account: Some(account), source, why }`, keep trying hosts, and at the end, in place of `host_wait`'s plain
   `RunWait::Host`, return that wait unless `model_readiness` reads `Usable`, a busy host holding the
   login. Update the doc comment's probe count.
5. `jig-domain-tasks`: `control.rs`, `check_llm`, and `settings_moves.rs` and `domain.rs`'s `row.llm`
   `None` arm: an unnamed task needs its authority to allow one of the charter's models.

**This changes what a shared item means** (`llm`, `RunWait`), so also run the readers' tests:
`-p jig-domain-core`, `-p jig-domain-tasks`, `-p jig-core-world`, `-p jig-tasks-world`, `-p jig-engine-clients-world`,
`-p jig-client-protocol-world`, `-p jig-conformance-world`; then `heavy cargo check -p temper-service --tests`, and
`-p temper-service --lib`, `-p temper-engine-domain-world`, `-p temper-conformance-world`. A fake-box story that fails only because a chat's model now differs is a
report, under `## Left`, for `stock-one-account`.

## Report

As the implementer's text says; the options you took; whether any temper test's expected model moved.
