//! The reference service's accounts: jig's host around its tokens, its root and its store
//! (jig/docs/design/protocol/service.md, 11.1).
use crate::values::Values;
use jig_domain_core as core;
use jig_protocol_accounts::host::Output;
pub use jig_protocol_accounts::host::{Accounts, Runtime as AccountsRuntime, TrustBounds};
use jig_protocol_store as store;
use jig_test_domain as root;
use skein_io as io;
use skein_lib::{Queue, Time, Token, Wall};

/// One pass of the host: each output is routed before the next is asked for.
pub(super) fn iterate(
    accounts: &mut Accounts,
    now: Time,
    wall: Wall,
    values: &mut Values,
    events: &mut Queue<root::Event>,
    store: &mut crate::store::Store,
) {
    let marked = store.host.component.mark_durable();
    let count = accounts.begin(now, wall, marked, store.installation, &mut store.host.sources);
    for _ in 0..count {
        match accounts.output(&mut values.accounts) {
            Some(Output::Account(event)) => events.push(root::Event::Core(core::Event::Account(event))),
            Some(Output::Store { exchange, request }) => {
                if let Some(event) =
                    store.host.secrets(crate::tokens::token(crate::tokens::ACCOUNTS, exchange.raw()), request)
                {
                    accounts.answer(event);
                }
            }
            Some(Output::InvalidSecret) => store.host.above.push(store::Event::PageRefused {
                owner: Token::new(0),
                failure: store::restore::Failure {
                    cause: store::restore::Cause::Decode(store::restore::DecodeFailure::Value),
                    row: None,
                },
            }),
            None => (),
        }
    }
    values.accounts.expire(now);
}

pub(super) fn owns(token: Token) -> bool {
    token.tag() == crate::tokens::ACCOUNTS
}
pub(super) fn tag(request: io::Request) -> io::Request {
    crate::tokens::tag(request, crate::tokens::ACCOUNTS)
}
pub(super) fn untag(event: io::Event) -> io::Event {
    crate::tokens::untag(event)
}

#[cfg(test)]
#[path = "tests/accounts.rs"]
mod tests;
