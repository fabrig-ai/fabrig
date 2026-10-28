//! Temper's child actions and the shell's obligations.
use crate::{Address, Place};
use alloc::boxed::Box;
use skein_lib::List;
/// A view act routed exhaustively to its child.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Forge(crate::forge::Act),
    Pages(crate::pages::Act),
}
/// One external fact or act for the root to route.
#[derive(Debug)]
pub enum Event {
    Forge(crate::forge::Event),
    Start { address: Address, kept: List<jig_client::Record>, with: Option<jig_client::With> },
    Went { place: Place },
    Act { action: Action },
    Client { event: jig_client::Event },
    AccountKey { value: skein_lib::Secret },
    SignInAddress { value: skein_lib::Secret },
    SignedOut,
}
/// Bounded routing and child facts for the shell to observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    UnknownAsker,
}
/// One shell obligation; keyed sends come only from the table.
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Client {
        request: jig_client::Request,
    },
    Show {
        address: Address,
        replace: bool,
    },
    /// Send the page away to this address, once (client/README.md, 3.5).
    Leave {
        to: Box<[u8]>,
        replace: bool,
    },
    SignOut,
    Dropped {
        count: u32,
    },
    Fact {
        fact: Fact,
    },
}
