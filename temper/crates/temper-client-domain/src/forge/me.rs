//! The forge reads its party from the shared client's own me handle.
use super::{Request, Side};
use jig_client::{
    Client, Handle, Latest, Link, Told, Watchable,
    contract::{Delivery, Subject},
};
use skein_lib::Queue;

pub(super) const FORGES: u64 = 1;
pub(crate) fn handle() -> Handle {
    Handle { asker: crate::owner::Owner::Forge.asker(), number: 0 }
}
pub(crate) fn told(side: &mut Side, client: &Client, told: &Told, out: &mut Queue<Request>) -> bool {
    match told {
        Told::Link { link } => {
            side.link = *link;
            match link {
                Link::Starting | Link::Live | Link::Offline if !side.me => {
                    side.me = true;
                    out.push(Request::Client {
                        asked: jig_client::Event::Watch { handle: handle(), subject: Watchable::Jig(Subject::Me) },
                    });
                }
                Link::SignedOut | Link::OutOfDate => {
                    if side.me {
                        side.me = false;
                        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
                    }
                    party(side, None);
                }
                Link::Starting | Link::Live | Link::Offline => {}
            }
            side.changed();
        }
        Told::Changed { handle: value } if *value == handle() => {
            let Some(document) = client.document(*value) else { return true };
            let Some(Latest::Jig(Delivery::Me(frame))) = document.latest else { return true };
            party(side, Some(frame.party.number));
        }
        Told::Ended { handle: value, .. } if *value == handle() => {
            side.me = false;
            party(side, None);
        }
        Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Page { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => return false,
    }
    true
}
fn party(side: &mut Side, party: Option<u64>) {
    side.party = party;
    if let Some(draft) = &mut side.form {
        draft.confirmed = party.is_some();
        side.changed();
    }
}
