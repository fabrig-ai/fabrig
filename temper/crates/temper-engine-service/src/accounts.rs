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

/// The owner of a secret of the wrong kind, above every exchange the accounts' host issues.
pub(crate) const WRONG_KIND: Token = Token::new(PREFIX | (1_u64 << 59));

/// One pass of the host: each output is routed before the next is asked for.
pub(crate) fn iterate(
    accounts: &mut Accounts,
    now: Time,
    wall: Wall,
    values: &mut Values,
    events: &mut Queue<root::Event>,
    store: &mut crate::store::Store,
) {
    let marked = store.host.mark_durable();
    let count = accounts.begin(now, wall, marked, store.installation, &mut store.host.sources);
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
            Some(Output::InvalidSecret) => store.host.above.push(store::Event::Corrupt { owner: WRONG_KIND }),
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
        empty: accounts.idle(),
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
