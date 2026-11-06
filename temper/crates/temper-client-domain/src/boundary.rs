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
    TabRefused,
}
/// Bounded routing and child facts for the shell to observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    UnknownAsker,
}
/// What the shell does with a sign-in's tab (client/README.md, 3.5, `ConnectModel`, The sign-in's tab).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tab {
    /// In the press's own turn, before the sign-in is carried; the provider names its words.
    Open { provider: u32 },
    /// The answer's address, its opener cut: once, from that answer.
    Point { to: Box<[u8]> },
    /// An answer with no address, or one the page has left.
    Close,
}
/// One shell obligation; keyed sends come only from the table.
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Tab {
        tab: Tab,
    },
    Client {
        request: jig_client::Request,
    },
    /// What the unlinked client asks, which its host carries (jig's client.md, 4.4).
    Unlinked {
        request: jig_client::UnlinkedRequest,
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
