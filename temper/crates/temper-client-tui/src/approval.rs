//! The browser's approval on the TUI (temper/docs/design/client/sign-in.md, 1.1).
use crate::Tui;
use alloc::boxed::Box;
use jig_client::{ApprovalCode, Link, NotSignedIn, Taking};
use temper_client_domain::{Domain, asked::State};

/// The approval page's address, from the TUI's own address and code.
#[must_use]
pub fn approval_address(origin: &crate::Origin, code: Option<ApprovalCode>) -> Box<[u8]> {
    crate::origin::page(origin, &temper_client_domain::address::print_approve(code))
}

/// Whether the one-click ask offers a typed code instead (client/tui/pages/sign-in.md, 3).
#[must_use]
pub fn may_type_instead(tui: &Tui, root: &Domain) -> bool {
    if tui.listener_opening() || root.client().link().state != Link::SignedOut || may_ask_again(tui, root) {
        return false;
    }
    match root.asked() {
        Some(State::Asked { port: Some(_), .. }) => true,
        None | Some(State::Asking | State::NotAsked(_) | State::Asked { port: None, .. }) => false,
    }
}

/// Whether the sign-in screen can ask for another approval.
#[must_use]
pub fn may_ask_again(tui: &Tui, root: &Domain) -> bool {
    let link = root.client().link();
    if tui.listener_opening() || link.state != Link::SignedOut || root.asked() == Some(State::Asking) {
        return false;
    }
    if tui.pasted() {
        return true;
    }
    match root.asked() {
        Some(State::NotAsked(_)) => return true,
        None | Some(State::Asking | State::Asked { .. }) => {}
    }
    match link.not_signed_in {
        Some(
            NotSignedIn::Approval(Taking::Refused | Taking::Expired | Taking::Used | Taking::Unknown)
            | NotSignedIn::Full { .. },
        ) => true,
        None
        | Some(
            NotSignedIn::Approval(Taking::Waiting | Taking::NotGranted)
            | NotSignedIn::Busy
            | NotSignedIn::NotReady
            | NotSignedIn::LinkUsed
            | NotSignedIn::LinkExpired
            | NotSignedIn::LinkUnknown,
        ) => false,
    }
}
