//! The terminal's card lines (temper/docs/design/client/pages/README.md, A proposal or an Accept card).
use crate::words::Words;
use alloc::boxed::Box;
use jig_client::contract as c;
use temper_client_domain::pages::inbox::Card;
/// A delegation's one line (temper/docs/design/client/pages/README.md, A proposal or an Accept card).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DelegationLine {
    pub kind: Box<[u8]>,
    pub title: Box<[u8]>,
    pub state: Box<[u8]>,
}
/// The delegated task's kind, title and state (temper/docs/design/client/pages/README.md, A proposal or an Accept card).
#[must_use]
pub fn delegation_line(task: &c::Task, w: &Words) -> DelegationLine {
    crate::cards::delegation::line(task, w)
}
/// The title after Proposal, absent on other cards (temper/docs/design/client/pages/README.md, A proposal or an Accept card).
#[must_use]
pub fn proposal_line(card: &Card, w: &Words) -> Option<Box<[u8]>> {
    crate::cards::proposal::line(card, w)
}
