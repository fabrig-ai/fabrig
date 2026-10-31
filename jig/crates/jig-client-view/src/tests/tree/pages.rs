#![expect(
    clippy::wildcard_enum_match_arm,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests select scripted requests"
)]
use crate::JigBindings;
use crate::tests::fixtures::refusal_answer;
use crate::tests::fixtures::{AnsweredByEngine, Ended, Fixture, ReadPage, Served, Watched};
use crate::tests::kit::Goldens;
pub(in crate::tests) use crate::tests::kit::page as build;
use crate::tests::tree::cards::{CLIENT, document, drive, start, text, words};
use crate::{
    Event, Request,
    state::{
        address::{Address, Place},
        page::Page,
    },
};
use jig_client::contract::*;
use skein_lib::{List, Token};
use skein_ui::{Builder, Role, Tree};

pub(in crate::tests) fn deliver(d: &mut Fixture, token: jig_client::Handle, document: Delivery) -> Vec<Request> {
    drive(d, Watched { watch: token, event: WatchEvent::Delivered(document) })
}
pub(in crate::tests) fn named_binding(tree: &Tree, name: &[u8]) -> Token {
    tree.nodes().iter().find(|n| n.name.as_deref() == Some(name) && n.binding.is_some()).unwrap().binding.unwrap().token
}
pub(in crate::tests) fn special_pages(words: &crate::words::Words) {
    let mut goldens = Goldens::new("tree/pages/special_pages");
    let (mut d, f) = start();
    goldens.check(&build(&d, 2, words).0);
    deliver(&mut d, f, Delivery::Me(Box::new(document())));
    drive(&mut d, Event::Went { place: Place::Missing });
    goldens.check(&build(&d, 2, words).0);
    drive(&mut d, jig_client::Up::SignedOut);
    assert_eq!(d.page(), &Page::SigningIn);
    let (tree, _) = build(&d, 2, words);
    assert!(tree.nodes().iter().any(|node| node.slot == Some(crate::state::signin::SLOT)));
    let (mut d, f) = start();
    drive(&mut d, Served { served: false });
    assert!(deliver(&mut d, f, Delivery::Me(Box::new(document()))).is_empty());
    goldens.check(&build(&d, 2, words).0);
    let mut d = Fixture::new(&CLIENT);
    let kept = crate::tests::kit::list([crate::state::store::Record::Draft {
        version: crate::state::store::STORE_VERSION,
        party: 2,
        task: 1,
        words: text(b"private"),
        touched: 1,
    }]);
    let out =
        drive(&mut d, Event::Start { seed: 8, place: Place::Jig(Address::Inbox { project: None }), records: kept });
    let f = out
        .iter()
        .find_map(|r| match r {
            Request::Client {
                asked: jig_client::Event::Watch { handle: watch, subject: jig_client::Watchable::Jig(Subject::Me) },
            } => Some(*watch),
            _ => None,
        })
        .unwrap();
    deliver(&mut d, f, Delivery::Me(Box::new(document())));
    assert_eq!(d.page(), &Page::SomeoneElse);
    let tree = build(&d, 2, words).0;
    goldens.check(&tree);
}

fn conversations_domain() -> Fixture {
    let (mut d, f) = start();
    let mut frame = document();
    let template = frame.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap();
    template.executor = TemplateExecutor::Charter(2);
    template.budget = TemplateBudget::Amount(20);
    deliver(&mut d, f, Delivery::Me(Box::new(frame)));
    drive(&mut d, Event::Went { place: Place::Jig(Address::Conversations { project: Some(4) }) });
    d
}
fn conversations_form(words: &crate::words::Words) {
    let mut goldens = Goldens::new("tree/pages/conversations_form");
    let mut d = conversations_domain();
    let (tree, mut bindings) = build(&d, 1, words);
    goldens.check(&tree);
    assert_eq!(
        crate::decode(&mut bindings, named_binding(&tree, b"Start a conversation"), skein_ui::Act::Press),
        Some(crate::state::action::Act::BeginConversation { project: 4 })
    );
    let edit = crate::decode(
        &mut bindings,
        named_binding(&tree, b"First words"),
        skein_ui::Act::Input { value: b"Hello".to_vec().into_boxed_slice() },
    )
    .unwrap();
    drive(&mut d, Event::Act { action: edit });
    let out = drive(&mut d, Event::Act { action: crate::state::action::Act::BeginConversation { project: 4 } });
    let request = out
        .iter()
        .find_map(|r| match r {
            Request::Client { asked: jig_client::Event::Send { key: request, .. } } => Some(*request),
            _ => None,
        })
        .unwrap();
    let waiting = build(&d, 1, words).0;
    goldens.check(&waiting);
    drive(&mut d, AnsweredByEngine { request, answer: refusal_answer(Refusal::NotOffered) });
    goldens.check(&build(&d, 1, words).0);
    assert!(
        tree.nodes()
            .iter()
            .filter(|n| n.binding.is_some())
            .all(|n| n.name.as_ref().is_some_and(|name| !name.is_empty()))
    );
}

fn whole_page(d: &Fixture) -> Tree {
    let mut limits = crate::tests::tree::cards::limits();
    limits.nodes = 2000;
    limits.text_bytes = 200_000;
    limits.bindings = 512;
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.client.text);
    b.open(Role::Page);
    assert!(crate::jig(
        &mut b,
        &d.view(),
        skein_ui::SlotRef { owner: (crate::Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
        words(),
        &mut bindings
    ));
    b.close();
    let tree = b.finish();
    for node in tree.nodes() {
        if (node.role == Role::Button || node.role == Role::Tab || node.role == Role::MenuItem) && node.name.is_some() {
            assert_eq!(node.size, 1, "named control holds a node: {node:?}");
        }
    }
    tree
}

#[test]
fn each_of_jigs_pages_keeps_the_outline() {
    use crate::state::address::{ProjectSection, TaskSection};
    let mut goldens = Goldens::new("pages/frame");
    let rows = [
        ("inbox", Place::Jig(Address::Inbox { project: None })),
        ("conversations", Place::Jig(Address::Conversations { project: None })),
        ("task overview", Place::Jig(Address::Task { task: 8, section: TaskSection::Overview })),
        ("project overview", Place::Jig(Address::Project { project: 4, section: ProjectSection::Overview })),
        ("deployment", Place::Jig(Address::Deployment)),
        ("page missing", Place::Missing),
    ];
    let (mut d, watch) = start();
    let tree = whole_page(&d);
    goldens.check(&tree);
    assert_eq!(skein_ui::outline::check(&tree), None, "starting");
    deliver(&mut d, watch, Delivery::Me(Box::new(document())));
    for (label, place) in rows {
        let out = drive(&mut d, Event::Went { place });
        for request in out {
            if let Request::Client {
                asked:
                    jig_client::Event::Watch { handle: watch, subject: jig_client::Watchable::Jig(Subject::Task { .. }) },
            } = request
            {
                deliver(&mut d, watch, Delivery::Task(Box::new(crate::tests::tree::cards_objects::task_doc(&[]))));
            }
        }
        let tree = whole_page(&d);
        goldens.check(&tree);
        assert_eq!(skein_ui::outline::check(&tree), None, "{label}");
    }
    drive(&mut d, jig_client::Up::SignedOut);
    let tree = whole_page(&d);
    goldens.check(&tree);
    assert_eq!(skein_ui::outline::check(&tree), None, "signing in");
}

fn domain_with_an_owed_card() -> Fixture {
    let (mut d, watch) = start();
    deliver(&mut d, watch, Delivery::Me(Box::new(document())));
    let objects = crate::tests::kit::list([ObjectKey::Task(8)]);
    let out = drive(&mut d, Event::Named { objects, sections: List::with_capacity(0) });
    for request in out {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Task { .. }) },
        } = request
        {
            drive(
                &mut d,
                ReadPage {
                    read,
                    result: ReadResult::Value(ReadValue::Task(Box::new(crate::tests::tree::cards_objects::task_doc(
                        &[ActKind::Cancel],
                    )))),
                },
            );
        }
    }
    let id = d.objects().find(ObjectKey::Task(8)).unwrap();
    drive(
        &mut d,
        Event::Act { action: crate::state::action::Act::Intend { object: id.token(), act: ActKind::Cancel } },
    );
    assert_eq!(
        d.objects().get(id).unwrap().card,
        crate::state::objects::Card::Deciding { request: None, progress: crate::state::intents::Progress::Confirming }
    );
    drive(&mut d, Event::Act { action: crate::state::action::Act::Confirm });
    drive(&mut d, Event::Named { objects: List::with_capacity(0), sections: List::with_capacity(0) });
    d
}

#[test]
fn the_owed_cards_sit_in_their_aside_after_the_main() {
    let d = domain_with_an_owed_card();
    let tree = whole_page(&d);
    let main = tree.nodes().iter().position(|node| node.role == Role::Main).unwrap();
    let owed =
        tree.nodes().iter().position(|node| node.region == Some(crate::regions::JigRegion::Owed.region())).unwrap();
    assert_eq!(tree.nodes()[owed].role, Role::Complementary);
    assert_eq!(main + usize::try_from(tree.nodes()[main].size).unwrap(), owed);
    assert!(tree.nodes()[owed].size > 1);
    assert_eq!(skein_ui::outline::check(&tree), None);
}

#[test]
fn the_inbox_header_shows_with_either_group() {
    let mut goldens = Goldens::new("tree/pages/the_inbox_header");
    let (d, _) = crate::tests::tree::work_pages::entered(Address::Inbox { project: None });
    for group in [InboxReadGroup::Needs, InboxReadGroup::Updates] {
        let mut b = Builder::new(&crate::tests::tree::cards::limits());
        let mut bindings = JigBindings::new(64, CLIENT.client.text);
        b.open(Role::Page);
        assert!(crate::tree::sections::inbox::header(&mut b, &d.view(), words(), &mut bindings));
        assert!(crate::tree::sections::section(
            &mut b,
            &d.view(),
            words(),
            &mut bindings,
            crate::state::page::Section::Inbox { group, project: None }
        ));
        b.close();
        let tree = b.finish();
        assert_eq!(
            tree.nodes().iter().filter(|n| n.region == Some(crate::regions::JigRegion::InboxHeader.region())).count(),
            1,
            "{group:?}"
        );
        goldens.check(&tree);
    }
    let (other, _) = crate::tests::tree::work_pages::entered(Address::Conversations { project: None });
    let mut b = Builder::new(&crate::tests::tree::cards::limits());
    let mut bindings = JigBindings::new(64, CLIENT.client.text);
    assert!(!crate::tree::sections::inbox::header(&mut b, &other.view(), words(), &mut bindings));
    assert_eq!(b.finish().len(), 0);
}
#[test]
fn the_new_chat_form_is_a_form_inside_the_conversations() {
    let d = conversations_domain();
    let (tree, _) = build(&d, 1, words());
    let nodes = tree.nodes();
    let section =
        nodes.iter().position(|n| n.region == Some(crate::regions::JigRegion::Conversations.region())).unwrap();
    let form = nodes.iter().position(|n| n.region == Some(crate::regions::JigRegion::NewChat.region())).unwrap();
    assert_eq!(nodes[form].role, Role::Form);
    assert!(form > section && form < section + usize::try_from(nodes[section].size).unwrap());
    let field =
        nodes.iter().position(|n| n.role == Role::TextArea && n.name.as_deref() == Some(b"First words")).unwrap();
    assert!(field > form && field < form + usize::try_from(nodes[form].size).unwrap());
}

#[test]
fn a_missing_section_says_so_in_its_own_region() {
    let mut goldens = Goldens::new("tree/pages/a_missing_section");
    let (mut domain, frame) = start();
    deliver(&mut domain, frame, Delivery::Me(Box::new(document())));
    drive(&mut domain, Event::Went { place: Place::Elsewhere });
    let section = crate::state::page::Section::Task { task: 8, section: crate::state::address::TaskSection::Plan };
    let sections = crate::tests::kit::list([section]);
    let opened = drive(&mut domain, Event::Named { objects: List::with_capacity(0), sections });
    let watch = opened
        .iter()
        .find_map(|r| match r {
            Request::Client { asked: jig_client::Event::Watch { handle: watch, .. } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    drive(&mut domain, Ended { watch, end: WatchEnd::NotAllowed });
    let mut builder = Builder::new(&crate::tests::tree::cards::limits());
    let mut bindings = JigBindings::new(256, CLIENT.client.text);
    assert!(crate::tree::sections::section(&mut builder, &domain.view(), words(), &mut bindings, section));
    let tree = builder.finish();
    assert_eq!(tree.nodes()[0].region, Some(crate::regions::JigRegion::TaskPlan.region()));
    assert_eq!(tree.nodes()[0].size, tree.len());
    goldens.check(&tree);
}

#[test]
fn pages_show_non_work_states_and_the_offered_conversation_form() {
    let words = words();
    special_pages(words);
    conversations_form(words);
}
