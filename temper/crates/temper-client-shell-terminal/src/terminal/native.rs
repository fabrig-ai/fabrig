//! The native client's terminal face; ordinary transport notices remain local.
use super::{Client, Notice};
use crate::{
    client::{NativeClient, Shown},
    route,
};
pub(crate) fn event(event: &temper_client_domain::Event) -> Option<Notice> {
    use jig_client::Up;
    use jig_client::contract::{ReadNone, ReadResult, WatchEnd};
    use temper_client_domain::Event;
    if matches!(
        event,
        Event::SignedOut
            | Event::Client {
                event: jig_client::Event::Up {
                    up: Up::WatchEnded { end: WatchEnd::SignedOut, .. }
                        | Up::ReadDone { result: jig_client::Readout::Jig(ReadResult::None(ReadNone::SignedOut)), .. }
                }
            }
    ) {
        Some(Notice::SignedOut)
    } else {
        None
    }
}
pub(crate) fn signing(signing: &jig_client_protocol_link::Signing, out: bool) -> Option<Notice> {
    use jig_client_protocol_link::Signing;
    match signing {
        Signing::Again | Signing::Lost if !out => Some(Notice::SignInAgain),
        Signing::Busy if !out => Some(Notice::SignInBusy),
        Signing::Full { sign_ins, earliest, link_until } if !out => {
            Some(Notice::SignInFull { sign_ins: *sign_ins, earliest: *earliest, link_until: *link_until })
        }
        Signing::Refused if !out => Some(Notice::SignInRefused),
        Signing::SignedIn { .. }
        | Signing::SignedOut
        | Signing::Again
        | Signing::Busy
        | Signing::Full { .. }
        | Signing::Lost
        | Signing::Refused => None,
    }
}
impl Client for NativeClient {
    fn tree(&self) -> &skein_ui::Tree {
        Self::tree(self)
    }
    fn built(&self) -> u64 {
        self.seen().builds
    }
    fn act(&mut self, node: skein_ui::NodeId, act: skein_ui::Act) -> bool {
        Self::act(self, node, act)
    }
    fn sign_in(&mut self, link: &[u8]) -> Result<(), jig_client_protocol_http::Unlinked> {
        Self::sign_in(self, link)
    }
    fn notice(&mut self) -> Option<Notice> {
        while self.signing().is_some() {}
        while let Some(shown) = self.notices() {
            match shown {
                Shown::Terminal(notice) => return Some(notice),
                Shown::Notice(route::Notice::Credential { party, credential }) => {
                    return Some(Notice::Credential { party, token: credential });
                }
                Shown::Notice(route::Notice::Dropped { count }) => return Some(Notice::Dropped { count }),
                Shown::Notice(
                    route::Notice::Leave { .. }
                    | route::Notice::Unread
                    | route::Notice::Served { .. }
                    | route::Notice::Unasked
                    | route::Notice::Fact(_)
                    | route::Notice::SignedIn
                    | route::Notice::SignedOut,
                ) => {}
            }
        }
        None
    }
    fn close(&mut self) {
        Self::close(self);
    }
}
