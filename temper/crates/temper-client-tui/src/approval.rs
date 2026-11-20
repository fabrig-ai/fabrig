//! The browser's approval on the TUI (temper/docs/design/client/README.md, 4.1).
use crate::Tui;
use alloc::boxed::Box;
use jig_client::{ApprovalCode, Link, NotSignedIn, Taking};
use temper_client_domain::{Domain, asked::State};

/// The approval page's address, from the TUI's own address and code.
#[must_use]
pub fn approval_address(origin: &crate::Origin, code: ApprovalCode) -> Box<[u8]> {
    let code = match origin {
        crate::Origin::Http(_) => Some(code),
        crate::Origin::Https { .. } => None,
    };
    crate::origin::page(origin, &temper_client_domain::address::print_approve(code))
}

/// Whether the sign-in screen can ask for another approval.
#[must_use]
pub fn may_ask_again(tui: &Tui, root: &Domain) -> bool {
    let link = root.client().link();
    if link.state != Link::SignedOut || root.asked() == Some(State::Asking) {
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
