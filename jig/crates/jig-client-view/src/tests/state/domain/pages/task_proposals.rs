//! The Task document supplies proposal identities independently of transcript calls.

use crate::tests::fixtures::{Fixture, Watched};
use crate::tests::state::domain::pages::*;

fn opened() -> (Fixture, jig_client::Handle) {
    let (mut domain, _) = crate::tests::state::domain::objects::live();
    let out = drive(&mut domain, Time::ZERO, Event::Went { place: Place::Jig(Address::task(8)) });
    (domain, open_token(&out))
}
fn task(keys: &[ProposalKey], activity: u64) -> Task {
    let mut task = crate::tests::state::domain::confirmation::task();
    task.context = None;
    task.activity = activity;
    task.proposals = List::with_capacity(2);
    for key in keys {
        task.proposals.push(*key).unwrap();
    }
    task
}
fn deliver(domain: &mut Fixture, watch: jig_client::Handle, value: Task) -> Box<[Request]> {
    drive(domain, Time::ZERO, Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(value))) })
}
fn proposal_read(out: &[Request], key: ProposalKey) -> jig_client::Handle {
    let mut found = None;
    for row in out {
        if let Request::Client {
            asked:
                jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Proposal { proposal }) },
        } = row
            && *proposal == key
        {
            assert!(found.replace(*read).is_none(), "one read per supplied proposal");
        }
    }
    found.expect("supplied proposal is read")
}

#[test]
fn task_proposal_registration_keeps_the_header_then_refuses_excess_distinct_makers_without_guessing_identity() {
    let task_key = ProposalKey { proposer: Proposer::Task(8), number: 1 };
    let party_key = ProposalKey { proposer: Proposer::Party(8), number: 1 };
    let (mut domain, watch) = opened();
    let out = deliver(&mut domain, watch, task(&[task_key, party_key], 11));
    proposal_read(&out, task_key);
    assert!(domain.task_page().header.is_some());
    assert_eq!(domain.task_page().proposals.as_slice(), &[task_key]);
    assert!(domain.objects().find(ObjectKey::Proposal(party_key)).is_none());
    assert!(domain.objects().find(ObjectKey::Task(8)).is_some());
    let Page::Work { excess: 1, refused: Some(Limit::Objects), .. } = domain.page() else {
        panic!("one excess object is refused in place");
    };
}
