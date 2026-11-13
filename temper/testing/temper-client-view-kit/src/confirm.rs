use super as fixtures;
use jig_client::contract::{self as c, ActKind, Admission, Read, ReadValue};
use skein_lib::{List, Wall};
use temper_client_domain::address::Filter;

pub const PARTY: u64 = 73;

pub const TEXT_BYTES: u32 = 32;

pub const SINGLE_OFFER: u32 = 1;

pub const FUNDING_PERIOD: u64 = 1;

pub const FUNDING_NEEDED: u64 = 5;

pub const FUNDING_LEFT: u64 = 20;

pub const FUNDING_AFTER: u64 = 15;

pub const EMPTY_BATCH: u32 = 0;

pub const PROPOSAL: c::ProposalKey =
    c::ProposalKey { proposer: c::Proposer::Task(fixtures::INBOX_TASK), number: fixtures::INBOX_QUESTION };

#[must_use]
pub fn text(words: &[u8]) -> c::Text {
    c::Text::new(Box::from(words), TEXT_BYTES).expect("fixture text")
}

#[must_use]
pub fn root(act: ActKind, admission: Admission) -> temper_client_domain::Domain {
    let mut acts = List::with_capacity(SINGLE_OFFER);
    acts.push(c::Offered { act, admission, effect: None }).expect("one act");
    let mut funding = List::with_capacity(SINGLE_OFFER);
    funding
        .push(c::FundingOffer {
            source: c::Funder::Pool { project: fixtures::PROJECT, person: PARTY, period: FUNDING_PERIOD },
            needed: FUNDING_NEEDED,
            left: Some(FUNDING_LEFT),
            after: Some(FUNDING_AFTER),
        })
        .expect("one funding source");
    let page = c::Proposal {
        key: PROPOSAL,
        project: fixtures::PROJECT,
        projects: List::from_boxed(Box::new([fixtures::PROJECT])),
        reason: c::DocumentText::complete(text(b"Make a goal")),
        action: c::ProposalAction::Batch(List::with_capacity(EMPTY_BATCH)),
        as_holder: true,
        waiting: c::ProposalWaiting { holder: c::HistoryHolder::Person(PARTY), since: Wall::EPOCH },
        funding,
        acts,
    };
    fixtures::inbox_with_document(
        Filter::ALL,
        Some((Read::Proposal { proposal: PROPOSAL }, ReadValue::Proposal(Box::new(page)))),
    )
}
