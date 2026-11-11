//! The project's frame (temper/docs/design/client/view.md, 1 and 3).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests compare semantic subtrees and terminal text"
)]
use super::fixtures as f;
use crate::look;
use jig_client::contract as c;
use skein_lib::{Env, List, Queue, Time, Token};
use skein_ui::{Act, Node, Role, State, Style, Tree};
use temper_client_domain::{self as d, Address, ChatsFilter, Event, GoalsFilter, pages};

fn styled(tree: &Tree, style: Style) -> &[Node] {
    let at = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&style)).expect("styled node");
    let end = at.checked_add(usize::try_from(tree.nodes()[at].size).unwrap()).unwrap();
    &tree.nodes()[at..end]
}
fn children(nodes: &[Node]) -> Vec<&Node> {
    let mut out = Vec::new();
    let mut at = 1;
    while at < nodes.len() {
        out.push(&nodes[at]);
        at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    }
    out
}

#[test]
fn the_new_box_draws_the_parts_items_descriptions_and_addresses() {
    use d::forge::ways::Ways;
    for (drafts, ask) in [(false, false), (false, true), (true, false), (true, true)] {
        let (mut root, watch, ..) = f::root_at_with_values(
            Address::Home { project: f::PROJECT },
            &f::domain_limits(),
            b"Shop",
            None,
            None,
            None,
            Some(&[]),
            Some(Ways { drafts, ..Ways::STOCK }),
            false,
            true,
        );
        if ask {
            let me = f::asking_me(root.pages().me(root.client()).unwrap().clone());
            f::deliver_me(&mut root, watch, me);
        }
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let entry = styled(view.tree(), look::RAIL_NEW);
            assert_eq!(entry[0].role, Role::Slot);
            let parts = children(entry);
            assert_eq!(parts.len(), 1);
            assert_eq!(parts[0].role, if drafts { Role::Menu } else { Role::Link });
            assert_eq!(
                parts[0].name.as_deref(),
                Some(if drafts { b"New draft or change".as_slice() } else { b"New change" })
            );
            if drafts {
                let items: Vec<_> = entry.iter().filter(|n| n.role == Role::MenuItem).collect();
                assert_eq!(items.len(), if ask { 3 } else { 2 });
                let expected = [
                    (
                        b"Draft".as_slice(),
                        b"A design session that ends in a goal, best had in the terminal.".as_slice(),
                        Address::NewChat { draft: true },
                    ),
                    (b"Change".as_slice(), b"Ask for one change.".as_slice(), Address::NewChat { draft: false }),
                    (
                        b"Ask temper".as_slice(),
                        b"Ask about this project, or change how its work runs.".as_slice(),
                        Address::Ask { project: f::PROJECT },
                    ),
                ];
                let rows: Vec<_> = items
                    .into_iter()
                    .zip(expected)
                    .map(|(n, (name, line, address))| {
                        assert_eq!(n.name.as_deref(), Some(name));
                        assert_eq!(n.description.as_deref(), Some(line));
                        (n.id, address)
                    })
                    .collect();
                for (id, address) in rows {
                    assert_eq!(
                        crate::act(&mut view, id, Act::Press),
                        Some(d::Action::Pages(pages::Act::Go { address }))
                    );
                }
            } else {
                let id = parts[0].id;
                assert_eq!(crate::href(&mut view, id).as_deref(), Some(b"/chats/new".as_slice()));
                assert_eq!(
                    crate::act(&mut view, id, Act::Press),
                    Some(d::Action::Pages(pages::Act::Go { address: Address::NewChat { draft: false } }))
                );
            }
        }
    }
}

fn project_rows(count: u32, repository: &[u8]) -> List<temper_client_protocol_wire::v1::ForgeRow> {
    use temper_client_protocol_wire::v1 as w;
    let mut row = super::connect_forge::row(
        3,
        w::Kind::Forgejo,
        w::Mode::Owned,
        super::connect_forge::connected(),
        w::Offer::Offered,
    )
    .into_parts();
    row.repositories = List::with_capacity(count);
    for _ in 0..count {
        row.repositories
            .push(
                w::Adoption::new(
                    &w::CEILINGS,
                    w::AdoptionParts { owner: b"work".as_slice().into(), name: repository.into(), project: f::PROJECT },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut rows = List::with_capacity(1);
    rows.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    rows
}

fn switcher_rows(tree: &Tree) -> Vec<&Node> {
    let mut rows = Vec::new();
    for node in children(styled(tree, look::PROJECTS_MENU)) {
        if node.styles.as_slice().contains(&look::PROJECTS_ROW) {
            let at = tree.nodes().iter().position(|n| n.id == node.id).unwrap();
            rows.push(&tree.nodes()[at.checked_add(1).unwrap()]);
        } else if node.role == Role::Button {
            rows.push(node);
        }
    }
    rows
}

#[test]
fn switcher_rows_keep_repository_words_and_counts_in_both_shells() {
    let address = Address::Home { project: f::PROJECT };
    let mut ended = f::forge_root_watch(address.clone(), project_rows(1, b"hello"));
    f::root_event(
        &mut ended.0,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch: ended.1, end: jig_client::contract::WatchEnd::NotAllowed },
            },
        },
    );
    for (root, repository, more, terminal) in vec![
        (ended.0, None, None, "Shop"),
        (
            f::forge_root(address.clone(), project_rows(0, b"hello")),
            Some("no repository"),
            None,
            "Shop · no repository",
        ),
        (f::forge_root(address.clone(), project_rows(1, b"hello")), Some("work/hello"), None, "Shop · work/hello"),
        (
            f::forge_root(address.clone(), project_rows(3, b"hello")),
            Some("work/hello"),
            Some(" +2"),
            "Shop · work/hello +2",
        ),
        (f::playground_root(address, f::PROJECT, true), Some("acme/shop"), None, "Shop · acme/shop"),
    ]
    .into_boxed_slice()
    {
        for browser in [false, true] {
            let mut view = if browser { f::browser_page_view(&root) } else { f::page_view(&root) };
            let rows: Vec<_> =
                switcher_rows(view.tree()).into_iter().filter(|n| n.name.as_deref() != Some(b"All projects")).collect();
            assert_eq!(rows.len(), 2);
            assert!(rows[0].states.as_slice().contains(&State::Pressed));
            assert!(!rows[1].states.as_slice().contains(&State::Pressed));
            let id = rows[0].id;
            if browser {
                assert_eq!(rows[0].name.as_deref(), Some(b"Shop".as_slice()));
                let nodes = view.tree().nodes();
                let at = nodes.iter().position(|n| n.id == rows[0].id).unwrap() - 1;
                let row = &nodes[at..at + usize::try_from(nodes[at].size).unwrap()];
                let parts = children(row);
                assert_eq!(parts.len(), if repository.is_some() { 2 } else { 1 });
                assert_eq!(parts[0].role, Role::Button);
                if let Some(repository) = repository {
                    assert_eq!(parts[1].role, Role::Text);
                    assert_eq!(parts[1].styles.as_slice(), [look::PROJECTS_REPO]);
                    let at = nodes.iter().position(|n| n.id == parts[1].id).unwrap();
                    let small = &nodes[at..at + usize::try_from(nodes[at].size).unwrap()];
                    let labels = children(small);
                    assert_eq!(labels[0].text.get(0).unwrap().text.as_ref(), repository.as_bytes());
                    assert_eq!(labels.len(), if more.is_some() { 2 } else { 1 });
                    if let Some(more) = more {
                        assert_eq!(labels[1].styles.as_slice(), [look::PROJECTS_MORE]);
                        assert_eq!(labels[1].text.get(0).unwrap().text.as_ref(), more.as_bytes());
                    }
                }
            } else {
                assert_eq!(rows[0].name.as_deref(), Some(b"Shop".as_slice()));
                let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
                skein_ui_text::render(view.tree(), b"temper", Some(crate::frame::link_region()), &mut print);
                let text = String::from_utf8_lossy(print.text.as_slice());
                assert!(text.lines().any(|line| line.trim().ends_with(&format!("{terminal} (pressed)"))), "{text}");
            }
            assert_eq!(
                crate::act(&mut view, id, Act::Press),
                Some(d::Action::Pages(pages::Act::Project { project: f::PROJECT }))
            );
        }
    }
}

#[test]
fn the_panel_puts_all_projects_first_and_links_after_the_forms() {
    for (projects, across, renaming, filtered) in [
        (1, false, false, false),
        (2, false, false, false),
        (2, true, false, false),
        (2, false, true, false),
        (2, false, false, true),
    ] {
        let (mut root, watch, _) = f::root_at(Address::Home { project: f::PROJECT }, &f::domain_limits());
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.projects.get_mut(0).unwrap().role = jig_client::contract::HeldRole::Owner;
        if projects == 1 {
            me.projects = List::from_boxed(Box::new([me.projects.get(0).unwrap().clone()]));
        }
        f::deliver_me(&mut root, watch, me);
        let filter =
            d::address::Filter { project: Some(f::PROJECT), kind: Some(d::address::Kind::Question), goal: Some(9) };
        if across || filtered {
            f::root_event(
                &mut root,
                Event::Went {
                    place: d::Place::Address(Address::Inbox {
                        filter: d::address::Filter { project: if across { None } else { Some(f::PROJECT) }, ..filter },
                    }),
                },
            );
        }
        if renaming {
            f::root_event(
                &mut root,
                Event::Act { action: d::Action::Pages(pages::Act::Switcher(pages::switcher::Act::Rename)) },
            );
        }
        for (browser, mut view) in [(false, f::page_view(&root)), (true, f::browser_page_view(&root))] {
            assert_eq!(
                styled(view.tree(), look::PROJECTS)[0].name.as_deref(),
                Some(if across { b"All projects".as_slice() } else { b"Shop" })
            );
            let panel = children(styled(view.tree(), look::PROJECTS_MENU));
            assert_eq!(panel.len(), projects + usize::from(projects >= 2) + 1);
            let rows = switcher_rows(view.tree());
            assert_eq!(
                rows.iter().filter(|n| n.name.as_deref() == Some(b"All projects")).count(),
                usize::from(projects >= 2)
            );
            let all_id = if projects >= 2 { Some(rows[0].id) } else { None };
            if projects >= 2 {
                assert_eq!(rows[0].name.as_deref(), Some(b"All projects".as_slice()));
                assert_eq!(rows[0].states.as_slice().contains(&State::Pressed), across);
                if browser {
                    let row = styled(view.tree(), look::PROJECTS_ROW);
                    let small = children(row)[1];
                    assert_eq!(small.styles.as_slice(), [look::PROJECTS_REPO]);
                    assert_eq!(small.name.as_deref(), Some(b"2 projects".as_slice()));
                } else {
                    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
                    skein_ui_text::render(view.tree(), b"temper", Some(crate::frame::link_region()), &mut print);
                    assert!(String::from_utf8_lossy(print.text.as_slice()).contains("All projects · 2 projects"));
                }
                if !renaming {
                    assert_eq!(rows[1].states.as_slice().contains(&State::Pressed), !across);
                }
            }
            if renaming {
                assert_eq!(panel[1].name.as_deref(), Some(b"Rename Shop".as_slice()));
            }
            assert_eq!(panel.last().unwrap().styles.as_slice(), [look::PROJECTS_ACTS]);
            let acts = children(styled(view.tree(), look::PROJECTS_ACTS));
            assert_eq!(
                acts.iter().map(|n| n.role).collect::<Vec<_>>(),
                if renaming {
                    vec![Role::Form, Role::Link, Role::Link]
                } else {
                    vec![Role::Button, Role::Form, Role::Link, Role::Link]
                }
            );
            if !renaming {
                assert_eq!(acts[0].name.as_deref(), Some(b"Rename Shop".as_slice()));
            }
            let new = acts[acts.len() - 3];
            let how = acts[acts.len() - 2];
            let deployment = acts.last().unwrap();
            assert_eq!(new.name.as_deref(), Some(b"New project".as_slice()));
            assert_eq!(how.name.as_deref(), Some(b"How work runs for Shop".as_slice()));
            assert_eq!(deployment.name.as_deref(), Some(b"Deployment".as_slice()));
            let (how, deployment) = (how.id, deployment.id);
            assert_eq!(crate::href(&mut view, how).as_deref(), Some(b"/projects/1/settings".as_slice()));
            assert_eq!(crate::href(&mut view, deployment).as_deref(), Some(b"/accounts".as_slice()));
            if let Some(id) = all_id {
                let target = if across || filtered {
                    Address::Inbox { filter: d::address::Filter { project: None, ..filter } }
                } else {
                    Address::AllHome
                };
                assert_eq!(
                    crate::act(&mut view, id, Act::Press),
                    Some(d::Action::Pages(pages::Act::Go { address: target }))
                );
            }
        }
    }
}

#[test]
fn members_keep_new_and_settings_without_rename_in_both_shells() {
    let root = f::root_in_project(None);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let acts = children(styled(view.tree(), look::PROJECTS_ACTS));
        assert_eq!(acts.iter().map(|n| n.role).collect::<Vec<_>>(), [Role::Form, Role::Link, Role::Link]);
        assert_eq!(acts[0].name.as_deref(), Some(b"New project".as_slice()));
        assert_eq!(acts[1].name.as_deref(), Some(b"How work runs for Shop".as_slice()));
        assert!(f::find(view.tree(), Role::Button, b"Rename Shop").is_none());
    }
}

#[test]
fn the_frame_requires_me_and_a_project_and_yields_to_welcome_and_sign_in() {
    let root = f::root_in_project(None);
    assert_eq!(crate::frame::rail_project(&root, &f::words(), &crate::sign_in::Ways::NONE), Some(f::PROJECT));
    for root in f::pair(f::signed_out(), f::out_of_date()) {
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(view.tree().nodes().iter().any(|n| n.role == Role::Banner));
            assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::RAIL)));
        }
    }
    let view = f::built();
    assert!(view.tree().nodes().iter().any(|n| n.role == Role::Banner));
    let root = f::welcome_root(d::Step::Ready, false);
    assert_eq!(crate::frame::rail_project(&root, &f::words(), &crate::sign_in::Ways::NONE), None);
    assert!(f::page_view(&root).tree().nodes().iter().any(|n| n.role == Role::Banner));
    let root = f::no_project_root();
    assert!(root.pages().me(root.client()).is_some());
    assert_eq!(root.pages().project(), None);
    assert!(f::page_view(&root).tree().nodes().iter().any(|n| n.role == Role::Banner));
    let root = f::signed_out();
    let view = f::sign_in_view(
        &root,
        crate::words::Shell::Browser,
        crate::sign_in::Ways::parse("google", b"https://temper.example"),
    );
    assert!(view.tree().nodes().iter().any(|n| n.role == Role::Banner));
    let mut root = f::root_in_project(None);
    f::root_event(&mut root, Event::Went { place: d::Place::Authorize { query: Box::from(b"state=test".as_slice()) } });
    assert!(root.authorization().is_some());
    assert_eq!(crate::frame::rail_project(&root, &f::browser_words(), &crate::sign_in::Ways::NONE), None);
    assert!(f::browser_page_view(&root).tree().nodes().iter().any(|n| n.role == Role::Banner));
}

// The rail's fixtures use one goal template and one row in its fact watch.
const GOAL_LABEL: u32 = 2;
const GOAL_TASK: u64 = 9;
const GOAL_PRIORITY: u32 = 1;
const TEXT_BYTES: u32 = 64;

pub(super) fn with_goal_fact(root: &mut d::Domain, me_watch: Token, delivered: Option<bool>) {
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let mut flags = super::chats::line().flags;
    flags.conversation = false;
    flags.listed = true;
    let goal = c::Template {
        resources: List::uncapped(),
        number: jig_client::shapes::GOAL,
        name: f::task::document_words(b"goal"),
        executor: c::TemplateExecutor::Charter(1),
        contract: c::TaskContractDocument::Report { words: TEXT_BYTES },
        shallower: 1,
        budget: c::TemplateBudget::AsAllowed { cap: None },
        deadline: c::TemplateDeadline::None,
        tracked: Some(GOAL_LABEL),
        flags,
        label: GOAL_LABEL,
        grace: None,
        llm: None,
    };
    let mut asking = goal.clone();
    asking.number = 5;
    asking.executor = c::TemplateExecutor::Charter(11);
    me.projects.get_mut(0).unwrap().templates = List::from_boxed(Box::new([goal, asking]));
    let mut out = Queue::uncapped();
    d::step(
        root,
        &Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut out,
    );
    if let Some(has_goal) = delivered {
        let watch = out
            .iter()
            .find_map(|request| match request {
                d::Request::Unlinked {
                    request:
                        jig_client::UnlinkedRequest::Down {
                            down:
                                jig_client::Down::Open {
                                    watch,
                                    subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, window: 1 }),
                                },
                        },
                } if query.tracked == Some(true) => Some(*watch),
                d::Request::Client { .. }
                | d::Request::Unlinked { .. }
                | d::Request::Show { .. }
                | d::Request::Leave { .. }
                | d::Request::SignOut
                | d::Request::Dropped { .. }
                | d::Request::Screen { .. }
                | d::Request::KeepLook { .. }
                | d::Request::KeepSignIn { .. }
                | d::Request::Tab { .. }
                | d::Request::Copy { .. }
                | d::Request::Push(_)
            | d::Request::Fact { .. } => None,
            })
            .expect("goal fact watch");
        let mut rows = List::uncapped();
        if has_goal {
            let mut row = super::chats::line();
            row.tracked = Some(GOAL_PRIORITY);
            row.flags.conversation = false;
            rows.push(row).unwrap();
        }
        deliver_document(root, watch, c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })));
    }
}
fn deliver_document(root: &mut d::Domain, watch: Token, document: c::Delivery) {
    f::root_event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(document) },
            },
        },
    );
}
// Deliver the rail's project read, preserving the address the row checks.
pub(super) fn with_people(root: &mut d::Domain, people: u32) {
    let address = root.pages().address().clone();
    f::root_event(root, Event::Went { place: d::Place::Address(Address::Forges) });
    let mut out = Queue::uncapped();
    d::step(
        root,
        &Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        Event::Went { place: d::Place::Address(Address::Settings { project: f::PROJECT }) },
        &mut out,
    );
    let read = out
        .iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Project { .. }) },
                    },
            } = request
            {
                Some(*read)
            } else {
                None
            }
        })
        .expect("rail's project read");
    let limits = f::domain_limits();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let mut document =
        jig_client_protocol_link::translate::documents::project(jig_client_test_records::owner_project_record(), &link)
            .unwrap();
    document.number = f::PROJECT;
    let reader = &root.pages().me(root.client()).unwrap().party;
    let member = c::ProjectMember {
        party: reader.number,
        kind: c::PartyKind::Person,
        name: reader.name.clone(),
        login: reader.login.clone(),
        role: c::HeldRole::Owner,
    };
    document.members = List::with_capacity(people);
    for number in 0..people {
        document.members.push(c::ProjectMember { party: u64::from(number), ..member.clone() }).unwrap();
    }
    f::root_event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Project(Box::new(document)))),
                },
            },
        },
    );
    f::root_event(root, Event::Went { place: d::Place::Address(address) });
}
fn current_names(tree: &Tree, style: Style) -> Vec<&[u8]> {
    children(styled(tree, style))
        .into_iter()
        .filter(|node| node.states.as_slice().contains(&State::Current))
        .map(|node| node.name.as_deref().unwrap())
        .collect()
}

#[test]
fn a_chat_is_never_goals() {
    for (asking, expected) in [(false, b"Work".as_slice()), (true, b"Ask temper".as_slice())] {
        let (mut root, me, chat) = f::root_at(Address::Chat { task: GOAL_TASK }, &f::domain_limits());
        with_goal_fact(&mut root, me, Some(true));
        let asking_me = f::asking_me(root.pages().me(root.client()).unwrap().clone());
        f::deliver_me(&mut root, me, asking_me);
        let mut document = f::task::delegate();
        document.project = f::PROJECT;
        document.goal = Some(GOAL_TASK);
        document.priority = Some(GOAL_PRIORITY);
        document.chip.executor = c::ChipExecutor::Agent { charter: if asking { 37 } else { 1 } };
        deliver_document(&mut root, chat.unwrap(), c::Delivery::Task(Box::new(document)));
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let nav = current_names(view.tree(), look::RAIL_NAV);
            let foot = current_names(view.tree(), look::RAIL_FOOT);
            assert_eq!(nav.into_iter().chain(foot).collect::<Vec<_>>(), [expected]);
            if view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::TABBAR)) {
                let tabs = styled(view.tree(), look::TABBAR);
                assert_eq!(tabs[0].styles.as_slice().contains(&look::TABBAR_ASK), asking);
                assert_eq!(current_names(view.tree(), look::TABBAR), [expected]);
                let ask = children(tabs).into_iter().find(|n| n.name.as_deref() == Some(b"Ask temper")).unwrap();
                assert_eq!(f::nodes_text(styled(view.tree(), look::TABBAR)).as_ref(), b"HomeNeedsWorkAsk");
                assert_eq!(ask.states.as_slice().contains(&State::Current), asking);
            }
        }
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks both navigation surfaces across address groups and counts")]
fn rail_and_tabbar_keep_the_order_addresses_current_states_and_count() {
    use d::WorkFilter;
    for (address, current) in [
        (Address::Home { project: f::PROJECT }, "Home"),
        (Address::Inbox { filter: d::address::Filter::ALL }, "Needs you"),
        (Address::Work { project: f::PROJECT, filter: WorkFilter::All }, "Work"),
        (Address::Goals { project: f::PROJECT, filter: GoalsFilter::All }, "Work"),
        (Address::Changes { project: f::PROJECT }, "Work"),
        (Address::Chats { filter: ChatsFilter::All }, "Work"),
        (Address::NewChat { draft: false }, "Work"),
        (Address::Chat { task: 9 }, "Work"),
        (Address::Task { task: 9 }, "Work"),
        (Address::Team { project: f::PROJECT }, "Team"),
        (Address::Settings { project: f::PROJECT }, "How work runs"),
        (Address::Role { project: f::PROJECT, charter: 2 }, "How work runs"),
        (Address::Rule { project: f::PROJECT, kind: 2, number: 1 }, "How work runs"),
        (Address::Repositories { project: f::PROJECT }, "How work runs"),
        (Address::Adopt { project: f::PROJECT }, "How work runs"),
        (Address::Ask { project: f::PROJECT }, "Ask temper"),
        (Address::Accounts, "Deployment"),
        (Address::Forges, "Deployment"),
        (Address::Models { project: f::PROJECT }, "Deployment"),
        (Address::Builders { project: f::PROJECT }, "Deployment"),
        (Address::Members, "Deployment"),
        (Address::Limits, "Deployment"),
        (Address::ConnectModel { way: None }, "Deployment"),
        (Address::ConnectForge { forge: None }, "Deployment"),
    ] {
        let (mut root, watch, _, _) = f::root_at_with_needs(address.clone(), &f::domain_limits());
        let me = f::asking_me(root.pages().me(root.client()).unwrap().clone());
        f::deliver_me(&mut root, watch, me);
        let team = current == "Team";
        let (home, needs) = if address == (Address::Inbox { filter: d::address::Filter::ALL }) {
            (b"/home".as_slice(), b"/".as_slice())
        } else {
            (b"/projects/1".as_slice(), b"/?project=1".as_slice())
        };
        if team {
            with_people(&mut root, 2);
        }
        for (browser, mut view) in [(false, f::page_view(&root)), (true, f::browser_page_view(&root))] {
            assert_eq!(skein_ui::outline::check(view.tree()), None, "{address:?}");
            assert!(!view.tree().nodes().iter().any(|n| n.role == Role::Banner));
            let rail = styled(view.tree(), look::RAIL);
            assert_eq!(rail[0].role, Role::Navigation);
            assert_eq!(rail[0].name.as_deref(), Some(b"Shop".as_slice()));
            assert_eq!(children(rail).iter().any(|n| n.styles.as_slice().contains(&look::AWAY_CHIP)), browser);
            let parts: Vec<_> = children(rail)
                .into_iter()
                .filter(|n| {
                    !n.styles.as_slice().contains(&look::RAIL_UNFOLD) && !n.styles.as_slice().contains(&look::AWAY_CHIP)
                })
                .collect();
            assert_eq!(
                parts.iter().map(|n| n.role).collect::<Vec<_>>(),
                if browser {
                    vec![Role::Link, Role::Disclosure, Role::Slot, Role::Slot, Role::Slot, Role::Button]
                } else {
                    vec![Role::Link, Role::Disclosure, Role::Slot, Role::Slot, Role::Slot]
                }
            );
            assert_eq!(parts[0].name.as_deref(), Some(b"temper home".as_slice()));
            assert_eq!(parts[1].styles.as_slice(), [look::PROJECTS]);
            assert_eq!(parts[2].styles.as_slice(), [look::RAIL_NAV]);
            assert_eq!(parts[3].styles.as_slice(), [look::RAIL_NEW]);
            assert_eq!(parts[4].styles.as_slice(), [look::RAIL_FOOT]);
            let brand = parts[0].id;
            assert_eq!(crate::href(&mut view, brand).as_deref(), Some(home));
            let nav = children(styled(view.tree(), look::RAIL_NAV));
            let mut names = vec![Some(b"Home".as_slice()), Some(b"Needs you"), Some(b"Work")];
            if team {
                names.push(Some(b"Team"));
            }
            names.extend([Some(b"Ask temper".as_slice()), None, Some(b"How work runs")]);
            assert_eq!(nav.iter().map(|n| n.name.as_deref()).collect::<Vec<_>>(), names);
            let sep = nav[4 + usize::from(team)];
            assert_eq!(sep.role, Role::Slot);
            assert_eq!(sep.styles.as_slice(), [look::RAIL_SEP]);
            let expected = if current == "Deployment" { vec![] } else { vec![current.as_bytes()] };
            assert_eq!(current_names(view.tree(), look::RAIL_NAV), expected, "{address:?}");
            assert_eq!(
                current_names(view.tree(), look::RAIL_FOOT),
                if current == "Deployment" { vec![b"Deployment".as_slice()] } else { vec![] },
                "{address:?}"
            );
            let rows: Vec<_> = nav.into_iter().filter(|n| n.role == Role::Link).map(|n| n.id).collect();
            let mut hrefs = vec![home, needs, b"/projects/1/work"];
            if team {
                hrefs.push(b"/projects/1/team");
            }
            hrefs.extend([b"/projects/1/ask".as_slice(), b"/projects/1/settings"]);
            for (id, href) in rows.into_iter().zip(hrefs) {
                assert_eq!(crate::href(&mut view, id).as_deref(), Some(href));
            }
            if view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::TABBAR)) {
                assert!(browser);
                assert_eq!(
                    styled(view.tree(), look::TABBAR)[0].styles.as_slice().contains(&look::TABBAR_ASK),
                    current == "Ask temper"
                );
                let links = children(styled(view.tree(), look::TABBAR));
                assert_eq!(f::nodes_text(styled(view.tree(), look::TABBAR)).as_ref(), b"HomeNeedsWorkAsk");
                let ids: Vec<_> = links.iter().map(|n| n.id).collect();
                for (id, href) in ids.into_iter().zip([home, needs, b"/projects/1/work", b"/projects/1/ask"]) {
                    assert_eq!(crate::href(&mut view, id).as_deref(), Some(href));
                }
                assert_eq!(
                    children(styled(view.tree(), look::TABBAR))
                        .iter()
                        .map(|n| n.name.as_deref().unwrap())
                        .collect::<Vec<_>>(),
                    [b"Home".as_slice(), b"Needs", b"Work", b"Ask temper"]
                );
                assert_eq!(
                    current_names(view.tree(), look::TABBAR),
                    if ["Home", "Needs you", "Work", "Ask temper"].contains(&current) {
                        vec![if current == "Needs you" { b"Needs".as_slice() } else { current.as_bytes() }]
                    } else {
                        vec![]
                    }
                );
            }
            if browser {
                let switch = children(styled(view.tree(), look::RAIL)).last().unwrap().id;
                assert_eq!(
                    crate::act(&mut view, switch, Act::Press),
                    Some(d::Action::Pages(pages::Act::Terminal(pages::terminal::Act::Night)))
                );
            }
        }
    }
    let (mut root, _, _) = f::root_at(Address::Home { project: f::PROJECT }, &f::domain_limits());
    for night in [false, true] {
        let mut view = f::browser_page_view(&root);
        let parts = children(styled(view.tree(), look::RAIL));
        let switch = parts.last().unwrap();
        assert_eq!(switch.styles.as_slice(), [look::NIGHT_SWITCH]);
        assert_eq!(switch.name.as_deref(), Some(b"Night mode".as_slice()));
        assert_eq!(switch.states.as_slice().contains(&State::Pressed), night);
        let id = switch.id;
        let action = crate::act(&mut view, id, Act::Press).unwrap();
        f::root_event(&mut root, Event::Act { action });
    }
    assert!(!root.pages().look().night);
    let (mut root, watch, _, inbox) =
        f::root_at_with_needs(Address::Inbox { filter: d::address::Filter::ALL }, &f::domain_limits());
    for (inbox_count, cut, approval) in [(0, false, false), (3, false, false), (3, true, false), (2, false, true)] {
        let count = inbox_count + u32::from(approval);
        let rows = [super::needs::approval_row()];
        super::needs::approvals(&mut root, if approval { &rows } else { &[] }, 0);
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.needs_total = count;
        f::deliver_me(&mut root, watch, me);
        deliver_count(&mut root, inbox, inbox_count, cut);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            for style in [look::RAIL_NAV, look::TABBAR] {
                if !view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&style)) {
                    continue;
                }
                let nodes = styled(view.tree(), style);
                let name = match (style == look::TABBAR, count == 0, cut) {
                    (true, true, _) => "Needs",
                    (true, false, false) => "Needs 3",
                    (true, false, true) => "Needs 3+",
                    (false, true, _) => "Needs you",
                    (false, false, false) => "Needs you 3",
                    (false, false, true) => "Needs you 3+",
                };
                assert_eq!(current_names(view.tree(), style), [name.as_bytes()]);
                let badges: Vec<_> = nodes.iter().filter(|n| n.styles.as_slice().contains(&look::NAV_COUNT)).collect();
                assert_eq!(badges.len(), usize::from(count > 0));
                if count > 0 {
                    assert_eq!(badges[0].text.get(0).unwrap().text.as_ref(), if cut { b"3+".as_slice() } else { b"3" });
                }
            }
        }
    }
    for (address, current, asking) in [
        (Address::Home { project: f::PROJECT }, b"Home".as_slice(), false),
        (Address::Ask { project: f::PROJECT }, b"Ask temper".as_slice(), true),
    ] {
        let (mut root, me, _, settings, ..) =
            f::home_root(&f::domain_limits(), List::with_capacity(0), Some(&[]), false);
        let me_document = f::asking_me(root.pages().me(root.client()).unwrap().clone());
        let mut out = Queue::uncapped();
        let env = Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() };
        d::step(
            &mut root,
            &env,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: settings,
                        event: c::WatchEvent::Delivered(c::Delivery::Settings(Box::new(c::SettingsDocument {
                            newest: 0,
                            settings: List::with_capacity(0),
                            held: List::with_capacity(0),
                            pending: List::with_capacity(0),
                        }))),
                    },
                },
            },
            &mut out,
        );
        d::step(
            &mut root,
            &env,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: me,
                        event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me_document))),
                    },
                },
            },
            &mut out,
        );
        d::step(&mut root, &env, Event::Went { place: d::Place::Address(address.clone()) }, &mut out);
        while let Some(request) = out.pop() {
            let d::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else {
                continue;
            };
            let up = match down {
                jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { .. }) } => {
                    jig_client::Up::Watched {
                        watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                            rows: List::with_capacity(0),
                            more: false,
                        }))),
                    }
                }
                jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { .. }) } => {
                    jig_client::Up::ReadDone {
                        read,
                        result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(
                            c::TaskPage {
                                live: List::with_capacity(0),
                                live_after: None,
                                ended: c::EndedGroup::Rows { rows: List::with_capacity(0), after: None, cut: false },
                            },
                        )))),
                    }
                }
                jig_client::Down::Open { .. }
                | jig_client::Down::Read { .. }
                | jig_client::Down::Continue { .. }
                | jig_client::Down::Close { .. }
                | jig_client::Down::Save { .. }
                | jig_client::Down::Erase { .. }
                | jig_client::Down::Send { .. } => continue,
            };
            d::step(&mut root, &env, Event::Client { event: jig_client::Event::Up { up } }, &mut out);
        }
        let mut view = f::browser_page_view(&root);
        let tabs = styled(view.tree(), look::TABBAR);
        assert!(
            tabs[0].styles.as_slice().contains(&look::TABBAR_BEGINNING),
            "{:?}",
            pages::home::facts(&root, f::PROJECT, f::NOW)
        );
        assert_eq!(tabs[0].styles.as_slice().contains(&look::TABBAR_ASK), asking);
        let links = children(tabs);
        assert_eq!(
            links.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
            [b"Home".as_slice(), b"Ask temper", b"Deployment"]
        );
        assert_eq!(f::nodes_text(tabs).as_ref(), b"HomeAskDeployment");
        assert_eq!(current_names(view.tree(), look::TABBAR), [current]);
        let deployment = links[2].id;
        assert_eq!(crate::href(&mut view, deployment).as_deref(), Some(b"/accounts".as_slice()));
    }
}

#[test]
fn the_foot_keeps_status_identity_and_lost_sign_out_and_places_behind_for_each_shell() {
    for state in [jig_client::Link::Live, jig_client::Link::Offline] {
        let (mut root, watch) = f::root_with_watch(None);
        if state == jig_client::Link::Offline {
            f::root_event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
                    },
                },
            );
        }
        for browser in [false, true] {
            let view = if browser { f::browser_page_view(&root) } else { f::page_view(&root) };
            let foot = children(styled(view.tree(), look::RAIL_FOOT));
            let region = foot.iter().find(|node| node.region == Some(crate::frame::link_region())).unwrap();
            let expected = match state {
                jig_client::Link::Starting => b"Connecting".as_slice(),
                jig_client::Link::Offline => b"Offline",
                jig_client::Link::Live => b"Live",
                jig_client::Link::SignedOut => b"Signed out",
                jig_client::Link::OutOfDate => b"Out of date",
            };
            assert!(foot.iter().any(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Deployment")));
            if browser {
                assert!(f::find(view.tree(), Role::Menu, b"Pat").is_some());
            } else {
                assert_eq!(foot.last().unwrap().name.as_deref(), Some(b"Pat".as_slice()));
            }
            let status = f::find(view.tree(), Role::Status, expected).unwrap();
            assert!(
                view.tree()
                    .nodes()
                    .iter()
                    .skip_while(|n| n.id != region.id)
                    .take(usize::try_from(region.size).unwrap())
                    .any(|n| n.id == status)
            );
            assert_eq!(view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::TABBAR)), browser);
        }
        let (mut root, _, page) = f::root_at(Address::Inbox { filter: d::address::Filter::ALL }, &f::domain_limits());
        f::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::WatchEnded {
                        watch: page.unwrap(),
                        end: jig_client::contract::WatchEnd::Dropped,
                    },
                },
            },
        );
        for browser in [false, true] {
            let view = if browser { f::browser_page_view(&root) } else { f::page_view(&root) };
            let foot = children(styled(view.tree(), look::RAIL_FOOT));
            let behind = f::find(view.tree(), Role::Status, b"Behind").unwrap();
            let region = foot.iter().find(|node| node.region == Some(crate::frame::link_region())).unwrap();
            let inside = view
                .tree()
                .nodes()
                .iter()
                .skip_while(|n| n.id != region.id)
                .take(usize::try_from(region.size).unwrap())
                .any(|n| n.id == behind);
            assert_eq!(inside, !browser);
            if browser {
                let at = foot.iter().position(|node| node.id == region.id).unwrap();
                assert_eq!(foot[at + 1].id, behind);
            }
        }
        f::root_event(&mut root, Event::Act { action: d::Action::Pages(pages::Act::SignOut) });
        f::root_event(&mut root, Event::Client { event: jig_client::Event::SignOutLost });
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(
                children(styled(view.tree(), look::RAIL_FOOT)).last().unwrap().name.as_deref(),
                Some(b"Sign out did not reach the engine.".as_slice())
            );
            assert!(f::find(view.tree(), Role::Button, b"Sign out").is_none());
        }
    }
}

#[test]
fn settings_carries_both_forge_and_account_marks() {
    use jig_client::contract::AccountState;
    use temper_client_protocol_wire::v1 as w;
    for (connection, account, style, forge_words, account_words) in [
        (
            w::Connection::Connecting,
            AccountState::Later { until: None },
            look::NAV_LIMITED,
            b"a forge is setting up".as_slice(),
            b"an account is limited".as_slice(),
        ),
        (
            w::Connection::Stopped(w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::default() }).unwrap()),
            AccountState::SignInAgain,
            look::NAV_NEEDS,
            b"a forge needs you".as_slice(),
            b"an account needs you".as_slice(),
        ),
    ] {
        let (mut root, watch) =
            f::forge_root_accounts(Address::Accounts, super::connect_forge::rows(connection, w::Offer::Offered));
        let mut accounts = super::accounts::document(1);
        accounts.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state = account;
        super::accounts::deliver(&mut root, watch, accounts);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let foot = styled(view.tree(), look::RAIL_FOOT);
            let link = children(foot).into_iter().find(|n| n.role == Role::Link).unwrap();
            assert_eq!(link.name.as_deref(), Some(b"Deployment".as_slice()));
            assert!(link.states.as_slice().contains(&State::Current));
            let marks = foot.iter().filter(|n| n.styles.as_slice().contains(&style)).collect::<Vec<_>>();
            assert_eq!(marks.len(), 2);
            assert_eq!(marks[0].name.as_deref(), Some(forge_words));
            assert_eq!(marks[1].name.as_deref(), Some(account_words));
            let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
            skein_ui_text::render(view.tree(), b"temper", Some(crate::frame::link_region()), &mut print);
            let text = String::from_utf8_lossy(print.text.as_slice());
            let expected = format!(
                "Deployment · {} · {}",
                core::str::from_utf8(forge_words).unwrap(),
                core::str::from_utf8(account_words).unwrap()
            );
            if !view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::TABBAR)) {
                assert!(text.contains(&expected), "{text}");
            }
        }
    }
}

#[test]
fn the_playground_setup_link_leads_the_foot_and_leaves() {
    let mut root = f::playground_root(Address::Forges, f::PROJECT, true);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let foot = children(styled(view.tree(), look::RAIL_FOOT));
        let link = foot.iter().find(|node| node.styles.as_slice().contains(&look::LEAVE_PLAYGROUND)).unwrap();
        assert_eq!(link.name.as_deref(), Some(b"Set up temper".as_slice()));
        let id = link.id;
        assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Forge(d::forge::Act::EndPlayground)));
    }
    f::root_event(&mut root, Event::Act { action: d::Action::Forge(d::forge::Act::EndPlayground) });
    assert!(f::find(f::page_view(&root).tree(), Role::Link, b"Set up temper").is_none());
}

#[test]
fn the_terminal_numbers_the_rail_and_keeps_the_first_line() {
    let (mut root, watch) = f::root_with_watch(None);
    f::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
        },
    );
    for (people, first_line, lines) in [
        (
            None,
            "== Inbox == Offline; Behind",
            &[
                "[2] All projects · 2 projects (pressed)",
                "[7] How work runs for Shop",
                "[8] Deployment",
                "[9] Home",
                "[10] Needs you (current)",
                "[11] Work",
                "[12] How work runs",
                "[13] Draft",
                "[14] Change",
                "[15] Away",
                "[16] Deployment",
            ][..],
        ),
        (
            Some(2),
            "== Who owns what on Shop == Offline; Behind",
            &[
                "[2] All projects · 2 projects",
                "[7] Team for Shop (current)",
                "[8] How work runs for Shop",
                "[9] Deployment",
                "[10] Home",
                "[11] Needs you",
                "[12] Work",
                "[13] Team (current)",
                "[14] How work runs",
                "[15] Draft",
                "[16] Change",
                "[17] Away",
                "[18] Deployment",
            ][..],
        ),
    ] {
        if let Some(people) = people {
            f::root_event(&mut root, Event::Went { place: d::Place::Address(Address::Team { project: f::PROJECT }) });
            with_people(&mut root, people);
        }
        let view = f::page_view(&root);
        let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
        skein_ui_text::render(view.tree(), b"temper", Some(crate::frame::link_region()), &mut print);
        let text = String::from_utf8_lossy(print.text.as_slice());
        assert_eq!(text.lines().next(), Some(first_line));
        for line in lines {
            assert!(text.lines().any(|s| s.trim() == *line), "{line}: {text}");
        }
        assert!(!text.contains("Sections:"));
        assert!(!text.contains("Sign out"));
    }
}

#[test]
fn kind_seven_changes_no_navigation_entry() {
    use d::forge::ways::Ways;
    for (ways, refused, people) in [
        (Some(Ways { goals: true, changes: true, ..Ways::STOCK }), false, None),
        (None, false, None),
        (None, true, None),
        (Some(Ways::STOCK), false, None),
        (Some(Ways::STOCK), false, Some(1)),
        (Some(Ways::STOCK), false, Some(2)),
    ] {
        let address = Address::Changes { project: f::PROJECT };
        let mut root = if refused { f::root_with_refused_ways(address) } else { f::root_with_ways(address, ways) };
        if let Some(people) = people {
            with_people(&mut root, people);
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            for style in [look::RAIL_NAV, look::TABBAR] {
                if !view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&style)) {
                    continue;
                }
                let names: Vec<_> =
                    children(styled(view.tree(), style)).into_iter().filter_map(|n| n.name.as_deref()).collect();
                assert_eq!(
                    names,
                    if style == look::RAIL_NAV {
                        if people == Some(2) {
                            vec![b"Home".as_slice(), b"Needs you", b"Work", b"Team", b"How work runs"]
                        } else {
                            vec![b"Home".as_slice(), b"Needs you", b"Work", b"How work runs"]
                        }
                    } else {
                        vec![b"Home".as_slice(), b"Needs", b"Work"]
                    }
                );
                assert_eq!(current_names(view.tree(), style), [b"Work".as_slice()]);
            }
        }
    }
}
#[test]
fn drafts_offered_reads_drafts() {
    const OTHER_PROJECT: u32 = 17;
    use d::forge::ways::Ways;
    for (ways, offered) in [
        (None, false),
        (Some(Ways::STOCK), true),
        (Some(Ways { drafts: false, ..Ways::STOCK }), false),
        (Some(Ways { drafts: false, goals: true, ..Ways::STOCK }), false),
    ] {
        let root = f::root_with_ways(Address::Chats { filter: ChatsFilter::All }, ways);
        assert_eq!(crate::pages::chats::drafts_offered(&root, f::PROJECT), offered);
        assert!(!crate::pages::chats::drafts_offered(&root, OTHER_PROJECT));
    }
}

fn deliver_count(root: &mut d::Domain, watch: Token, count: u32, cut: bool) {
    use jig_client::contract as c;
    let mut rows = List::with_capacity(count);
    for number in 0..count {
        rows.push(super::documents::entry(
            c::ObjectKey::PersonTask(u64::from(number)),
            f::PROJECT,
            1,
            skein_lib::Wall::EPOCH,
            None,
        ))
        .unwrap();
    }
    f::root_event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(super::documents::inbox(
                        rows,
                        count,
                        c::Count { value: 0, limited: false },
                        0,
                        cut,
                    )))),
                },
            },
        },
    );
}

#[test]
fn the_switch_shows_when_a_fresh_projects_classes_read_as_stock() {
    let root = f::root_with_ways(Address::Home { project: f::PROJECT }, Some(d::forge::ways::Ways::STOCK));
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let node = &styled(view.tree(), look::AWAY)[0];
        assert_eq!(node.role, Role::Button);
        assert_eq!(node.name.as_deref(), Some(b"Away".as_slice()));
        let id = node.id;
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(d::Action::Pages(pages::Act::Away(pages::away::Act::Toggle { project: f::PROJECT })))
        );
        let foot = children(styled(view.tree(), look::RAIL_FOOT));
        let at = foot.iter().position(|n| n.styles.as_slice().contains(&look::AWAY)).unwrap();
        assert_eq!(foot[at + 1].region, Some(crate::frame::link_region()));
        assert!(foot.iter().any(|node| node.name.as_deref() == Some(b"Deployment")));
    }
}
#[test]
fn a_goal_alone_does_not_show_the_switch() {
    let (mut root, me, _, _, _, _, _, _, _) = f::root_at_with_values(
        Address::Home { project: f::PROJECT },
        &f::domain_limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        None,
        false,
        true,
    );
    with_goal_fact(&mut root, me, Some(true));
    assert!(!root.away_on(f::PROJECT, f::NOW));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::AWAY)));
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::AWAY_CHIP)));
    }
}
#[test]
fn you_approves_classes_do_not_show_the_switch() {
    let root = f::root_with_classes(
        Address::Home { project: f::PROJECT },
        &temper_protocol_reaches::Classes::stock(temper_protocol_reaches::Way::YouApprove),
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::AWAY)));
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::AWAY_CHIP)));
    }
}
#[test]
fn the_switch_shows_across_projects_with_stock_classes() {
    for (address, expected) in [
        (Address::Home { project: f::PROJECT }, pages::away::Act::Toggle { project: f::PROJECT }),
        (Address::AllHome, pages::away::Act::ToggleAll),
    ] {
        let mut root = f::root_with_ways(Address::Home { project: f::PROJECT }, Some(d::forge::ways::Ways::STOCK));
        f::root_event(&mut root, Event::Went { place: d::Place::Address(address.clone()) });
        let mut view = f::page_view(&root);
        let node = &styled(view.tree(), look::AWAY)[0];
        assert_eq!(node.role, Role::Button, "{address:?}");
        assert_eq!(node.name.as_deref(), Some(b"Away".as_slice()), "{address:?}");
        assert!(!node.states.as_slice().contains(&State::Pressed), "{address:?}");
        let id = node.id;
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(d::Action::Pages(pages::Act::Away(expected))),
            "{address:?}"
        );
    }
}
#[test]
fn on_it_says_until_and_is_pressed() {
    for zone in [0_i32, 120_i32] {
        let mut root = f::away_root(Some(skein_lib::Wall::from_nanos(28_800_000_000_000)), None);
        root.set_zone_minutes(zone);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let node = &styled(view.tree(), look::AWAY)[0];
            assert_eq!(
                node.name.as_deref(),
                Some(if zone == 0_i32 { b"Away until 08:00".as_slice() } else { b"Away until 10:00" })
            );
            assert!(node.states.as_slice().contains(&State::Pressed));
            assert_eq!(children(styled(view.tree(), look::AWAY)).last().unwrap().styles.as_slice(), [look::AWAY_TRACK]);
        }
    }
}
#[test]
fn on_with_no_until_it_says_away() {
    let root = f::away_root(None, None);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let node = &styled(view.tree(), look::AWAY)[0];
        assert_eq!(node.name.as_deref(), Some(b"Away".as_slice()));
        assert!(node.states.as_slice().contains(&State::Pressed));
    }
}
#[test]
fn past_until_it_draws_off_and_asks_a_wake() {
    let until = skein_lib::Wall::from_nanos(28_800_000_000_000);
    let root = f::away_root(Some(until), None);
    let mut times = crate::words::Times::new(f::NOW);
    let mut builder = skein_ui::Builder::new(&f::tree_limits());
    crate::frame::rail(
        &mut builder,
        &root,
        &f::browser_words(),
        &mut crate::frame::Bindings::new(f::tree_limits().bindings),
        &mut crate::forge::bindings::Bindings::new(
            f::tree_limits().bindings,
            f::domain_limits().forge.checks,
            f::FORGE,
        ),
        f::PROJECT,
        &mut times,
        &crate::forge_sign_in::Forges::new(),
    );
    assert_eq!(times.next, Some(until), "the rail requests the expiry wake independently of page times");
    let mut view = f::view_of(f::browser_words());
    crate::build(&mut view, &root, skein_lib::Wall::from_nanos(until.as_nanos() - 1), &mut Queue::uncapped());
    assert_eq!(view.next_change(), Some(until));
    crate::build(&mut view, &root, until, &mut Queue::uncapped());
    let node = &styled(view.tree(), look::AWAY)[0];
    assert_eq!(node.name.as_deref(), Some(b"Away".as_slice()));
    assert!(!node.states.as_slice().contains(&State::Pressed));
    let (mut root, me) = f::away_frame(Some(until), None);
    with_goal_fact(&mut root, me, Some(true));
    let mut view = f::view_of(f::browser_words());
    crate::build(&mut view, &root, skein_lib::Wall::from_nanos(until.as_nanos() - 1), &mut Queue::uncapped());
    assert_eq!(view.next_change(), Some(until));
    crate::build(&mut view, &root, until, &mut Queue::uncapped());
    let node = &styled(view.tree(), look::AWAY)[0];
    assert_eq!(node.name.as_deref(), Some(b"Away".as_slice()));
    assert!(!node.states.as_slice().contains(&State::Pressed));
    assert_eq!(styled(view.tree(), look::AWAY_CHIP)[0].name.as_deref(), Some(b"Away: off".as_slice()));
    assert!(view.next_change().is_none_or(|next| next > until));
}
#[test]
fn disabled_while_out() {
    let mut root = f::away_root(None, None);
    f::away_toggle(&mut root);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        for style in [look::AWAY, look::AWAY_CHIP] {
            if let Some(node) = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&style)) {
                assert!(node.states.as_slice().contains(&State::Disabled));
                let id = node.id;
                assert_eq!(crate::act(&mut view, id, Act::Press), None);
            }
        }
    }
}
#[test]
fn the_chip_is_the_browsers_alone() {
    let root = f::away_root(None, None);
    let terminal = f::page_view(&root);
    assert!(!terminal.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::AWAY_CHIP)));
    let browser = f::browser_page_view(&root);
    let parts = children(styled(browser.tree(), look::RAIL));
    assert_eq!(
        parts.iter().map(|node| node.role).collect::<Vec<_>>(),
        [Role::Link, Role::Disclosure, Role::Button, Role::Slot, Role::Slot, Role::Slot, Role::Button]
    );
    assert_eq!(parts[3].styles.as_slice(), [look::RAIL_NAV]);
    assert_eq!(parts[4].styles.as_slice(), [look::RAIL_NEW]);
    assert_eq!(parts[5].styles.as_slice(), [look::RAIL_FOOT]);
    assert_eq!(parts[1].styles.as_slice(), [look::PROJECTS]);
    assert_eq!(parts[2].styles.as_slice(), [look::AWAY_CHIP]);
    assert_eq!(parts[2].role, Role::Button);
    assert!(parts[2].states.as_slice().contains(&State::Pressed));
}
#[test]
fn in_the_terminal_it_is_numbered() {
    let root = f::away_root(None, None);
    let mut view = f::page_view(&root);
    let id = styled(view.tree(), look::AWAY)[0].id;
    let (id, _) = f::terminal_command(view.tree(), &view.limits().tree, id, b"Away (pressed)", b"Deployment");
    assert_eq!(
        crate::act(&mut view, id, Act::Press),
        Some(d::Action::Pages(pages::Act::Away(pages::away::Act::Toggle { project: f::PROJECT })))
    );
}

#[test]
fn the_row_is_a_button_and_a_link_beside_it() {
    // The shell draws the button's name; a child text would repeat it.
    const NAME_BUTTON_NODES: u32 = 1;
    let root = f::forge_root(Address::Home { project: f::PROJECT }, project_rows(1, b"hello"));
    let mut view = repository_view(&root, f::browser_words());
    let menu = children(styled(view.tree(), look::PROJECTS_MENU));
    assert_eq!(menu[0].role, Role::Slot);
    let row = &view.tree().nodes()[view
        .tree()
        .nodes()
        .iter()
        .position(|n| n.name.as_deref() == Some(b"Shop") && n.role == Role::Button)
        .unwrap()
        - 1..];
    let row = &row[..usize::try_from(row[0].size).unwrap()];
    let parts = children(row);
    assert_eq!(parts.iter().map(|n| n.role).collect::<Vec<_>>(), [Role::Button, Role::Text]);
    assert_eq!(parts[0].name.as_deref(), Some(b"Shop".as_slice()));
    assert_eq!(parts[0].size, NAME_BUTTON_NODES);
    let button = parts[0].id;
    let link = f::find(view.tree(), Role::Link, "work/hello ↗".as_bytes()).expect("repository link");
    assert_eq!(
        crate::href(&mut view, link).as_deref(),
        Some(b"https://forge.example/user/oauth2/temper?redirect_to=%2Fwork%2Fhello".as_slice())
    );
    assert_eq!(crate::act(&mut view, link, Act::Press), None);
    assert_eq!(
        crate::act(&mut view, button, Act::Press),
        Some(d::Action::Pages(pages::Act::Project { project: f::PROJECT }))
    );
}

fn repository_view(root: &d::Domain, words: crate::words::Words) -> crate::View {
    let mut view = f::view_of(words);
    let mut facts = crate::forge_sign_in::Forges::new();
    facts.read("3 forgejo https://forge.example owned");
    view.forge_sign_in(facts);
    crate::build(&mut view, root, f::NOW, &mut Queue::uncapped());
    view
}
#[test]
fn several_repositories_keep_the_count_outside_the_link() {
    let root = f::forge_root(Address::Home { project: f::PROJECT }, project_rows(3, b"hello"));
    let view = repository_view(&root, f::browser_words());
    let nodes = view.tree().nodes();
    let at = nodes.iter().position(|n| n.styles.as_slice().contains(&look::PROJECTS_REPO) && n.name.is_none()).unwrap();
    let small = children(&nodes[at..at + usize::try_from(nodes[at].size).unwrap()]);
    assert_eq!(small.iter().map(|n| n.role).collect::<Vec<_>>(), [Role::Link, Role::Text]);
    assert_eq!(small[0].name.as_deref(), Some("work/hello ↗".as_bytes()));
    assert_eq!(small[1].styles.as_slice(), [look::PROJECTS_MORE]);
    assert_eq!(small[1].text.get(0).unwrap().text.as_ref(), b" +2");
}
fn repository_pages() -> [Address; 4] {
    [
        Address::Home { project: f::PROJECT },
        Address::Repositories { project: f::PROJECT },
        Address::Forges,
        Address::Welcome { step: d::Step::Forge { forge: None } },
    ]
}
fn linked_rows() -> List<temper_client_protocol_wire::v1::ForgeRow> {
    use temper_client_protocol_wire::v1 as w;
    let rows = super::connect_forge::first_rows(super::connect_forge::connected(), super::connect_forge::made());
    let mut row = rows.get(0).unwrap().clone().into_parts();
    let adopted = project_rows(1, b"hello");
    row.repositories = adopted.get(0).unwrap().clone().into_parts().repositories;
    let mut rows = List::with_capacity(1);
    rows.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    rows
}
#[test]
fn no_link_sits_inside_a_button() {
    for address in repository_pages() {
        let root = f::forge_root(address.clone(), linked_rows());
        let mut view = repository_view(&root, f::browser_words());
        let tree = view.tree();
        for (at, node) in tree.nodes().iter().enumerate() {
            if node.role == Role::Button {
                let end = at.checked_add(usize::try_from(node.size).unwrap()).unwrap();
                assert!(!tree.nodes()[at..end].iter().any(|child| child.role == Role::Link), "{address:?}");
            }
        }
        if let Address::Welcome { .. } = address {
            assert!(
                f::find(
                    tree,
                    Role::Paragraph,
                    "Forgejo on this machine · connected as temper · work/hello made".as_bytes()
                )
                .is_some()
            );
            let line = styled(tree, look::WELCOME_FORGE_STATUS);
            let at = tree.nodes().iter().position(|node| node.id == line[0].id).unwrap();
            assert_eq!(
                f::node_text(tree, at).as_ref(),
                "✓Forgejo on this machine · connected as temper · work/hello ↗ made".as_bytes()
            );
        }
        let link = if let Address::Home { .. } = address {
            f::find(tree, Role::Link, "work/hello ↗".as_bytes()).expect("switcher link")
        } else {
            styled(tree, look::MAIN)
                .iter()
                .find(|node| node.role == Role::Link && node.name.as_deref() == Some("work/hello ↗".as_bytes()))
                .expect("body repository link")
                .id
        };
        assert_eq!(
            crate::href(&mut view, link).as_deref(),
            Some(b"https://forge.example/user/oauth2/temper?redirect_to=%2Fwork%2Fhello".as_slice())
        );
        let terminal = repository_view(&root, f::words());
        assert!(
            !terminal
                .tree()
                .nodes()
                .iter()
                .any(|n| n.role == Role::Link
                    && n.external.as_deref().is_some_and(|url| url.ends_with(b"%2Fwork%2Fhello"))),
            "{address:?}"
        );
    }
}
#[test]
fn the_playground_and_no_repository_stay_plain() {
    for address in repository_pages() {
        for root in vec![
            f::playground_root(address.clone(), f::PROJECT, true),
            f::forge_root(address.clone(), project_rows(0, b"hello")),
        ]
        .into_boxed_slice()
        {
            let view = repository_view(&root, f::browser_words());
            assert!(
                !view.tree().nodes().iter().any(|n| n.role == Role::Link
                    && n.name.as_deref().is_some_and(|name| name.ends_with(" ↗".as_bytes()) && name.contains(&b'/'))),
                "{address:?}"
            );
        }
    }
}
#[test]
fn the_terminal_row_is_unchanged() {
    let root = f::forge_root(Address::Home { project: f::PROJECT }, project_rows(3, b"hello"));
    let view = repository_view(&root, f::words());
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", Some(crate::frame::link_region()), &mut print);
    let text = String::from_utf8_lossy(print.text.as_slice());
    assert!(text.lines().any(|line| line.trim() == "[3] Shop · work/hello +2 (pressed)"), "{text}");
    assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::PROJECTS_ROW)));
}
