//! Shared proposals are rendered from supplied Task-page names, not invented calls.
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary proofs drive public events and inspect semantic trees"
)]
use crate::JigBindings;
use crate::tests::fixtures::{Fixture, ReadPage};
use crate::tests::tree::{
    cards::{CLIENT, contains, drive, text, words},
    pages::deliver,
    work_pages::{entered, list},
};
use crate::{
    Request,
    state::address::{Address, TaskSection},
};
use jig_client::contract::*;
use skein_lib::Wall;
use skein_ui::{Builder, Role, Tree};
fn key(proposer: Proposer) -> ProposalKey {
    ProposalKey { proposer, number: 1 }
}
fn task(keys: &[ProposalKey], activity: u64) -> Task {
    let mut doc = crate::tests::tree::cards_objects::task_doc(&[]);
    doc.chip.executor = Executor::Agent { charter: 2 };
    doc.chip.phase = Phase::Running;
    *doc.chip.details = crate::tests::fixtures::chip_phase(OwnerChipPhase::Active(RunStage::Running { attempt: 2 }));
    doc.context = None;
    doc.hold = None;
    doc.proposals = list(keys.iter().copied());
    doc.activity = activity;
    doc
}
fn opened(keys: &[ProposalKey]) -> (Fixture, jig_client::Handle, Vec<Request>) {
    let (mut d, out) = entered(Address::Task { task: 8, section: TaskSection::Activity });
    let watch =
        out.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    let out = deliver(&mut d, watch, Delivery::Task(Box::new(task(keys, 11))));
    (d, watch, out)
}

fn token(out: &[Request], key: ProposalKey) -> jig_client::Handle {
    out.iter()
        .find_map(|r| match r {
            Request::Client { asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Proposal { proposal }) } } if *proposal == key => Some(*read),
            _ => None,
        })
        .unwrap()
}

fn build(d: &Fixture) -> Tree {
    let tree = build_tree(d);
    crate::tests::tree::cards_access::check(&tree);
    tree
}
fn build_tree(d: &Fixture) -> Tree {
    let w = words();
    let nodes = crate::task_nodes(&CLIENT, &(CLIENT).client, 0)
        .unwrap()
        .checked_add(crate::confirmation_nodes(&CLIENT, &(CLIENT).client).unwrap())
        .unwrap()
        .checked_add(crate::notices_nodes(&CLIENT, &(CLIENT).client).unwrap())
        .unwrap()
        .checked_add(1)
        .unwrap();
    let mut limits = crate::tests::tree::cards::limits();
    limits.nodes = nodes;
    limits.depth = 34;
    limits.bindings = nodes;
    limits.slots = nodes;
    limits.text_bytes = nodes.checked_mul(limits.text_per_node).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(nodes, CLIENT.client.text);
    b.open(Role::Page);
    crate::tree::pages::main(&mut b, &d.view(), w, &mut bindings);
    crate::tree::pages::after_page(&mut b, &d.view(), w, &mut bindings);
    b.close();
    let tree = b.finish();
    assert!(tree.len() <= nodes);

    tree
}
fn cards(tree: &Tree, key: ProposalKey) -> usize {
    let wanted = crate::tree::pages::object_key(ObjectKey::Proposal(key));
    tree.nodes().iter().filter(|n| n.key == Some(wanted)).count()
}

#[test]
fn a_task_activity_renders_the_native_whole_pending_proposal_with_its_supplied_funding() {
    let k = key(Proposer::Task(8));
    let (mut d, _, out) = opened(&[k]);
    let native = OwnerProposal {
        key: k,
        project: 4,
        reason: DocumentText::complete(text(b"native whole shared reason")),
        action: OwnerProposalAction::Batch(list([])),
        as_holder: true,
        waiting: OwnerProposalWaiting { holder: OwnerHistoryHolder::Person(1), since: Wall::EPOCH },
        funding: list([OwnerFundingOffer {
            source: Funder::Pool { project: 4, person: 1, period: 3 },
            needed: 150,
            left: Some(250),
            after: Some(100),
        }]),
        acts: list([Offered { effect: None, act: ActKind::Accept, admission: Admission::Allowed }]),
    };
    drive(
        &mut d,
        ReadPage { read: token(&out, k), result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(native))) },
    );
    let tree = build(&d);
    assert_eq!(cards(&tree, k), 1);
    assert!(contains(&tree, b"native whole shared reason"));
    assert!(contains(&tree, b"150"));
    assert!(contains(&tree, b"250"));
    assert!(contains(&tree, b"100"));
    assert!(tree.nodes().iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Accept")));
}
