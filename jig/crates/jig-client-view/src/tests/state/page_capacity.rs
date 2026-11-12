#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary maximum fixtures script public sources and inspect node budgets"
)]
use crate::tests::fixtures::{Fixture, ReadPage};
use crate::tests::tree::{
    cards::{CLIENT, document, drive, start, words},
    pages::deliver,
    work_pages::list,
};
use crate::{JigBindings, Owner, words::Words};
use skein_ui::{Builder, Role, SlotRef, Tree};

use crate::{
    Event, Request,
    state::{
        action::Act,
        address::{Address, Place, TaskSection},
    },
};
use jig_client::contract::*;
use skein_lib::List;
use skein_lib::Token;
fn full() -> Text {
    let mut bytes = Vec::new();
    bytes.resize(usize::try_from(CLIENT.client.text).unwrap(), b'x');
    Text::new(bytes.into_boxed_slice(), CLIENT.client.text).unwrap()
}

pub(in crate::tests) fn entered(address: Address) -> (Fixture, Vec<Request>) {
    let (mut d, watch) = start();
    let mut frame = document();
    let mut project = frame.projects.get(0).unwrap().clone();
    project.name = DocumentText::complete(full());
    project.role = HeldRole::Policy { role: 1 };
    let mut second = project.clone();
    second.project = 5;
    second.role = HeldRole::Policy { role: 2 };
    frame.projects = list([project, second]);
    frame.party.name = DocumentText::complete(full());
    frame.party.login = DocumentText::complete(full());
    deliver(&mut d, watch, Delivery::Me(Box::new(frame)));
    let out = drive(&mut d, Event::Went { place: Place::Jig(address) });
    (d, out)
}
pub(in crate::tests) fn render(d: &Fixture, words: &Words) -> Tree {
    let limits = crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, words).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.client.text);
    b.open(Role::Page);
    assert!(crate::jig(
        &mut b,
        &d.view(),
        SlotRef { owner: (Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
        words,
        &mut bindings
    ));
    b.close();
    let tree = b.finish();
    assert!(u32::try_from(tree.nodes().len()).unwrap() <= limits.nodes);
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Heading { level: 1 }).count(), 1);
    assert!(
        tree.nodes()
            .iter()
            .filter(|n| n.binding.is_some())
            .all(|n| n.name.as_ref().is_some_and(|name| !name.is_empty()))
    );
    tree
}

fn not_the_work(words: &Words) {
    let mut d = Fixture::new(&CLIENT);
    drive(&mut d, Event::Start { seed: 8, place: Place::Jig(Address::Inbox { project: None }), records: list([]) });
    drive(&mut d, jig_client::Up::SignedOut);
    render(&d, words);
    let (d, _) = start();
    render(&d, words);
}

#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::print_stdout,
    reason = "reports configured capacity evidence for the composition handoff"
)]
fn capacity_composes_siblings_fragments_and_application_pages_with_checked_overflow() {
    let words = words();
    let limits = crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, words).unwrap();
    let largest = crate::page_nodes(&CLIENT, &(CLIENT).client, 0).unwrap();
    assert_eq!(
        limits.nodes,
        largest
            .checked_add(
                crate::banner_nodes(&CLIENT, &(CLIENT).client)
                    .unwrap()
                    .checked_add(crate::navigation_nodes(&CLIENT, &(CLIENT).client).unwrap())
                    .unwrap()
            )
            .unwrap()
            .checked_add(crate::confirmation_nodes(&CLIENT, &(CLIENT).client).unwrap())
            .unwrap()
            .checked_add(crate::notices_nodes(&CLIENT, &(CLIENT).client).unwrap())
            .unwrap()
            .checked_add(3)
            .unwrap()
    );
    let extra = crate::sizes::limits::compose(
        &CLIENT,
        &(CLIENT).client,
        words,
        &crate::state::page::Arrangement::none(),
        largest.checked_add(7).unwrap(),
        9,
        2,
    )
    .unwrap();
    assert_eq!(extra.nodes, limits.nodes.checked_add(16).unwrap());
    assert_eq!(extra.depth, limits.depth.checked_add(2).unwrap());
    assert_eq!(crate::page_nodes(&CLIENT, &(CLIENT).client, 9), largest.checked_add(9));
    assert!(
        crate::sizes::limits::compose(
            &CLIENT,
            &(CLIENT).client,
            words,
            &crate::state::page::Arrangement::none(),
            u32::MAX,
            0,
            0
        )
        .is_none()
    );
    assert!(crate::page_nodes(&CLIENT, &(CLIENT).client, u32::MAX).is_none());
    let mut huge = CLIENT;
    huge.client.rows = u32::MAX;
    assert!(crate::page_nodes(&huge, &(huge).client, 0).is_none());

    let (mut d, _) = start();
    drive(&mut d, Event::Went { place: Place::Elsewhere });
    let mut b = Builder::new(&extra);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.client.text);
    b.open(Role::Page);
    assert!(!crate::jig(
        &mut b,
        &d.view(),
        SlotRef { owner: (Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
        words,
        &mut bindings
    ));
    b.open(Role::Main);
    b.open(Role::Heading { level: 1 });
    b.text(skein_ui::Run::Plain, b"Application page");
    b.close();
    b.close();
    crate::after_page(&mut b, &d.view(), words, &mut bindings);
    b.close();
    let slots = b.slots(0);
    assert_eq!(slots.len(), 1);
    let mut fragment = Builder::new(&extra);
    fragment.open(Role::Region);
    fragment.name(b"Application navigation");
    fragment.open(Role::Button);
    fragment.name(b"Home");
    fragment.close();
    fragment.close();
    b.splice(slots[0].id, fragment);
    let tree = b.finish();
    assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::Heading { level: 1 }).count(), 1);
    println!(
        "fixture page nodes={largest}, composed nodes={}, depth={}, text/node={}, text bytes={}, bindings={}, slots={}, patches={}",
        limits.nodes,
        limits.depth,
        limits.text_per_node,
        limits.text_bytes,
        limits.bindings,
        limits.slots,
        limits.patches
    );
}

#[test]
fn each_page_bound_keeps_its_value_at_the_client_limits() {
    // Added-dependency words and provenance, circle chips and their fallbacks
    // enlarge the shared chip envelope and the standalone owner metadata.
    // Pin the resulting page totals so that later bound changes stay visible.
    const DEPLOYMENT_NODES: u32 = 10_937;
    assert_eq!(
        [
            crate::inbox_nodes(&CLIENT, &(CLIENT).client, 0),
            crate::conversations_nodes(&CLIENT, &(CLIENT).client, 0),
            crate::conversation_nodes(&CLIENT, &(CLIENT).client, 0),
            crate::task_nodes(&CLIENT, &(CLIENT).client, 0),
            crate::project_nodes(&CLIENT, &(CLIENT).client, 0),
            crate::deployment_nodes(&CLIENT, &(CLIENT).client, 0),
            crate::not_the_work_nodes(&CLIENT, &(CLIENT).client, 0),
            crate::page_nodes(&CLIENT, &(CLIENT).client, 0)
        ],
        [
            Some(421_723),
            Some(106_538),
            Some(1_261_704),
            Some(1_457_191),
            Some(632_709),
            Some(DEPLOYMENT_NODES),
            Some(25),
            Some(1_457_191)
        ]
    );
}

#[test]
fn an_arranged_task_page_counts_the_sum_of_its_sections() {
    let sections = [
        TaskSection::Overview,
        TaskSection::Activity,
        TaskSection::Plan,
        TaskSection::History,
        TaskSection::Budget,
        TaskSection::Panels,
    ];
    let counts = sections.map(|s| crate::task_section_nodes(&CLIENT, &(CLIENT).client, s).unwrap());
    assert_eq!(
        (crate::task_header_nodes(&CLIENT, &(CLIENT).client).unwrap(), counts),
        (52_952, [228_749, 1_404_239, 390_868, 17_504, 71, 94])
    );
    let arrangement = crate::state::page::Arrangement { task: list(sections), project: list([]) };
    assert_eq!(
        crate::arranged_nodes(&CLIENT, &(CLIENT).client, 0, &arrangement),
        Some(52_952 + 228_749 + 1_404_239 + 390_868 + 17_504 + 71 + 94)
    );
}
#[test]
fn no_arrangement_keeps_each_page_bound() {
    each_page_bound_keeps_its_value_at_the_client_limits();
    let arrangement = crate::state::page::Arrangement::none();
    assert_eq!(crate::arranged_nodes(&CLIENT, &(CLIENT).client, 0, &arrangement), Some(1_457_191));
    assert_eq!(crate::arranged_nodes(&CLIENT, &(CLIENT).client, 9, &arrangement), Some(1_457_200));
}

fn most_owed_cards() -> Fixture {
    let (mut d, watch) = start();
    deliver(&mut d, watch, Delivery::Me(Box::new(document())));
    let objects = list((1..=CLIENT.objects).map(|n| ObjectKey::Task(u64::from(n))));
    let out = drive(&mut d, Event::Named { objects, sections: List::with_capacity(0) });
    for request in out {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Task { task }) },
        } = request
        {
            let mut doc = crate::tests::tree::cards_objects::task_doc(&[ActKind::Cancel]);
            doc.chip.task = task;
            drive(&mut d, ReadPage { read, result: ReadResult::Value(ReadValue::Task(Box::new(doc))) });
        }
    }
    for number in 1..=CLIENT.objects {
        let id = d.objects().find(ObjectKey::Task(u64::from(number))).unwrap();
        drive(&mut d, Event::Act { action: Act::Intend { object: id.token(), act: ActKind::Cancel } });
        drive(
            &mut d,
            Event::Act {
                action: Act::Edit {
                    field: crate::state::fields::Purpose::Reason,
                    text: crate::tests::tree::cards::text(b"cancel"),
                },
            },
        );
        if number < CLIENT.objects {
            drive(&mut d, Event::Act { action: Act::Confirm });
        }
    }
    drive(&mut d, Event::Named { objects: List::with_capacity(0), sections: List::with_capacity(0) });
    d
}
#[test]
fn the_most_owed_cards_fit_the_pages_bound() {
    let d = most_owed_cards();
    for (_, id) in d.objects().ids() {
        assert!(d.owes(*id));
    }
    assert_eq!(d.objects().len(), CLIENT.objects);
    let limits = crate::sizes::limits::of(&CLIENT, &(CLIENT).client, &crate::state::page::Arrangement::none()).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.client.text);
    b.open(Role::Page);
    crate::tree::pages::after_page(&mut b, &d.view(), words(), &mut bindings);
    b.close();
    assert!(b.finish().len() <= limits.nodes);
}

#[test]
fn pages_outside_the_work_keep_composed_capacity() {
    not_the_work(words());
}
