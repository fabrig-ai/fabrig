+++
id = "temper-accounts-host"
part = "service-accounts"
kind = "migration"
needs = ["jig-accounts-host"]
files = ["temper/crates/temper-service/src/accounts.rs", "temper/crates/temper-service/src/settings.rs", "temper/crates/temper-service/src/engine.rs", "temper/crates/temper-service/src/engine/diagnostics.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/clients.rs", "temper/crates/temper-service/src/clients/google.rs", "temper/crates/temper-service/src/setup.rs", "temper/crates/temper-service/src/tests/accounts_host.rs", "temper/crates/temper-service/src/tests/engine.rs", "temper/crates/temper-service/src/tests/fixtures.rs", "temper/crates/temper-service/Cargo.toml", "temper/crates/temper/src/accounts_runtime.rs", "temper/crates/temper/tests/support/owned.rs"]
effort = "high"
minutes = 55
+++

# temper: compose jig's accounts' host, and delete the copy

## Goal

`jig_protocol_accounts::host` ships the accounts' host, `skein_io::kernel::Routes` the kernel hand-off, and
`skein_lib::earliest` the deadline fold. `T/accounts.rs` (`notes/service-parts.md` names `T`) deletes its copies of all
three and keeps an adapter: temper's tokens, its root's event, its store's calls, its one-slot hold, and what an invalid
restored secret means to it.

Acceptance:

- `T/accounts.rs` holds what step 2 shows and nothing else: no `Kernel`, `earliest`, `unkept`, `copy_origins`,
  `copy_endpoint`, `provider_templates`, `registration_input`, `untag_store`, `impl Files` or `struct Accounts`.
- `temper_service::settings::Files` is `jig_protocol_config::Files`.
- Every `temper-service` test and every direct dependent's passes, with no test edited but as `Owns and may change`
  lists, and no assertion changed. No test moves.
- `tests::engine::a_cancellation_waiting_unpublished_keeps_its_completed_targets_route`,
  `a_stalled_link_consumer_drops_nothing_and_keeps_order` and `kernel_operations_are_routed_back_to_their_owner` stay
  where they are, green, under their names.
- **Size cap:** the branch's net lines are at most -290; about -320 is expected. Measure it, and stop and ask if it's
  passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`,
  `temper/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.1: the contract.
- `temper/docs/development/debt/D001-reference-service-copy.md`: read it. This task doesn't pay it; `service-items` does.
- `jig/crates/jig-protocol-accounts/src/host.rs` and `jig/testing/jig-test-service/src/accounts.rs`, the other adapter.

## Owns and may change

- Owns: `T/accounts.rs`.
- The readers, each edited only as this table says:

  | File | Edit |
  |---|---|
  | `T/settings.rs` | the `Files` struct goes, its doc comment with it; `pub use jig_protocol_config::Files;`. `settings::Files { .. }` literals in `temper` and its tests need no edit |
  | `T/engine.rs` | 7 `crate::accounts::earliest` become `skein_lib::earliest`; `accounts.work_pending(now)` becomes `(now, true)`; `accounts.iterate(..)` becomes `crate::accounts::iterate(accounts, ..)` |
  | `T/engine/diagnostics.rs` | `accounts.work_pending(now)` becomes `(now, true)` |
  | `T/system.rs` | the field and 2 `accounts::Kernel::new(` name `skein_io::kernel::Routes`; 2 `accounts::earliest` become `skein_lib::earliest`; `accounts.wait_state()` becomes `accounts::wait_state(accounts, engine.env.now)`; `Accounts::new`'s `seeds` becomes `&seeds.crypto, &seeds.installation` |
  | `T/clients.rs` | 2 `crate::accounts::earliest` become `skein_lib::earliest` |
  | `T/clients/google.rs` | 2, the same |
  | `T/setup.rs` | 3, the same |
  | `T/tests/engine.rs` | `crate::accounts::Kernel::new(4)` becomes `skein_io::kernel::Routes::new(4)`; 3 `.startup_state()` become `.component().startup(0)` |
  | `T/tests/fixtures.rs` | `accounts.iterate(..)` becomes `crate::accounts::iterate(accounts, ..)` |
  | `T/tests/accounts_host.rs` | below |
  | `temper/crates/temper/src/accounts_runtime.rs` | the constructor gains `trust: None` |
  | `temper/crates/temper/tests/support/owned.rs` | the constructor gains `trust: None` |
  | `temper/crates/temper-service/Cargo.toml` | `skein-oauth-accounts` moves to `[dev-dependencies]`: after this task only `T/tests/accounts_host.rs` names it. |

  `crate::accounts::{owns, tag, untag}` keep their names, so `T/forge.rs`, `T/route.rs`, `T/setup.rs`'s `untag`,
  `T/engine.rs`'s `owns` and `T/tests/engine.rs`'s `owns` need no edit. `crate::accounts::Accounts` and
  `crate::AccountsRuntime` keep their paths through the adapter's `pub use`, so `T/clients.rs`, `T/startup.rs`,
  `temper/crates/temper/src/lib.rs` and `T/tests/fixtures.rs`'s signatures need no edit. This corrects the survey: its
  `T/forge.rs` 1 is `untag`, not `earliest`, and `T/engine.rs` has 7, not 10.
- `T/tests/accounts_host.rs`, each edit a moved item's name or signature, no assertion changing. This departs from the
  common note's wording only in that a signature counts as a moved item here:
  - the 7 `accounts.iterate(..)` become `iterate(accounts, ..)`, or `iterate(&mut accounts, ..)`. Where rustfmt would
    wrap them, a local `fn pass(accounts, engine, events)` in the test file is yours to add;
  - 4 `work_pending(x)` become `work_pending(x, true)`;
  - `accounts.pending.is_some()` becomes `accounts.held()`, and `.is_none()` `!accounts.held()`;
  - `accounts.component.is_empty()` becomes `accounts.component().is_empty()`;
  - `AccountsRuntime { providers }` gains `trust: None`;
  - the 2 `Accounts::new` calls take `&seeds.crypto, &seeds.installation` of a local `fixtures::seeds()`;
  - the imports `use super::*` no longer brings: `domain`, `config`, `oauth`, `driver`, `List`, `Secret`.
- Not yours: any jig file. If the host can't do what this adapter needs, that is a `Question:`, whose answer is a jig
  task. `Store::secret_requests` and its hooks stay: the adapter calls `store.secrets`, so accounts' traffic is counted
  as it is today.
- `plan.toml`'s `wrong-secret-kind` question is the user's. Its default is this spec's: temper pushes `Corrupt`, as
  today, and no test or line here depends on the answer.

### Where the copies differ

`notes/service-parts.md` holds; this is how, for the points of the survey's table:

| Point | Holds | How |
|---|---|---|
| Trust-root bounds | temper's, none | `trust: None`, which skips the check, as today |
| `Effect::InvalidSecret` | temper's | `Output::InvalidSecret` pushes `Corrupt { owner: Token::new(0) }` |
| The output-room hold | temper's | `accounts.hold(event)` and `break` when `events.room() == 0`. `break` skips the rest of the count, as today, and `values.accounts.expire(now)` still runs |
| `aborting` | temper's | in the host; nothing here |
| `work_pending` | temper's | `marked = true` gives `!opened && !aborting`, today's term. `T/engine.rs`'s `effects` gate stays, and holds it back until the mark; `wait_state` reads the term ungated, as today |
| `untag_store` | one | the host's clears the top four bits, temper's cleared `PREFIX`: the same, as `T/engine.rs` sends `sources` only what `owns` passes, and every exchange leaves tagged `PREFIX` |
| `unkept` on `HandedIn` | jig's | unreachable from `party_account`; the arm goes with the copy |
| Root event, tokens | temper's | the adapter |
| `wait_state`, `startup_state`, `has_operations`, `return_to` | temper's reads | `wait_state` is an adapter function on the host's reads; `startup_state` was `cfg(test)` and goes to `.component().startup(0)`; `Routes::has_operations` is skein's; `return_to` is the host's |

## Steps

1. Read the item, the section and jig's adapter.
2. `T/accounts.rs` becomes, whole:

   ```rust
   //! Accounts routing around jig's host (jig/docs/design/protocol/service.md, 11.1;
   //! temper/docs/design/protocol/credentials.md, 3).
   use crate::values::Values;
   use jig_protocol_accounts::host::Output;
   pub use jig_protocol_accounts::host::{Accounts, Runtime as AccountsRuntime};
   use jig_protocol_store as store;
   use skein_io as io;
   use skein_lib::{Queue, Time, Token, Wall};
   use temper_engine_domain as root;

   const PREFIX: u64 = crate::tokens::ACCOUNTS;

   /// One pass of the host: each output is routed before the next is asked for.
   pub(crate) fn iterate(
       accounts: &mut Accounts,
       now: Time,
       wall: Wall,
       values: &mut Values,
       events: &mut Queue<root::Event>,
       store: &mut crate::store::Store,
   ) {
       let marked = store.component.mark_durable();
       let count = accounts.begin(now, wall, marked, store.installation, &mut store.sources);
       for _ in 0..count {
           match accounts.output(&mut values.accounts) {
               Some(Output::Account(event)) if events.room() == 0 => {
                   accounts.hold(event);
                   break;
               }
               Some(Output::Account(event)) => events.push(root::Event::Account(event)),
               Some(Output::Store { exchange, request }) => {
                   if let Some(event) = store.secrets(Token::new(PREFIX | exchange.raw()), request) {
                       accounts.answer(event);
                   }
               }
               Some(Output::InvalidSecret) => store.above.push(store::Event::Corrupt { owner: Token::new(0) }),
               None => (),
           }
       }
       values.accounts.expire(now);
   }

   /// Payload-free state at the shell's wait boundary; `now` is the engine's, which this host's clock follows.
   pub(crate) fn wait_state(accounts: &Accounts, now: Time) -> crate::diagnostics::WaitAccounts {
       crate::diagnostics::WaitAccounts {
           opened: accounts.opened(),
           work_pending: accounts.work_pending(now, true),
           effects: accounts.effects(),
           pending: accounts.held(),
           io: accounts.io.len(),
           empty: accounts.is_empty(),
       }
   }

   pub(crate) fn owns(token: Token) -> bool {
       token.raw() & (15_u64 << 60) == PREFIX
   }
   pub(crate) fn tag(request: io::Request) -> io::Request {
       match request.owner() {
           Some(owner) => {
               assert!(owner.raw() < (1_u64 << 60_u32), "bounded accounts exchange slots");
               request.with_owner(Token::new(owner.raw() | PREFIX))
           }
           None => request,
       }
   }
   pub(crate) fn untag(event: io::Event) -> io::Event {
       match event.owner() {
           Some(owner) => event.with_owner(Token::new(owner.untagged())),
           None => event,
       }
   }

   #[cfg(test)]
   #[path = "tests/accounts_host.rs"]
   mod tests;
   ```

   `untag` keeps its meaning: `io::Event::owner` and `with_owner` name the 12 variants its match listed, and leave
   `Resized` and `Shutdown` alone, and `owner.untagged()` is `!(15 << 60)`. Its callers strip other namespaces too.
   `engine::clock` sets `engine.env.now` and this host's clock together, so `wait_state` reads what it read.
3. The readers of the table. skein's `earliest` and `Routes` have the copies' signatures.
4. `cargo fmt`, and from `temper/`, `python3 -I -B scripts/old_forms.py`, with `--write` if a count fell. Then, one
   command at a time:

   ```
   TEMPER_BUNDLE=none heavy cargo clippy -p temper-service -p temper --all-targets -- -D warnings
   TEMPER_BUNDLE=none heavy cargo nextest run -p temper-service
   TEMPER_BUNDLE=none heavy cargo nextest run -p temper -p temper-system-world -p temper-real-loop-world -p temper-one-process-world -p temper-engine-domain-world -p temper-engine-protocol-forge-world -p temper-client-world -p temper-playground-world -p temper-playground -p temper-fake -p temper-github-app
   TEMPER_BUNDLE=none heavy cargo check -p temper-client-browser-world --tests
   ```

   Then D001's **Measure**, as the item gives it, per file. `accounts` reads 247 at this plan's base and about 25 after;
   the total falls with it. The citations check. Commit and push.

## Tests

- **All stay, as composition tests, reading the adapter:** `T/tests/accounts_host.rs`'s 15, `T/tests/accounts.rs`'s 3,
  `T/tests/engine.rs`'s. The three the survey calls host-alone, `dynamic_sources_use_provider_templates_and_restore_their_kept_records`,
  `missing_registration_runtime_is_refused_before_start` and `a_full_root_queue_retains_the_accounts_event_until_room_returns`,
  compile where they are, so they don't move (`notes/service-parts.md`, Tests). `jig-accounts-host`'s tests pin the
  host's rules in jig.
- **No test is added,** so no break is owed: `a_full_root_queue_retains_the_accounts_event_until_room_returns`,
  `aborting_does_not_submit_an_llm_key_adoption_to_the_closing_store`, `aborting_drops_a_finish_in_flight` and
  `a_restored_sources_other_kind_fails_the_read` keep pinning the hold, `aborting` and `Corrupt`, and read red if the
  adapter loses them.
- **Cited tests keep their names and modules,** so `temper/docs/design/domain/root.md`, whose citation of
  `a_cancellation_waiting_unpublished_keeps_its_completed_targets_route` still holds, needs no edit.

## Report

The common note's items. D001's measure per file, and the total. Net lines against the cap. Each file the spec's table
missed, with the edit. Any test whose assertion you wanted to change, which is a stop.
