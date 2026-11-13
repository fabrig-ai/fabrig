use super::fixtures;

use jig_client::contract::*;

use skein_lib::{Env, List, Queue, Time, Token, Wall};

use skein_ui::{Builder, Role};

use temper_client_domain::ChatsFilter;

use temper_client_domain::{Domain, Request};


pub(super) use super::fixtures::chats::{
    HELD_STATUSES, TASK, TEXT, deliver_live, deployment_chats_root, line, listed, listed_root, no_project_root,
    open_chats, row_status_styles, status_rows,
};


pub(super) fn tree(line: &TaskLine) -> (skein_ui::Tree, crate::frame::Bindings) {
    tree_at(line, fixtures::NOW)
}

fn tree_at(line: &TaskLine, now: Wall) -> (skein_ui::Tree, crate::frame::Bindings) {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
    crate::pages::chats::row(
        &mut b,
        &fixtures::root_in_project(Some(fixtures::PROJECT)),
        line,
        &fixtures::words(),
        &mut bindings,
        &mut crate::words::Times::new(now),
    );
    (b.finish(), bindings)
}

fn style_index(tree: &skein_ui::Tree, style: skein_ui::Style) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&style) {
            return index;
        }
    }
    panic!("missing style {style:?}");
}

fn role_count(tree: &skein_ui::Tree, role: Role) -> usize {
    let mut count = 0_usize;
    for node in tree.nodes() {
        if node.role == role {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}

#[test]
fn a_chats_row_links_to_its_chat_by_number() {
    let (tree, bindings) = tree(&line());
    let mut bound = None;
    for node in tree.nodes() {
        if node.role == Role::Link && node.name.as_deref() == Some(b"T9") {
            bound = node.binding;
        }
    }
    assert_eq!(role_count(&tree, Role::Link), 1);
    let mut view = fixtures::page_view(&listed_root());
    let id = fixtures::find(view.tree(), Role::Link, b"T9").unwrap();
    assert_eq!(crate::href(&mut view, id).as_deref(), Some(b"/chats/9".as_slice()));
    let binding = bound.expect("bound link");
    assert_eq!(
        bindings.decode(binding.token, skein_ui::Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: temper_client_domain::Address::Chat { task: TASK }
        }))
    );
}


#[test]
fn chats_have_one_list_or_the_empty_sentence() {
    let empty = fixtures::page_view(&fixtures::chats_root());
    assert!(fixtures::find(empty.tree(), Role::Heading { level: 1 }, b"Talk to an agent.").is_some());
    assert!(fixtures::styles(empty.tree(), Role::List).is_empty());
    assert!(fixtures::find(empty.tree(), Role::Paragraph, b"None.").is_some());
    let view = fixtures::page_view(&listed_root());
    assert_eq!(fixtures::styles(view.tree(), Role::List).as_ref(), [Box::from([crate::look::CHATS_ROWS])]);
    let list = fixtures::find(view.tree(), Role::List, b"Your chats").expect("named chats list");
    let nodes = view.tree().nodes();
    for (index, node) in nodes.iter().enumerate() {
        if node.id == list {
            assert_eq!(node.role, Role::List);
            let end = index.checked_add(usize::try_from(node.size).expect("list size")).expect("list end");
            let mut child = index + 1;
            for task in [8, 9] {
                assert_eq!(nodes[child].role, Role::Item);
                assert_eq!(nodes[child].key.expect("chat line key").a, task);
                child = child.checked_add(usize::try_from(nodes[child].size).expect("line size")).expect("next line");
            }
            assert_eq!(child, end, "both lines belong to the named list");
        }
    }
}


#[test]
fn playground_chats_show_the_list_without_the_start() {
    let root = fixtures::playground_root(
        temper_client_domain::Address::Chats { filter: ChatsFilter::All },
        fixtures::PROJECT,
        true,
    );
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Heading { level: 2 }, b"Live chats").is_none());
    assert!(fixtures::find(view.tree(), Role::Heading { level: 2 }, b"Closed chats").is_none());
    assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, b"Talk to an agent about acme/shop.").is_some());
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"None.").is_some());
    let mut heading = None;
    for node in view.tree().nodes() {
        if node.role == (Role::Heading { level: 1 }) {
            assert!(heading.is_none(), "one level-1 heading");
            heading = node.name.as_deref();
        }
    }
    assert_eq!(heading, Some(b"Talk to an agent about acme/shop.".as_slice()));
    assert!(fixtures::find(view.tree(), Role::List, b"Suggested requests").is_none());
    assert!(fixtures::find(view.tree(), Role::Status, b"Ready").is_none());
    assert!(fixtures::find(view.tree(), Role::Text, b"acme/shop").is_none());
    assert!(fixtures::find(view.tree(), Role::Button, b"Try again").is_none());
}


#[test]
fn a_live_playground_chat_is_listed_under_your_chats() {
    use temper_client_domain::Address;
    let mut root = fixtures::playground_root(
        Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
        fixtures::PROJECT,
        true,
    );
    let limits = fixtures::domain_limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    let (watch, _) = open_chats(&mut root, &env, &mut out);
    let mut rows = List::with_capacity(1);
    rows.push(line()).expect("live row");
    deliver_live(&mut root, &env, &mut out, watch, rows);
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, b"Talk to an agent about acme/shop.").is_some());
    assert!(fixtures::find(view.tree(), Role::Link, b"T9").is_some());
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"Hello").is_some());
    assert!(fixtures::find(view.tree(), Role::Heading { level: 2 }, b"Closed chats").is_none());
    assert!(fixtures::find(view.tree(), Role::Button, b"More closed chats").is_none());
}


fn filter(root: &mut Domain, filter: ChatsFilter) {
    fixtures::root_event(
        root,
        temper_client_domain::Event::Went {
            place: temper_client_domain::Place::Address(temper_client_domain::Address::Chats { filter }),
        },
    );
}


fn numbers(tree: &skein_ui::Tree) -> Box<[u64]> {
    let mut numbers = List::with_capacity(u32::try_from(tree.nodes().len()).expect("tree nodes fit"));
    for node in tree.nodes() {
        if node.role == Role::Item
            && let Some(key) = node.key
        {
            numbers.push(key.a).expect("one key per node");
        }
    }
    numbers.into_boxed()
}

fn timed(task: u64, activity: u64, ended: Option<u64>) -> TaskLine {
    let mut line = line();
    line.chip.task = task;
    line.last_activity = Wall::from_nanos(activity);
    if let Some(ended) = ended {
        line.ended_at = Some(Wall::from_nanos(ended));
    }
    line
}

#[test]
fn one_list_merges_activity_and_end_times_with_lower_numbers_first_on_a_tie() {
    // Closed T8's activity is newer than everything, but its end is the tie.
    let root =
        listed(&[timed(9, 90, None), timed(6, 40, None)], &[timed(8, 200, Some(40)), timed(7, 0, Some(100))], false);
    let view = fixtures::page_view(&root);
    assert_eq!(numbers(view.tree()).as_ref(), [7, 9, 6, 8]);
    assert_eq!(fixtures::styles(view.tree(), Role::List).as_ref(), [Box::from([crate::look::CHATS_ROWS])]);
}

#[test]
fn each_filter_selects_its_lines_and_presses_its_bound_button() {
    use temper_client_domain::{Action, address::ChatsFilter as Filter, pages::Act};
    for (selected, name, expected) in [
        (Filter::All, b"All".as_slice(), [8, 9].as_slice()),
        (Filter::Live, b"Live", [9].as_slice()),
        (Filter::Closed, b"Closed", [8].as_slice()),
    ] {
        let mut root = listed_root();
        filter(&mut root, selected);
        let mut view = fixtures::page_view(&root);
        assert_eq!(numbers(view.tree()).as_ref(), expected);
        for (filter, label) in [(Filter::All, b"All".as_slice()), (Filter::Live, b"Live"), (Filter::Closed, b"Closed")]
        {
            let id = fixtures::find(view.tree(), Role::Button, label).expect("filter button");
            for node in view.tree().nodes() {
                if node.id == id {
                    assert_eq!(node.states.as_slice().contains(&skein_ui::State::Pressed), label == name);
                }
            }
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(Action::Pages(Act::ChatFilter { filter }))
            );
        }
        for task in expected {
            let label = match *task {
                8 => b"T8",
                9 => b"T9",
                _ => panic!("fixture task"),
            };
            let id = fixtures::find(view.tree(), Role::Link, label).expect("TN link");
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(Action::Pages(Act::Go { address: temper_client_domain::Address::Chat { task: *task } }))
            );
        }
    }
}

#[test]
fn empty_filters_all_say_none() {
    use temper_client_domain::address::ChatsFilter as Filter;
    for (selected, words) in [(Filter::All, b"None.".as_slice()), (Filter::Live, b"None."), (Filter::Closed, b"None.")]
    {
        let mut root = listed(&[], &[], false);
        filter(&mut root, selected);
        let view = fixtures::page_view(&root);
        assert!(fixtures::find(view.tree(), Role::Paragraph, words).is_some());
        assert!(fixtures::styles(view.tree(), Role::List).is_empty());
    }
    for (selected, live, ended, words) in
        [(Filter::Live, false, true, b"None.".as_slice()), (Filter::Closed, true, false, b"None.")]
    {
        let active = [line()];
        let closed = [timed(8, 0, Some(0))];
        let mut root = listed(if live { &active } else { &[] }, if ended { &closed } else { &[] }, false);
        filter(&mut root, selected);
        assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Paragraph, words).is_some());
    }
}

#[test]
fn more_closed_chats_is_bound_only_under_all_or_closed_with_an_older_page() {
    use temper_client_domain::{Action, address::ChatsFilter as Filter, pages::Act};
    for more in [false, true] {
        for selected in [Filter::All, Filter::Live, Filter::Closed] {
            let mut root = listed(&[line()], &[timed(8, 0, Some(0))], more);
            filter(&mut root, selected);
            let mut view = fixtures::page_view(&root);
            let button = fixtures::find(view.tree(), Role::Button, b"More closed chats");
            assert_eq!(button.is_some(), more && selected != Filter::Live);
            if let Some(id) = button {
                assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(Action::Pages(Act::ClosedChats)));
            }
        }
    }
}


#[test]
fn an_accepted_start_draws_no_applied_status() {
    use temper_client_domain::{self as d, Action, Event, pages::Act};
    let mut root = fixtures::playground_root(d::Address::NewChat { draft: false }, fixtures::PROJECT, true);
    let limits = fixtures::domain_limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    for act in [Act::ChatWords { words: Box::from(b"Hello".as_slice()) }, Act::StartChat] {
        d::step(&mut root, &env, Event::Act { action: Action::Pages(act) }, &mut out);
    }
    assert!(root.pages().chats().unwrap().start.is_some());
    d::step(
        &mut root,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        Event::Client { event: jig_client::Event::Stored },
        &mut out,
    );
    let mut attempt = None;
    for request in &out {
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: token, .. } },
        } = request
        {
            attempt = Some(*token);
        }
    }
    let attempt = attempt.unwrap();
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(Answer::Done(Outcome::Document(
                        DocumentOutcome::Created { first: 1, last: 1, given: None },
                    )))),
                    why: None,
                },
            },
        },
    );
    assert!(root.pages().chats().unwrap().outcome.is_some());
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Status, b"Applied").is_none());
}


fn index_of(nodes: &[skein_ui::Node], id: skein_ui::NodeId) -> usize {
    let mut at = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.id == id {
            at = Some(index);
        }
    }
    at.expect("node in the tree")
}

fn main_of(nodes: &[skein_ui::Node]) -> usize {
    let mut main = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Main {
            main = Some(index);
        }
    }
    main.expect("a main")
}

#[test]
fn the_chats_page_opens_with_its_head_band_and_section() {
    use crate::look;
    for root in fixtures::pair(
        fixtures::chats_root(),
        fixtures::playground_root(
            temper_client_domain::Address::Chats { filter: ChatsFilter::All },
            fixtures::PROJECT,
            true,
        ),
    ) {
        let mut view = fixtures::page_view(&root);
        let nodes = view.tree().nodes();
        let main = main_of(nodes);
        assert_eq!(nodes[main].styles.as_slice(), [look::MAIN, look::CHATS_PAGE]);
        let intro = main + 1;
        assert_eq!(nodes[intro].styles.as_slice(), [look::CHATS_INTRO]);
        for node in nodes {
            assert_ne!(node.role, Role::Complementary);
        }
        let section = style_index(view.tree(), look::CHATS_SECTION);
        assert!(intro < section);
        assert_eq!(nodes[section + 1].styles.as_slice(), [look::CHATS_SECTION_HEAD]);
        let end = section + 1 + usize::try_from(nodes[section + 1].size).unwrap();
        for name in [b"All".as_slice(), b"Live", b"Closed"] {
            let button = index_of(nodes, fixtures::find(view.tree(), Role::Button, name).unwrap());
            assert!(section < button && button < end);
        }
        let link = fixtures::find(view.tree(), Role::Link, b"+ New chat").expect("new chat link");
        assert!(index_of(nodes, link) < end);
        assert_eq!(nodes[end].styles.as_slice(), [look::EMPTY]);
        assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/chats/new".as_slice()));
    }
}


#[test]
fn the_repository_context_links_into_its_project() {
    use temper_client_domain::{self as d, forge::forges};
    use temper_client_protocol_wire::v1 as w;
    for adopted in [false, true] {
        let mut parts = super::connect_forge::row(
            2,
            w::Kind::Forgejo,
            w::Mode::Owned,
            super::connect_forge::connected(),
            w::Offer::NotOffered,
        )
        .into_parts();
        parts.repositories = List::with_capacity(1);
        if adopted {
            parts
                .repositories
                .push(
                    w::Adoption::new(
                        &w::CEILINGS,
                        w::AdoptionParts {
                            owner: b"work".as_slice().into(),
                            name: b"hello".as_slice().into(),
                            project: fixtures::PROJECT,
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let mut rows = List::with_capacity(1);
        rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
        let address = d::Address::Chats { filter: ChatsFilter::All };
        let root = fixtures::forge_root(address, rows);
        let mut view = fixtures::page_view(&root);
        let mut context = None;
        for node in view.tree().nodes() {
            if node.styles.as_slice().contains(&crate::look::CHATS_BAND) {
                context = Some(node.id);
            }
        }
        assert_eq!(context.is_some(), !adopted);
        let headline: &[u8] = if adopted { b"Talk to an agent about work/hello." } else { b"Talk to an agent." };
        assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, headline).is_some());
        assert!(fixtures::find(view.tree(), Role::Paragraph, "Shop · chats".as_bytes()).is_some());
        assert!(fixtures::find(view.tree(), Role::Paragraph, DRAFT_LEDE).is_some());
        if let Some(context) = context {
            let nodes = view.tree().nodes();
            let at = index_of(nodes, context);
            assert_eq!(nodes[at].role, Role::Slot);
            let end = at + usize::try_from(nodes[at].size).unwrap();
            assert!(
                skein_lib::bytes::find(
                    &fixtures::node_text(view.tree(), at),
                    b"Chats in Shop talk and plan, but change no code until a repository is in."
                )
                .is_some()
            );
            let link = fixtures::find(view.tree(), Role::Link, b"Adopt a repository").unwrap();
            assert!(index_of(nodes, link) < end);
            assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/projects/1/adopt".as_slice()));
            assert_eq!(
                crate::act(&mut view, link, skein_ui::Act::Press),
                Some(d::Action::Pages(d::pages::Act::Go { address: d::Address::Adopt { project: fixtures::PROJECT } }))
            );
        }
        assert_eq!(d::forge::listed::without_repository(root.forges(), None, fixtures::PROJECT), Some(!adopted));
        assert_eq!(d::forge::listed::without_repository(&forges::State::Unknown, None, fixtures::PROJECT), None);
    }
}

#[test]
fn an_ended_or_refused_forges_watch_says_nothing_about_a_repository() {
    use temper_client_domain as d;
    for end in [WatchEnd::NotAllowed, WatchEnd::SubjectEnded] {
        let (mut root, watch) =
            fixtures::forge_root_watch(d::Address::Chats { filter: ChatsFilter::All }, List::with_capacity(0));
        fixtures::root_event(
            &mut root,
            d::Event::Client { event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end } } },
        );
        let view = fixtures::page_view(&root);
        for node in view.tree().nodes() {
            assert!(!node.styles.as_slice().contains(&crate::look::CHATS_BAND));
        }
    }
}

#[test]
fn the_playgrounds_project_keeps_its_context_instead_of_no_repository() {
    use temper_client_domain as d;
    for start in [false, true] {
        let root = fixtures::playground_root(
            if start { d::Address::NewChat { draft: false } } else { d::Address::Chats { filter: ChatsFilter::All } },
            fixtures::PROJECT,
            true,
        );
        let view = fixtures::page_view(&root);
        for node in view.tree().nodes() {
            assert!(!node.styles.as_slice().contains(&crate::look::CHATS_BAND));
        }
        assert_eq!(
            d::forge::listed::without_repository(root.forges(), root.forge().playground(), fixtures::PROJECT),
            None
        );
    }
}


#[test]
fn chats_offers_no_first_chat_under_any_filter_even_while_starting() {
    use temper_client_domain::{Action, Event, pages::Act};
    for selected in [ChatsFilter::All, ChatsFilter::Live, ChatsFilter::Closed] {
        let mut root = listed(&[], &[], false);
        filter(&mut root, selected);
        fixtures::root_event(
            &mut root,
            Event::Act { action: Action::Pages(Act::ChatWords { words: b"draft".as_slice().into() }) },
        );
        for pending in [false, true] {
            if pending {
                fixtures::root_event(&mut root, Event::Act { action: Action::Pages(Act::StartChat) });
            }
            let view = fixtures::page_view(&root);
            assert!(fixtures::find(view.tree(), Role::Button, b"Tell temper how you work").is_none());
        }
    }
}


#[test]
fn no_welcome_and_no_project_says_so() {
    use temper_client_domain::{
        Action, Address, Event, Place,
        pages::{Act, switcher},
    };
    let mut root = no_project_root(Some(false));
    for address in [
        Address::Chats { filter: ChatsFilter::All },
        Address::NewChat { draft: false },
        Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
        Address::Accounts,
        Address::Settings { project: fixtures::PROJECT },
    ] {
        fixtures::root_event(&mut root, Event::Went { place: Place::Address(address) });
        for words in [fixtures::words(), fixtures::browser_words()] {
            let mut view = fixtures::view_of(words);
            fixtures::build(&mut view, &root);
            let tree = view.tree();
            assert_eq!(
                fixtures::main_text(tree).as_ref(),
                b"No project yetMake your first one, or ask an owner to add you to theirs."
            );
            assert_eq!(fixtures::styles(tree, Role::Main).as_ref(), [Box::from([crate::look::OUTSIDE])]);
            let form = fixtures::find(tree, Role::Form, b"New project").expect("first project form");
            let field = fixtures::person_find(tree, Role::TextField, b"Project name").expect("project name");
            let button = fixtures::person_find(tree, Role::Button, b"Create").expect("create");
            let mut form_end = None;
            for (index, node) in tree.nodes().iter().enumerate() {
                if node.id == form {
                    form_end = Some(index.checked_add(usize::try_from(node.size).unwrap()).unwrap());
                }
                if node.id == field {
                    assert!(node.states.as_slice().contains(&skein_ui::State::SubmitsOnEnter));
                    assert!(index < form_end.expect("field in the form"));
                }
                if node.id == button {
                    assert!(index < form_end.expect("create in the form"));
                }
            }
            assert_eq!(
                fixtures::styles(tree, Role::Button).as_ref(),
                [Box::from([]), Box::from([crate::look::OUTSIDE_PRIMARY])]
            );
            assert!(fixtures::find(tree, Role::Region, b"LLM account").is_none());
            assert_eq!(
                crate::act(&mut view, field, skein_ui::Act::Input { value: b"First".as_slice().into() }),
                Some(Action::Pages(Act::Switcher(switcher::Act::NewWords { words: b"First".as_slice().into() })))
            );
            for (node, act) in [(field, skein_ui::Act::Submit), (button, skein_ui::Act::Press)] {
                assert_eq!(crate::act(&mut view, node, act), Some(Action::Pages(Act::Switcher(switcher::Act::Create))));
            }
        }
    }
}


#[test]
fn the_chats_page_offers_nothing_of_the_welcome() {
    let root = no_project_root(Some(false));
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        let tree = fixtures::build(&mut view, &root);
        for node in tree.nodes() {
            for forbidden in [b"Set up temper".as_slice(), b"Try the playground"] {
                for value in [node.name.as_deref(), node.description.as_deref()].into_iter().flatten() {
                    assert!(skein_lib::bytes::find(value, forbidden).is_none(), "{forbidden:?}");
                }
                for run in &node.text {
                    assert!(skein_lib::bytes::find(&run.text, forbidden).is_none(), "{forbidden:?}");
                }
            }
        }
    }
}


#[test]
fn welcome_or_unknown_deployments_choose_their_no_project_page() {
    use temper_client_domain::{Action, Address, Event, Place, Step, pages::Act};
    let (mut offered, watch, _, _, _) =
        fixtures::root_at_full(Address::Welcome { step: Step::Welcome }, &fixtures::domain_limits(), b"Shop");
    let mut me = offered.pages().me(offered.client()).unwrap().clone();
    me.projects.clear();
    fixtures::deliver_me(&mut offered, watch, me);
    fixtures::root_event(
        &mut offered,
        Event::Went { place: Place::Address(Address::Chats { filter: ChatsFilter::All }) },
    );
    let setup = Action::Pages(Act::Go { address: Address::Welcome { step: Step::Forge { forge: None } } });
    let cases = [
        (Box::new(no_project_root(None)), b"Talk to an agent.".as_slice(), None, None, None),
        (
            Box::new(no_project_root(Some(true))),
            b"No project yet",
            Some(b"Setting temper up makes your first project.".as_slice()),
            Some(setup.clone()),
            None,
        ),
        (
            Box::new(offered),
            b"No project yet",
            Some(b"Setting temper up makes your first project.".as_slice()),
            Some(setup),
            Some(Action::Pages(Act::Playground)),
        ),
    ];
    for (root, heading, line, setup, playground) in cases {
        for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, heading).is_some());
            assert!(fixtures::find(view.tree(), Role::Form, b"New project").is_none());
            let paragraph =
                fixtures::find(view.tree(), Role::Paragraph, b"Setting temper up makes your first project.");
            assert_eq!(paragraph.is_some(), line.is_some());
            let primary = fixtures::person_find(view.tree(), Role::Link, b"Set up temper");
            let secondary = fixtures::person_find(view.tree(), Role::Button, b"Try the playground");
            for (node, expected) in [(primary, setup.clone()), (secondary, playground.clone())] {
                let action = match node {
                    Some(id) => crate::act(&mut view, id, skein_ui::Act::Press),
                    None => None,
                };
                assert_eq!(action, expected);
            }
        }
    }
}


#[test]
fn a_known_empty_project_list_is_required_for_no_project() {
    let template = fixtures::root_in_project(None);
    let me = template.pages().me(template.client()).unwrap().clone();
    for me in [None, Some(me)] {
        let root = deployment_chats_root(Some(false), me);
        let view = fixtures::page_view(&root);
        assert!(fixtures::find(view.tree(), Role::Complementary, b"Project").is_none());
        assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, b"No project yet").is_none());
    }
}


#[test]
fn chats_keeps_its_list_without_playground_suggestions_or_composer() {
    use crate::look;
    use temper_client_domain::Address;
    for root in fixtures::pair(
        fixtures::chats_root(),
        fixtures::playground_root(Address::Chats { filter: ChatsFilter::All }, fixtures::PROJECT, true),
    ) {
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let tree = view.tree();
            assert_eq!(role_count(tree, Role::Heading { level: 1 }), 1);
            for node in tree.nodes() {
                assert_ne!(node.role, Role::TextArea, "Chats has no composer");
                for style in [look::ENTRY, look::HINT, look::CHOICE] {
                    assert!(!node.styles.as_slice().contains(&style), "Chats has no composer parts");
                }
                for name in [b"Start".as_slice(), b"Start chat"] {
                    assert!(node.role != Role::Button || node.name.as_deref() != Some(name));
                }
            }
            assert!(fixtures::find(tree, Role::List, b"Suggested requests").is_none());
        }
    }
}


const DRAFT_GOAL: u64 = 42;

const DRAFT_LEDE: &[u8] = b"A chat answers a question or changes the code, and can write a draft beside it. A draft becomes a goal for bigger work.";


struct DraftsPage {
    root: Domain,
    out: Queue<Request>,
    goal_watch: Option<Token>,
    note_reads: u32,
}

impl DraftsPage {
    #[expect(clippy::too_many_lines, reason = "the fixture answers Chats through the public client events")]
    fn new(offered: bool, asking: bool, linked: bool) -> Self {
        use jig_client::{Down, Readable, Up, Watchable};
        use temper_client_domain::{Address, Event};
        use temper_client_protocol_wire::{self as wire, v1 as w};
        let limits = fixtures::domain_limits();
        let env = Env { now: Time::ZERO, wall: fixtures::NOW, limits };
        let source = fixtures::chat::messaging_root();
        let mut me = source.pages().me(source.client()).unwrap().clone();
        if asking {
            me = fixtures::asking_me(me);
        }
        let mut page = Self {
            root: temper_client_domain::unlinked(&limits, fixtures::SEED),
            out: Queue::uncapped(),
            goal_watch: None,
            note_reads: 0,
        };
        for event in [
            Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
            Event::Start {
                address: Address::Chats { filter: ChatsFilter::All },
                kept: List::with_capacity(0),
                with: Some(jig_client::With::Transport),
            },
            Event::Client { event: jig_client::Event::Up { up: Up::Served { served: true, versions: None } } },
        ] {
            temper_client_domain::step(&mut page.root, &env, event, &mut page.out);
        }
        for _ in 0_u32..128 {
            let Some(request) = page.out.pop() else { break };
            let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
            let up = match down {
                Down::Open { watch, subject: Watchable::Jig(Subject::Me) } => {
                    Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(me.clone()))) }
                }
                Down::Open { watch, subject: Watchable::Jig(Subject::Settings) } => Up::Watched {
                    watch,
                    event: WatchEvent::Delivered(Delivery::Settings(Box::new(SettingsDocument {
                        newest: 0,
                        settings: List::with_capacity(0),
                        held: List::with_capacity(0),
                        pending: List::with_capacity(0),
                    }))),
                },
                Down::Open { watch, subject: Watchable::Jig(Subject::Tasks { .. }) } => {
                    let mut rows = List::with_capacity(2);
                    let mut chat = line();
                    chat.last_activity = Wall::from_nanos(1);
                    if asking {
                        chat.chip.executor = ChipExecutor::Agent { charter: 37 };
                    }
                    rows.push(chat).unwrap();
                    rows.push(timed(7, 0, None)).unwrap();
                    Up::Watched {
                        watch,
                        event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { rows, more: false }))),
                    }
                }
                Down::Open { watch, subject: Watchable::Jig(Subject::Chips { tasks }) } => {
                    assert_eq!(tasks.as_slice(), [DRAFT_GOAL], "one newest goal, rows only");
                    assert!(page.goal_watch.is_none(), "one chips watch for the rows");
                    page.goal_watch = Some(watch);
                    let mut chip = line().chip;
                    chip.task = DRAFT_GOAL;
                    chip.title = DocumentText::complete(Text::new(b"Keep working".as_slice().into(), TEXT).unwrap());
                    Up::Watched {
                        watch,
                        event: WatchEvent::Delivered(Delivery::Chips(List::from_boxed(Box::new([chip])))),
                    }
                }
                Down::Open { watch, subject: Watchable::Family { record, .. } }
                    if wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch() == &w::Watch::Dismissed =>
                {
                    let mut dismissed = List::with_capacity(4);
                    if linked {
                        // Newer clock, then higher goal, win; another chat isn't watched.
                        for (chat, goal, at) in [(TASK, 40, 1), (TASK, 41, 2), (TASK, DRAFT_GOAL, 2), (99, 43, 3)] {
                            dismissed
                                .push(w::PersonFact::GoalFrom(
                                    w::GoalFrom::new(
                                        &w::CEILINGS,
                                        w::GoalFromParts { project: fixtures::PROJECT, chat, goal, at },
                                    )
                                    .unwrap(),
                                ))
                                .unwrap();
                        }
                    }
                    Up::FamilyDelivered {
                        watch,
                        record: jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::dismissals(
                                &w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).unwrap(),
                            ),
                        },
                    }
                }
                Down::Read { read, what: Readable::Family { record, .. } } => {
                    let reading = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap();
                    let w::Read::Setting(setting) = reading.read() else { continue };
                    let value = if setting.kind() == 7 {
                        let value = w::ProjectWays::new(
                            &w::CEILINGS,
                            w::ProjectWaysParts { drafts: offered, goals: false, changes: false, way: 0 },
                        )
                        .unwrap();
                        let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).unwrap());
                        value.encode(&mut writer).unwrap();
                        Some(writer.finish())
                    } else {
                        None
                    };
                    Up::ReadDone {
                        read,
                        result: jig_client::Readout::Family {
                            family: wire::FORGE,
                            record: Some(wire::setting_answer(
                                &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value }).unwrap(),
                            )),
                        },
                    }
                }
                Down::Read { read, what: Readable::Jig(Read::Notes { scope, description, after }) } => {
                    assert_eq!(scope, NoteScope::Tasks { project: fixtures::PROJECT });
                    assert_eq!((description, after), (None, None));
                    page.note_reads = page.note_reads.checked_add(1).unwrap();
                    let mut rows = List::with_capacity(3);
                    for task in [TASK, 8, 99] {
                        rows.push(Note {
                            number: 1,
                            revision: Revision { value: 1 },
                            scope: NoteScope::Task { project: fixtures::PROJECT, task },
                            description: DocumentText::complete(
                                Text::new(b"Away mode".as_slice().into(), TEXT).unwrap(),
                            ),
                            body: DocumentText::complete(Text::new(Box::default(), TEXT).unwrap()),
                            writer: NoteAuthor::Task { task, attempt: 1 },
                            tasks: List::with_capacity(0),
                            changed: None,
                            acts: List::with_capacity(0),
                            cut: false,
                        })
                        .unwrap();
                    }
                    Up::ReadDone {
                        read,
                        result: jig_client::Readout::Jig(ReadResult::Value(ReadValue::Notes(Box::new(NotePage {
                            rows,
                            before: None,
                            after: None,
                            cut: false,
                        })))),
                    }
                }
                Down::Read { read, what: Readable::Jig(Read::Tasks { .. }) } => Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: EndedGroup::Rows {
                            rows: List::from_boxed(Box::new([timed(8, 0, Some(0))])),
                            after: None,
                            cut: false,
                        },
                    })))),
                },
                Down::Save { .. }
                | Down::Erase { .. }
                | Down::Send { .. }
                | Down::Open { .. }
                | Down::Continue { .. }
                | Down::Close { .. }
                | Down::Read { .. } => continue,
            };
            temper_client_domain::step(
                &mut page.root,
                &env,
                Event::Client { event: jig_client::Event::Up { up } },
                &mut page.out,
            );
        }
        assert!(page.out.is_empty(), "fixture settled");
        page
    }
}


#[test]
fn drafts_keeps_rows_with_a_document() {
    let mut page = DraftsPage::new(true, false, false);
    assert_eq!(page.note_reads, 1);
    filter(&mut page.root, ChatsFilter::Drafts);
    for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        assert_eq!(numbers(view.tree()).as_ref(), [TASK, 8]);
        let mut filters = List::with_capacity(4);
        for node in view.tree().nodes() {
            if node.role == Role::Button
                && let Some(name) = node.name.as_deref()
                && [b"All".as_slice(), b"Live", b"Drafts", b"Closed"].contains(&name)
            {
                filters.push(name).unwrap();
            }
        }
        assert_eq!(filters.as_slice(), [b"All".as_slice(), b"Live", b"Drafts", b"Closed"]);
    }
}

#[test]
fn drafts_isnt_offered_with_no_tracked_template() {
    let mut page = DraftsPage::new(false, false, false);
    filter(&mut page.root, ChatsFilter::Drafts);
    let view = fixtures::page_view(&page.root);
    assert_eq!(numbers(view.tree()).as_ref(), [TASK, 7, 8]);
    assert!(fixtures::find(view.tree(), Role::Button, b"Drafts").is_none());
    let all = fixtures::find(view.tree(), Role::Button, b"All").unwrap();
    assert!(
        view.tree().nodes()[index_of(view.tree().nodes(), all)].states.as_slice().contains(&skein_ui::State::Pressed)
    );
}

#[test]
fn a_row_says_what_it_writes() {
    let page = DraftsPage::new(true, false, false);
    for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"writes away-mode.md").is_some());
    }
}

#[test]
fn a_row_says_which_goal_it_started() {
    let page = DraftsPage::new(true, false, true);
    for mut view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        let goal = fixtures::find(view.tree(), Role::Link, b"Keep working").unwrap();
        assert_eq!(crate::href(&mut view, goal).as_deref(), Some(b"/tasks/42".as_slice()));
        assert_eq!(
            crate::act(&mut view, goal, skein_ui::Act::Press),
            Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                address: temper_client_domain::Address::Task { task: DRAFT_GOAL }
            }))
        );
        let mut print = skein_ui_text::Print::new(&fixtures::tree_limits()).unwrap();
        skein_ui_text::render(view.tree(), b"temper", None, &mut print);
        assert!(skein_lib::bytes::find(print.text.as_slice(), b"started goal").is_some());
        assert!(skein_lib::bytes::find(print.text.as_slice(), b"Keep working").is_some());
    }
}

#[test]
fn the_lede_grows_with_the_offer() {
    for offered in [false, true] {
        let page = DraftsPage::new(offered, false, false);
        for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
            assert_eq!(fixtures::find(view.tree(), Role::Paragraph, DRAFT_LEDE).is_some(), offered);
        }
    }
}

#[test]
fn an_asking_chat_keeps_its_tag() {
    let page = DraftsPage::new(true, true, false);
    for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        let mut found = false;
        for (index, node) in view.tree().nodes().iter().enumerate() {
            if node.styles.as_slice().contains(&crate::look::CHATS_TOP) {
                let text = fixtures::node_text(view.tree(), index);
                if skein_lib::bytes::find(&text, b"T9").is_some() {
                    assert!(
                        skein_lib::bytes::find(&text, "T9 · Ask temper · writes away-mode.md".as_bytes()).is_some()
                    );
                    found = true;
                }
            }
        }
        assert!(found);
    }
}


#[test]
fn behind_with_unanswered_reads_chats_claims_no_empty() {
    for selected in [ChatsFilter::All, ChatsFilter::Live, ChatsFilter::Closed, ChatsFilter::Drafts] {
        let mut page = fixtures::ReadPage::new(temper_client_domain::Address::Chats { filter: selected });
        page.drop_inbox();
        for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
            assert!(fixtures::find(view.tree(), Role::Status, b"Behind").is_some());
            assert!(fixtures::find(view.tree(), Role::Paragraph, b"None.").is_none());
        }
    }
}

#[test]
fn a_lost_closed_read_keeps_the_closed_rows() {
    let mut page = fixtures::ReadPage::new(temper_client_domain::Address::Chats { filter: ChatsFilter::All });
    let ended = timed(8, 0, Some(0));
    page.live_chats(&[]);
    page.closed(core::slice::from_ref(&ended));
    page.live_chats(&[]);
    let read = page.reads.last().unwrap().0;
    page.answer(read, ReadResult::None(ReadNone::Unreachable));
    for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        assert!(skein_lib::bytes::find(&fixtures::main_text(view.tree()), ended.chip.title.bytes()).is_some());
    }
}

#[test]
fn drafts_claims_no_empty_until_its_notes_answer() {
    let mut page = fixtures::ReadPage::new(temper_client_domain::Address::Chats { filter: ChatsFilter::Drafts });
    page.live_chats(&[]);
    page.closed(&[]);
    for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        assert!(fixtures::find(view.tree(), Role::Paragraph, b"None.").is_none());
    }
    let mut read = None;
    for (token, what) in &page.reads {
        if let Read::Notes { .. } = what {
            read = Some(*token);
        }
    }
    let read = read.expect("notes read");
    page.answer(
        read,
        ReadResult::Value(ReadValue::Notes(Box::new(NotePage {
            rows: List::with_capacity(0),
            before: None,
            after: None,
            cut: false,
        }))),
    );
    for view in [fixtures::page_view(&page.root), fixtures::browser_page_view(&page.root)] {
        assert!(fixtures::find(view.tree(), Role::Paragraph, b"None.").is_some());
    }
}
