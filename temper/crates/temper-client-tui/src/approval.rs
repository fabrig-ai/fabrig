//! The browser's approval on the TUI (temper/docs/design/client/README.md, 7.1).
use crate::Tui;
use alloc::boxed::Box;
use jig_client::{ApprovalCode, Link, NotSignedIn, Taking};
use skein_lib::Writer;
use temper_client_domain::{Domain, asked::State};

/// The approval page's address, from the TUI's own address and code.
#[must_use]
pub fn approval_address(address: &[u8], code: ApprovalCode) -> Box<[u8]> {
    let scheme = b"http://";
    let path = temper_client_domain::address::print_approve(Some(code));
    let size = scheme.len().checked_add(address.len()).expect("approval authority fits");
    let size = size.checked_add(path.len()).expect("approval address fits");
    let mut out = Writer::new(size);
    out.put(scheme).expect("scheme counted");
    out.put(address).expect("address counted");
    out.put(&path).expect("path counted");
    out.finish()
}

/// Whether Enter in the empty Link field asks for another approval.
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
            NotSignedIn::Approval(Taking::Waiting)
            | NotSignedIn::Busy
            | NotSignedIn::NotReady
            | NotSignedIn::LinkUsed
            | NotSignedIn::LinkExpired
            | NotSignedIn::LinkUnknown,
        ) => false,
    }
}
