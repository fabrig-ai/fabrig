//! Temper's child actions and the shell's obligations.
use crate::{Address, Place};
use alloc::boxed::Box;
use skein_lib::List;
/// A view act routed exhaustively to its child.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Approve(crate::approve::Act),
    Copy { words: Box<[u8]> },
    Forge(crate::forge::Act),
    Pages(crate::pages::Act),
}
/// One external fact or act for the root to route.
#[derive(Debug)]
pub enum Event {
    /// The native client's ask for an approval (client/README.md, 4.1).
    AskApproval,
    /// The TUI's reads across projects (client/tui/launchpad.md, 2).
    Launchpad,
    /// The browser left full screen, whoever left it (client/pages/README.md).
    ScreenLeft,
    /// At start, the kept store’s look (client/pages/README.md).
    LookKept {
        look: crate::pages::terminal::Look,
    },
    Copied {
        token: u32,
        written: crate::copy::Written,
    },
    Forge(crate::forge::Event),
    Start {
        address: Address,
        kept: List<jig_client::Record>,
        with: Option<jig_client::With>,
    },
    Went {
        place: Place,
    },
    Act {
        action: Action,
    },
    Client {
        event: jig_client::Event,
    },
    AccountKey {
        value: skein_lib::Secret,
    },
    SignInAddress {
        value: skein_lib::Secret,
    },
    SignedOut,
    TabRefused,
    /// The browser's kept sign-in address (client/README.md, 4).
    SignInKept {
        source: u32,
        address: Box<[u8]>,
        refused: bool,
    },
}
/// Bounded routing and child facts for the shell to observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    UnknownAsker,
}
/// What the shell does with a sign-in's tab (client/pages/README.md, `ConnectModel`, The sign-in's tab).
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
    /// Full screen in the turn of a person’s terminal act (client/pages/README.md).
    Screen {
        on: bool,
    },
    /// Keep the look in the step that changed it (client/pages/README.md).
    KeepLook {
        look: crate::pages::terminal::Look,
    },
    /// Keep an answered sign-in beside the client's records (client/README.md, 4).
    KeepSignIn {
        source: u32,
        address: Box<[u8]>,
        refused: bool,
    },
    Tab {
        tab: Tab,
    },
    Copy {
        token: u32,
        words: Box<[u8]>,
    },
    Client {
        request: jig_client::Request,
    },
    /// What the unlinked client asks, which its host carries (jig's client/domain/client/README.md, 4.4).
    Unlinked {
        request: jig_client::UnlinkedRequest,
    },
    Show {
        address: Address,
        replace: bool,
    },
    /// Send the page away to this address, once (client/pages/README.md).
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
