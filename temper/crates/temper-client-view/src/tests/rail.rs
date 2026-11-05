//! The project's frame (temper/docs/design/client/README.md, 5.1 and 5.3).
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
fn the_rails_new_chat_is_a_link_and_a_menu() {
    let root = f::root_with_ways(Address::Home { project: f::PROJECT }, Some(d::forge::ways::Ways::STOCK));
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let entry = styled(view.tree(), look::RAIL_NEW);
        assert_eq!(entry[0].role, Role::Slot);
        let parts = children(entry);
        assert_eq!(parts.iter().map(|n| n.role).collect::<Vec<_>>(), [Role::Link, Role::Menu]);
        assert_eq!(parts[0].name.as_deref(), Some(b"New chat".as_slice()));
        assert_eq!(parts[1].name.as_deref(), Some(b"More ways to start work".as_slice()));
        let link = parts[0].id;
        assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/chats/new".as_slice()));
        assert_eq!(
            crate::act(&mut view, link, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::NewChat { draft: false } }))
        );
    }
}

#[test]
fn its_items_go_where_they_say() {
    let root = f::root_with_ways(
        Address::Home { project: f::PROJECT },
        Some(d::forge::ways::Ways { drafts: true, ..d::forge::ways::Ways::STOCK }),
    );
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        for (name, description, draft) in [
            (b"Chat".as_slice(), b"Ask a question or ask for a change.".as_slice(), false),
            (b"Goal".as_slice(), b"Agents plan a larger change and land it.".as_slice(), true),
        ] {
            let node = styled(view.tree(), look::RAIL_NEW)
                .iter()
                .find(|n| n.role == Role::MenuItem && n.name.as_deref() == Some(name))
                .unwrap();
            assert_eq!(node.description.as_deref(), Some(description));
            let id = node.id;
            assert_eq!(
                crate::act(&mut view, id, Act::Press),
                Some(d::Action::Pages(pages::Act::Go { address: Address::NewChat { draft } }))
            );
        }
    }
}

#[test]
fn goal_shows_while_drafts_are_offered() {
    for ways in [
        None,
        Some(d::forge::ways::Ways::STOCK),
        Some(d::forge::ways::Ways { drafts: true, ..d::forge::ways::Ways::STOCK }),
    ] {
        let root = f::root_with_ways(Address::Home { project: f::PROJECT }, ways);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let items = styled(view.tree(), look::RAIL_NEW)
                .iter()
                .filter(|n| n.role == Role::MenuItem)
                .map(|n| n.name.as_deref().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                items,
                if ways.is_some_and(|w| w.drafts) {
                    vec![b"Chat".as_slice(), b"Goal", b"All ways to start work"]
                } else {
                    vec![b"Chat".as_slice(), b"All ways to start work"]
                }
            );
        }
    }
}

#[test]
fn all_ways_goes_to_settings() {
    let root = f::root_with_ways(Address::Home { project: f::PROJECT }, Some(d::forge::ways::Ways::STOCK));
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let node = styled(view.tree(), look::RAIL_NEW_ALL).first().unwrap();
        assert_eq!(node.role, Role::MenuItem);
        assert_eq!(node.name.as_deref(), Some(b"All ways to start work".as_slice()));
        let id = node.id;
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::Settings { project: f::PROJECT } }))
        );
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
    children(styled(tree, look::PROJECTS_MENU)).into_iter().filter(|n| n.role == Role::Button).collect()
}

#[test]
fn switcher_rows_keep_repository_words_and_counts_in_both_shells() {
    // Sixteen repositories leave fifteen after the displayed one.
    const SIXTEEN: u32 = 16;
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
        (
            f::forge_root(address.clone(), project_rows(SIXTEEN, b"hello")),
            Some("work/hello"),
            Some(" +15"),
            "Shop · work/hello +15",
        ),
        (
            f::forge_root(address.clone(), project_rows(2, b"customer-portal-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")),
            Some("work/customer-portal-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
            Some(" +1"),
            "Shop · work/customer-portal-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx +1",
        ),
        (f::playground_root(address, f::PROJECT, true), Some("acme/shop"), None, "Shop · acme/shop"),
    ]
    .into_boxed_slice()
    {
        for browser in [false, true] {
            let mut view = if browser { f::browser_page_view(&root) } else { f::page_view(&root) };
            let rows = switcher_rows(view.tree());
            assert_eq!(rows.len(), 2);
            assert!(rows[0].states.as_slice().contains(&State::Pressed));
            assert!(!rows[1].states.as_slice().contains(&State::Pressed));
            let id = rows[0].id;
            if browser {
                assert_eq!(rows[0].name, None);
                let menu = styled(view.tree(), look::PROJECTS_MENU);
                let end = 1 + usize::try_from(rows[0].size).unwrap();
                let parts = children(&menu[1..end]);
                assert_eq!(parts.len(), if repository.is_some() { 2 } else { 1 });
                assert_eq!(parts[0].text.get(0).unwrap().text.as_ref(), b"Shop");
                if let Some(repository) = repository {
                    assert_eq!(parts[1].role, Role::Text);
                    assert_eq!(parts[1].styles.as_slice(), [look::PROJECTS_REPO]);
                    let small = styled(view.tree(), look::PROJECTS_REPO);
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
fn the_panel_puts_rename_its_form_new_and_settings_after_projects() {
    let (mut root, watch) = f::root_with_watch(None);
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().role = jig_client::contract::HeldRole::Owner;
    f::deliver_me(&mut root, watch, me);
    for renaming in [false, true] {
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let disclosure = children(styled(view.tree(), look::PROJECTS));
            assert_eq!(disclosure.len(), 1);
            assert_eq!(disclosure[0].styles.as_slice(), [look::PROJECTS_MENU]);
            let panel = children(styled(view.tree(), look::PROJECTS_MENU));
            assert_eq!(panel.iter().map(|n| n.role).collect::<Vec<_>>(), [Role::Button, Role::Button, Role::Slot]);
            assert_eq!(panel[2].styles.as_slice(), [look::PROJECTS_ACTS]);
            let acts = children(styled(view.tree(), look::PROJECTS_ACTS));
            assert_eq!(
                acts.iter().map(|n| n.role).collect::<Vec<_>>(),
                [if renaming { Role::Form } else { Role::Button }, Role::Form, Role::Link]
            );
            assert_eq!(acts[0].name.as_deref(), Some(b"Rename Shop".as_slice()));
            assert_eq!(acts[1].name.as_deref(), Some(b"New project".as_slice()));
            assert_eq!(acts[2].name.as_deref(), Some(b"Settings for Shop".as_slice()));
            let settings = acts[2].id;
            assert_eq!(crate::href(&mut view, settings).as_deref(), Some(b"/projects/1/settings".as_slice()));
        }
        f::root_event(
            &mut root,
            Event::Act { action: d::Action::Pages(pages::Act::Switcher(pages::switcher::Act::Rename)) },
        );
    }
}

#[test]
fn members_keep_new_and_settings_without_rename_in_both_shells() {
    let root = f::root_in_project(None);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let acts = children(styled(view.tree(), look::PROJECTS_ACTS));
        assert_eq!(acts.iter().map(|n| n.role).collect::<Vec<_>>(), [Role::Form, Role::Link]);
        assert_eq!(acts[0].name.as_deref(), Some(b"New project".as_slice()));
        assert_eq!(acts[1].name.as_deref(), Some(b"Settings for Shop".as_slice()));
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
// Four sections on a phone (client/README.md, 5.3).
const TABS: usize = 4;

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
    let mut configuring = goal.clone();
    configuring.number = 3;
    configuring.executor = c::TemplateExecutor::Charter(3);
    me.projects.get_mut(0).unwrap().templates = List::from_boxed(Box::new([goal, configuring]));
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
fn current_names(tree: &Tree, style: Style) -> Vec<&[u8]> {
    children(styled(tree, style))
        .into_iter()
        .filter(|node| node.states.as_slice().contains(&State::Current))
        .map(|node| node.name.as_deref().unwrap())
        .collect()
}

#[test]
fn goals_follows_kind_seven_or_keeps_its_open_page() {
    for (address, delivered, shown) in [
        (Address::Home { project: f::PROJECT }, None, false),
        (Address::Home { project: f::PROJECT }, Some(false), false),
        (Address::Home { project: f::PROJECT }, Some(true), false),
        (Address::Goals { project: f::PROJECT, filter: GoalsFilter::Ended }, None, true),
        (Address::Goals { project: f::PROJECT, filter: GoalsFilter::All }, Some(false), true),
    ] {
        let (mut root, me, _) = f::root_at(address, &f::domain_limits());
        with_goal_fact(&mut root, me, delivered);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let goals = children(styled(view.tree(), look::RAIL_NAV))
                .into_iter()
                .find(|node| node.name.as_deref() == Some(b"Goals"))
                .map(|node| node.id);
            assert_eq!(goals.is_some(), shown);
            if let Some(id) = goals {
                assert_eq!(crate::href(&mut view, id).as_deref(), Some(b"/projects/1/goals".as_slice()));
            }
            let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
            skein_ui_text::render(view.tree(), b"temper", Some(crate::frame::link_region()), &mut print);
            assert_eq!(
                String::from_utf8_lossy(print.text.as_slice()).lines().any(|line| line.contains("] Goals")),
                shown
            );
        }
    }
}

#[test]
fn goals_is_current_on_its_page_a_goal_and_a_goals_task() {
    for (address, priority, goal, delivered, expected) in [
        (
            Address::Goals { project: f::PROJECT, filter: GoalsFilter::Needs },
            None,
            None,
            false,
            vec![b"Goals".as_slice()],
        ),
        (Address::Task { task: GOAL_TASK }, Some(GOAL_PRIORITY), None, true, vec![b"Goals".as_slice()]),
        (Address::Task { task: GOAL_TASK }, None, Some(GOAL_TASK), true, vec![b"Goals".as_slice()]),
        (Address::Task { task: GOAL_TASK }, None, None, true, vec![]),
        (Address::Task { task: GOAL_TASK }, Some(GOAL_PRIORITY), Some(GOAL_TASK), false, vec![]),
    ] {
        let (mut root, me, task) = f::root_at(address, &f::domain_limits());
        with_goal_fact(&mut root, me, Some(true));
        if delivered {
            let mut document = f::task::delegate();
            document.project = f::PROJECT;
            document.priority = priority;
            document.goal = goal;
            deliver_document(&mut root, task.unwrap(), c::Delivery::Task(Box::new(document)));
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(current_names(view.tree(), look::RAIL_NAV), expected);
            if view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&look::TABBAR)) {
                assert_eq!(current_names(view.tree(), look::TABBAR), expected);
            }
        }
    }
}

#[test]
fn a_chat_is_never_goals() {
    for (configuring, expected) in [(false, b"Chats".as_slice()), (true, b"Settings".as_slice())] {
        let (mut root, me, chat) = f::root_at(Address::Chat { task: GOAL_TASK }, &f::domain_limits());
        with_goal_fact(&mut root, me, Some(true));
        let mut document = f::task::delegate();
        document.project = f::PROJECT;
        document.goal = Some(GOAL_TASK);
        document.priority = Some(GOAL_PRIORITY);
        document.chip.executor = c::ChipExecutor::Agent { charter: if configuring { 3 } else { 1 } };
        deliver_document(&mut root, chat.unwrap(), c::Delivery::Task(Box::new(document)));
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let nav = current_names(view.tree(), look::RAIL_NAV);
            let foot = current_names(view.tree(), look::RAIL_FOOT);
            assert_eq!(nav.into_iter().chain(foot).collect::<Vec<_>>(), [expected]);
        }
    }
}

#[test]
fn the_tab_bar_is_four_either_way() {
    for (has_goal, expected) in [
        (false, [b"Home".as_slice(), b"Needs you", b"Chats", b"Settings"]),
        (true, [b"Home".as_slice(), b"Needs you", b"Goals", b"Chats"]),
    ] {
        let root = f::root_with_ways(
            Address::Home { project: f::PROJECT },
            Some(d::forge::ways::Ways { goals: has_goal, ..d::forge::ways::Ways::STOCK }),
        );
        let mut view = f::browser_page_view(&root);
        let links = children(styled(view.tree(), look::TABBAR));
        assert_eq!(links.len(), TABS);
        assert_eq!(links.iter().map(|node| node.name.as_deref().unwrap()).collect::<Vec<_>>(), expected);
        let ids: Vec<_> = links.iter().map(|node| node.id).collect();
        let hrefs = if has_goal {
            [b"/projects/1".as_slice(), b"/?project=1", b"/projects/1/goals", b"/chats"]
        } else {
            [b"/projects/1".as_slice(), b"/?project=1", b"/chats", b"/projects/1/settings"]
        };
        for (id, href) in ids.into_iter().zip(hrefs) {
            assert_eq!(crate::href(&mut view, id).as_deref(), Some(href));
        }
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks both navigation surfaces across addresses and counts")]
fn rail_and_tabbar_keep_the_order_addresses_current_states_and_count() {
    let addresses = [
        Address::Home { project: f::PROJECT },
        Address::Changes { project: f::PROJECT },
        Address::Goals { project: f::PROJECT, filter: GoalsFilter::All },
        Address::Inbox { filter: d::address::Filter::ALL },
        Address::Chats { filter: ChatsFilter::All },
        Address::NewChat { draft: false },
        Address::Chat { task: 9 },
        Address::Task { task: 9 },
        Address::Accounts,
        Address::ConnectModel { way: None },
        Address::Forges,
        Address::ConnectForge { forge: None },
        Address::Adopt { project: f::PROJECT },
        Address::Settings { project: f::PROJECT },
    ];
    for address in addresses {
        let (mut root, watch, _, inbox) = f::root_at_with_needs(address.clone(), &f::domain_limits());
        for (count, cut) in [(0, false), (3, false), (3, true)] {
            let mut me = root.pages().me(root.client()).unwrap().clone();
            me.needs_total = count;
            f::deliver_me(&mut root, watch, me);
            deliver_count(&mut root, inbox, count, cut);
            for (browser, mut view) in [(false, f::page_view(&root)), (true, f::browser_page_view(&root))] {
                assert_eq!(skein_ui::outline::check(view.tree()), None, "{address:?}");
                assert!(!view.tree().nodes().iter().any(|n| n.role == Role::Banner));
                let rail = styled(view.tree(), look::RAIL);
                assert_eq!(rail[0].role, Role::Navigation);
                assert_eq!(rail[0].name.as_deref(), Some(b"Shop".as_slice()));
                let parts: Vec<_> =
                    children(rail).into_iter().filter(|n| !n.styles.as_slice().contains(&look::RAIL_UNFOLD)).collect();
                assert_eq!(
                    parts.iter().map(|n| n.role).collect::<Vec<_>>(),
                    [Role::Link, Role::Disclosure, Role::Slot, Role::Slot, Role::Slot]
                );
                assert_eq!(parts[0].name.as_deref(), Some(b"temper home".as_slice()));
                assert_eq!(parts[1].styles.as_slice(), [look::PROJECTS]);
                assert_eq!(parts[2].styles.as_slice(), [look::RAIL_NAV]);
                assert_eq!(parts[3].styles.as_slice(), [look::RAIL_NEW]);
                assert_eq!(parts[4].styles.as_slice(), [look::RAIL_FOOT]);
                let brand = parts[0].id;
                let new_chat = children(styled(view.tree(), look::RAIL_NEW))[0].id;
                assert_eq!(crate::href(&mut view, brand).as_deref(), Some(b"/projects/1".as_slice()));
                assert_eq!(crate::href(&mut view, new_chat).as_deref(), Some(b"/chats/new".as_slice()));
                let needs: &[u8] = if count == 0 {
                    b"Needs you".as_slice()
                } else if cut {
                    b"Needs you 3+"
                } else {
                    b"Needs you 3"
                };
                let current = match address {
                    Address::Home { .. } => Some(b"Home".as_slice()),
                    Address::Inbox { .. } => Some(needs),
                    Address::Chats { .. } | Address::NewChat { .. } | Address::Chat { .. } => Some(b"Chats".as_slice()),
                    Address::Changes { .. } => Some(b"Changes".as_slice()),
                    Address::Goals { .. } => Some(b"Goals".as_slice()),
                    Address::Settings { .. }
                    | Address::Builders { .. }
                    | Address::Repositories { .. }
                    | Address::Configure { .. }
                    | Address::Accounts
                    | Address::ConnectModel { .. }
                    | Address::Forges
                    | Address::ConnectForge { .. }
                    | Address::Adopt { .. } => Some(b"Settings".as_slice()),
                    Address::Task { .. } | Address::Welcome { .. } => None,
                };
                let mut styles = vec![look::RAIL_NAV];
                let tab = view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::TABBAR));
                if tab {
                    styles.push(look::TABBAR);
                    let nodes = view.tree().nodes();
                    let at = nodes.iter().position(|n| n.styles.as_slice().contains(&look::TABBAR)).unwrap();
                    let main = nodes.iter().position(|n| n.role == Role::Main).unwrap();
                    assert_eq!(at, main + usize::try_from(nodes[main].size).unwrap());
                    assert_eq!(nodes[at].role, Role::Navigation);
                    assert_eq!(nodes[at].name.as_deref(), Some(b"Sections".as_slice()));
                }
                for style in styles {
                    let links = children(styled(view.tree(), style));
                    let goals = root.pages().goals().is_some();
                    let expected = if goals {
                        vec![b"Home".as_slice(), needs, b"Goals", b"Chats"]
                    } else if style == look::TABBAR {
                        vec![b"Home".as_slice(), needs, b"Chats", b"Settings"]
                    } else if address == (Address::Changes { project: f::PROJECT }) {
                        vec![b"Home".as_slice(), needs, b"Chats", b"Changes"]
                    } else {
                        vec![b"Home".as_slice(), needs, b"Chats"]
                    };
                    assert_eq!(links.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(), expected);
                    let count_nodes = styled(view.tree(), style)
                        .iter()
                        .filter(|n| n.styles.as_slice().contains(&look::NAV_COUNT))
                        .collect::<Vec<_>>();
                    assert_eq!(count_nodes.len(), usize::from(count > 0));
                    if count > 0 {
                        assert_eq!(
                            count_nodes[0].text.get(0).unwrap().text.as_ref(),
                            if cut { b"3+".as_slice() } else { b"3" }
                        );
                    }
                    let ids = links
                        .iter()
                        .map(|n| (n.id, n.name.clone(), n.states.as_slice().contains(&State::Current)))
                        .collect::<Vec<_>>();
                    let hrefs = if goals {
                        vec![b"/projects/1".as_slice(), b"/?project=1", b"/projects/1/goals", b"/chats"]
                    } else {
                        vec![
                            b"/projects/1".as_slice(),
                            b"/?project=1",
                            b"/chats",
                            if style == look::TABBAR { b"/projects/1/settings" } else { b"/projects/1/changes" },
                        ]
                    };
                    for ((id, name, marked), href) in ids.into_iter().zip(hrefs) {
                        assert_eq!(marked, name.as_deref() == current && current.is_some());
                        assert_eq!(crate::href(&mut view, id).as_deref(), Some(href));
                    }
                }
                if let Address::Inbox { .. } = address {
                    if browser {
                        assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Inbox").is_none());
                    } else {
                        assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Inbox").is_some());
                    }
                }
                assert_eq!(f::find(view.tree(), Role::Button, b"Sign out").is_some(), address == Address::Accounts);
            }
        }
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
            assert_eq!(foot[0].region, Some(crate::frame::link_region()));
            let expected = match state {
                jig_client::Link::Starting => b"Connecting".as_slice(),
                jig_client::Link::Offline => b"Offline",
                jig_client::Link::Live => b"Live",
                jig_client::Link::SignedOut => b"Signed out",
                jig_client::Link::OutOfDate => b"Out of date",
            };
            assert!(foot.iter().any(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Settings")));
            assert_eq!(foot.last().unwrap().name.as_deref(), Some(b"Pat".as_slice()));
            let region = foot[0];
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
            let region = foot[0];
            let behind = f::find(view.tree(), Role::Status, b"Behind").unwrap();
            let inside = view
                .tree()
                .nodes()
                .iter()
                .skip_while(|n| n.id != region.id)
                .take(usize::try_from(region.size).unwrap())
                .any(|n| n.id == behind);
            assert_eq!(inside, !browser);
            if browser {
                assert_eq!(foot[1].id, behind);
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
        let (mut root, watch) = f::forge_root_accounts(
            Address::Settings { project: f::PROJECT },
            super::connect_forge::rows(connection, w::Offer::Offered),
        );
        let mut accounts = super::accounts::document(1);
        accounts.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state = account;
        super::accounts::deliver(&mut root, watch, accounts);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let foot = styled(view.tree(), look::RAIL_FOOT);
            let link = children(foot).into_iter().find(|n| n.role == Role::Link).unwrap();
            assert_eq!(link.name.as_deref(), Some(b"Settings".as_slice()));
            assert!(link.states.as_slice().contains(&State::Current));
            let marks = foot.iter().filter(|n| n.styles.as_slice().contains(&style)).collect::<Vec<_>>();
            assert_eq!(marks.len(), 2);
            assert_eq!(marks[0].name.as_deref(), Some(forge_words));
            assert_eq!(marks[1].name.as_deref(), Some(account_words));
            let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
            skein_ui_text::render(view.tree(), b"temper", Some(crate::frame::link_region()), &mut print);
            let text = String::from_utf8_lossy(print.text.as_slice());
            let expected = format!(
                "Settings · {} · {}",
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
        assert_eq!(foot[0].name.as_deref(), Some(b"Set up temper".as_slice()));
        let id = foot[0].id;
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
    let view = f::page_view(&root);
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", Some(crate::frame::link_region()), &mut print);
    let text = String::from_utf8_lossy(print.text.as_slice());
    assert_eq!(text.lines().next(), Some("== Inbox == Offline; Behind"));
    for line in [
        "[7] Home",
        "[8] Needs you (current)",
        "[9] Chats",
        "[10] + New chat",
        "[11] Chat",
        "[12] All ways to start work",
        "[13] Settings",
    ] {
        assert!(text.lines().any(|s| s.trim() == line), "{line}: {text}");
    }
    assert!(!text.contains("Sections:"));
    assert!(!text.contains("Sign out"));
}

#[test]
fn settings_is_current_on_every_page_of_settings() {
    for address in [
        Address::Settings { project: f::PROJECT },
        Address::Repositories { project: f::PROJECT },
        Address::Forges,
        Address::Accounts,
        Address::ConnectForge { forge: None },
        Address::ConnectModel { way: None },
        Address::Adopt { project: f::PROJECT },
        Address::Configure { project: f::PROJECT },
        Address::Chat { task: 9 },
    ] {
        let (mut root, _, _) = f::root_at(address.clone(), &f::domain_limits());
        if address == (Address::Chat { task: 9 }) {
            let template = f::chat::messaging_root();
            let (mut configuring, watch, task_watch) = f::root_at(Address::Chat { task: 9 }, &f::domain_limits());
            f::deliver_me(
                &mut configuring,
                watch,
                f::configuring_me(template.pages().me(template.client()).unwrap().clone()),
            );
            let mut task = template.pages().chat().unwrap().task(template.client()).unwrap().clone();
            task.chip.executor = jig_client::contract::ChipExecutor::Agent { charter: 37 };
            f::root_event(
                &mut configuring,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Watched {
                            watch: task_watch.unwrap(),
                            event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Task(
                                Box::new(task),
                            )),
                        },
                    },
                },
            );
            root = configuring;
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            for style in [look::RAIL_FOOT, look::TABBAR] {
                if style == look::TABBAR && !view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&style)) {
                    continue;
                }
                let links: Vec<_> = styled(view.tree(), style)
                    .iter()
                    .filter(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Settings"))
                    .collect();
                assert_eq!(links.len(), 1, "{address:?}");
                assert!(links[0].states.as_slice().contains(&State::Current), "{address:?}");
            }
        }
    }
}

#[test]
fn the_rail_shows_what_kind_seven_shows() {
    use d::forge::ways::Ways;
    for ways in [
        Ways::STOCK,
        Ways { drafts: true, ..Ways::STOCK },
        Ways { goals: true, ..Ways::STOCK },
        Ways { changes: true, ..Ways::STOCK },
        Ways { away: true, ..Ways::STOCK },
    ] {
        let root = f::root_with_ways(Address::Home { project: f::PROJECT }, Some(ways));
        assert_eq!(root.forge().ways(f::PROJECT), Some(ways));
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let names = children(styled(view.tree(), look::RAIL_NAV))
                .iter()
                .map(|n| n.name.as_deref().unwrap())
                .collect::<Vec<_>>();
            let mut expected = vec![b"Home".as_slice(), b"Needs you"];
            if ways.goals {
                expected.push(b"Goals");
            }
            expected.push(b"Chats");
            if ways.changes {
                expected.push(b"Changes");
            }
            assert_eq!(names, expected);
            assert!(f::find(view.tree(), Role::Link, b"Away").is_none());
        }
    }
}
#[test]
fn until_read_the_optional_entries_are_hidden() {
    for refused in [false, true] {
        let address = Address::Home { project: f::PROJECT };
        let root = if refused { f::root_with_refused_ways(address) } else { f::root_with_ways(address, None) };
        assert_eq!(root.forge().ways(f::PROJECT), None);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            for name in [b"Goals".as_slice(), b"Changes", b"Away"] {
                assert!(f::find(view.tree(), Role::Link, name).is_none());
            }
            assert!(f::find(view.tree(), Role::Link, b"Settings").is_some());
        }
    }
}
#[test]
fn a_page_keeps_its_entry_while_on_it() {
    use d::forge::ways::Ways;
    let address = Address::Changes { project: f::PROJECT };
    for (ways, refused) in [(None, false), (None, true), (Some(Ways::STOCK), false)] {
        let root =
            if refused { f::root_with_refused_ways(address.clone()) } else { f::root_with_ways(address.clone(), ways) };
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let links = children(styled(view.tree(), look::RAIL_NAV));
            let link = links.iter().find(|n| n.name.as_deref() == Some(b"Changes")).unwrap();
            assert!(link.states.as_slice().contains(&State::Current));
            let id = link.id;
            assert_eq!(crate::href(&mut view, id).as_deref(), Some(b"/projects/1/changes".as_slice()));
        }
    }
}
#[test]
fn a_hidden_page_keeps_its_address() {
    use d::forge::ways::Ways;
    for (address, name) in [
        (Address::Changes { project: f::PROJECT }, b"Changes".as_slice()),
        (Address::Goals { project: f::PROJECT, filter: GoalsFilter::All }, b"Goals".as_slice()),
    ] {
        let mut root = f::root_with_ways(Address::Home { project: f::PROJECT }, Some(Ways::STOCK));
        assert!(f::find(f::page_view(&root).tree(), Role::Link, name).is_none());
        f::root_event(&mut root, Event::Went { place: d::Place::Address(address.clone()) });
        assert_eq!(root.pages().address(), &address);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(current_names(view.tree(), look::RAIL_NAV).contains(&name));
        }
    }
}
#[test]
fn the_tab_bar_by_goals() {
    use d::forge::ways::Ways;
    for ways in [None, Some(Ways::STOCK), Some(Ways { goals: true, changes: true, ..Ways::STOCK })] {
        let root = f::root_with_ways(Address::Home { project: f::PROJECT }, ways);
        let view = f::browser_page_view(&root);
        let tabs = children(styled(view.tree(), look::TABBAR));
        assert_eq!(
            tabs.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
            if ways.is_some_and(|ways| ways.goals) {
                [b"Home".as_slice(), b"Needs you", b"Goals", b"Chats"]
            } else {
                [b"Home".as_slice(), b"Needs you", b"Chats", b"Settings"]
            }
        );
        assert!(f::find(view.tree(), Role::Link, b"Settings for Shop").is_some());
        let terminal = f::page_view(&root);
        assert!(!terminal.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::TABBAR)));
    }
}
#[test]
fn drafts_offered_reads_drafts() {
    const OTHER_PROJECT: u32 = 17;
    use d::forge::ways::Ways;
    for (ways, offered) in [
        (None, false),
        (Some(Ways::STOCK), false),
        (Some(Ways { drafts: true, ..Ways::STOCK }), true),
        (Some(Ways { goals: true, ..Ways::STOCK }), false),
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
fn no_switch_on_a_fresh_project() {
    let root = f::root_with_ways(Address::Home { project: f::PROJECT }, Some(d::forge::ways::Ways::STOCK));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::AWAY)));
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::AWAY_CHIP)));
    }
}
#[test]
fn the_switch_shows_with_a_goal() {
    let (mut root, me) = f::root_with_watch(None);
    with_goal_fact(&mut root, me, Some(true));
    assert!(!root.away_on(f::PROJECT, f::NOW));
    let view = f::page_view(&root);
    assert_eq!(styled(view.tree(), look::AWAY)[0].name.as_deref(), Some(b"Away".as_slice()));
    let foot = children(styled(view.tree(), look::RAIL_FOOT));
    let at = foot.iter().position(|n| n.styles.as_slice().contains(&look::AWAY)).unwrap();
    assert_eq!(foot[at + 1].name.as_deref(), Some(b"Settings".as_slice()));
}
#[test]
fn the_switch_shows_with_ways_away() {
    let root = f::root_with_ways(
        Address::Home { project: f::PROJECT },
        Some(d::forge::ways::Ways { away: true, ..d::forge::ways::Ways::STOCK }),
    );
    let mut view = f::page_view(&root);
    let node = &styled(view.tree(), look::AWAY)[0];
    assert_eq!(node.role, Role::Button);
    let id = node.id;
    assert_eq!(
        crate::act(&mut view, id, Act::Press),
        Some(d::Action::Pages(pages::Act::Away(pages::away::Act::Toggle { project: f::PROJECT })))
    );
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
    );
    assert_eq!(times.next, Some(until), "the rail requests the expiry wake independently of page times");
    let mut view = f::view_of(f::browser_words());
    crate::build(&mut view, &root, skein_lib::Wall::from_nanos(until.as_nanos() - 1), &mut Queue::uncapped());
    assert_eq!(view.next_change(), Some(until));
    crate::build(&mut view, &root, until, &mut Queue::uncapped());
    assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::AWAY)));
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
        [Role::Link, Role::Disclosure, Role::Button, Role::Slot, Role::Slot, Role::Slot]
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
    let (id, _) = f::terminal_command(view.tree(), &view.limits().tree, id, b"Away (pressed)", b"Settings");
    assert_eq!(
        crate::act(&mut view, id, Act::Press),
        Some(d::Action::Pages(pages::Act::Away(pages::away::Act::Toggle { project: f::PROJECT })))
    );
}
