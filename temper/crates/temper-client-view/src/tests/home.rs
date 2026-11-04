use super::fixtures as f;
use skein_ui::{Act, Role};
use temper_client_domain::{self as d, Address, ChatsFilter};

const UNKNOWN: u32 = 99;

fn printed(view: &crate::View) -> Box<[u8]> {
    let mut print = skein_ui_text::Print::new(&view.limits().tree).expect("terminal bounds");
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    skein_lib::bytes::copy_of(print.text.as_slice())
}

fn main_nodes(tree: &skein_ui::Tree) -> &[skein_ui::Node] {
    let mut main = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::Main {
            let end = index.checked_add(usize::try_from(node.size).expect("main size")).expect("main end");
            main = tree.nodes().get(index..end);
        }
    }
    main.expect("main subtree")
}

fn main_find(tree: &skein_ui::Tree, role: Role, name: &[u8]) -> skein_ui::NodeId {
    let mut found = None;
    for node in main_nodes(tree) {
        if node.role == role && node.name.as_deref() == Some(name) {
            found = Some(node.id);
        }
    }
    found.expect("main control")
}

#[test]
fn project_pages_draw_their_heading_in_both_shells() {
    for (address, heading) in [
        (Address::Home { project: f::PROJECT }, b"You\xe2\x80\x99re set up. Nothing has run yet.".as_slice()),
        (Address::Settings { project: f::PROJECT }, b"Settings for Shop".as_slice()),
    ] {
        let (root, _, _) = f::root_at(address, &f::domain_limits());
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            let main = main_nodes(tree);
            assert!(main.first().expect("main").styles.as_slice().contains(&crate::look::MAIN));
            let _heading = main_find(tree, Role::Heading { level: 1 }, heading);
            assert!(skein_lib::bytes::find(&f::main_text(tree), b"not one of yours").is_none());
        }
        let view = f::page_view(&root);
        let printed = printed(&view);
        assert!(skein_lib::bytes::find(&printed, heading).is_some());
    }
}

#[test]
fn unlisted_projects_explain_and_link_to_chats() {
    for address in [Address::Home { project: UNKNOWN }, Address::Settings { project: UNKNOWN }] {
        let (root, _, _) = f::root_at(address, &f::domain_limits());
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let main = f::main_text(view.tree());
            assert!(skein_lib::bytes::find(&main, b"Project 99 is not one of yours.").is_some());
            let link = main_find(view.tree(), Role::Link, b"Chats");
            assert_eq!(
                crate::act(&mut view, link, Act::Press),
                Some(d::Action::Pages(d::pages::Act::Go {
                    address: Address::Chats { filter: ChatsFilter::All, start: false },
                }))
            );
        }
        let view = f::page_view(&root);
        assert!(skein_lib::bytes::find(&printed(&view), b"Project 99 is not one of yours.").is_some());
    }
}

fn settings_root() -> d::Domain {
    f::root_at(Address::Settings { project: f::PROJECT }, &f::domain_limits()).0
}

#[test]
fn settings_lists_links_then_sign_out_in_both_shells() {
    let root = settings_root();
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let nodes = main_nodes(view.tree());
        assert_eq!(nodes[0].styles.as_slice(), &[crate::look::MAIN, crate::look::SETTINGS]);
        assert_eq!(crate::look::NAMES.style(crate::look::SETTINGS), Some(b"temper-settings".as_slice()));
        let mut at = 1;
        let mut list = None;
        for expected in [Role::Heading { level: 1 }, Role::List, Role::Button] {
            let node = &nodes[at];
            assert_eq!(node.role, expected);
            if expected == Role::List {
                list = Some(subtree(view.tree(), node.id));
            }
            at += usize::try_from(node.size).unwrap();
        }
        assert_eq!(at, nodes.len());
        let list = list.unwrap();
        let mut at = 1;
        for name in [b"Forges".as_slice(), b"Accounts", b"Adopt a repository"] {
            assert_eq!(list[at].role, Role::Item);
            assert_eq!(list[at + 1].role, Role::Link);
            assert_eq!(list[at + 1].name.as_deref(), Some(name));
            at += usize::try_from(list[at].size).unwrap();
        }
        assert_eq!(at, list.len());
        for (name, address, href) in [
            (b"Forges".as_slice(), Address::Forges, b"/forges".as_slice()),
            (b"Accounts", Address::Accounts, b"/accounts"),
            (b"Adopt a repository", Address::Adopt { project: f::PROJECT }, b"/projects/1/adopt"),
        ] {
            let link = main_find(view.tree(), Role::Link, name);
            assert!(!subtree(view.tree(), link)[0].states.as_slice().contains(&skein_ui::State::Current));
            assert_eq!(crate::href(&mut view, link).as_deref(), Some(href));
            assert_eq!(crate::act(&mut view, link, Act::Press), Some(d::Action::Pages(d::pages::Act::Go { address })));
        }
        let button = main_find(view.tree(), Role::Button, b"Sign out");
        assert_eq!(crate::act(&mut view, button, Act::Press), Some(d::Action::Pages(d::pages::Act::SignOut)));
    }
}

#[test]
fn settings_links_carry_the_forge_and_account_marks_in_both_shells() {
    use jig_client::contract::AccountState;
    use temper_client_protocol_wire::v1 as w;
    for (connection, account, style, forge_words, account_words) in [
        (
            w::Connection::Connecting,
            AccountState::Later { until: Some(f::NOW) },
            crate::look::NAV_LIMITED,
            b"a forge is setting up".as_slice(),
            b"an account is limited".as_slice(),
        ),
        (
            w::Connection::Stopped(w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::default() }).unwrap()),
            AccountState::SignInAgain,
            crate::look::NAV_NEEDS,
            b"a forge needs you",
            b"an account needs you",
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
            for (name, expected) in [(b"Forges".as_slice(), forge_words), (b"Accounts", account_words)] {
                let link = main_find(view.tree(), Role::Link, name);
                let mut marks = 0_u32;
                for node in subtree(view.tree(), link) {
                    if node.styles.as_slice().contains(&style) {
                        marks += 1;
                        assert_eq!(node.role, Role::Text);
                        assert_eq!(node.name.as_deref(), Some(expected));
                        assert_eq!(f::nodes_text(core::slice::from_ref(node)).as_ref(), expected);
                    }
                }
                assert_eq!(marks, 1, "one mark on each settings link");
            }
        }
        let text = printed(&f::page_view(&root));
        for expected in [forge_words, account_words] {
            assert!(skein_lib::bytes::find(&text, expected).is_some());
        }
    }
}

#[test]
fn settings_terminal_numbers_each_link_and_sign_out() {
    let root = settings_root();
    let mut view = f::page_view(&root);
    for (role, name, action) in [
        (Role::Link, b"Forges".as_slice(), d::pages::Act::Go { address: Address::Forges }),
        (Role::Link, b"Accounts", d::pages::Act::Go { address: Address::Accounts }),
        (Role::Link, b"Adopt a repository", d::pages::Act::Go { address: Address::Adopt { project: f::PROJECT } }),
        (Role::Button, b"Sign out", d::pages::Act::SignOut),
    ] {
        let control = main_find(view.tree(), role, name);
        let (node, act) = f::terminal_command(view.tree(), &view.limits().tree, control, name, b"Settings for Shop");
        assert_eq!(crate::act(&mut view, node, act), Some(d::Action::Pages(action)));
    }
}

#[test]
fn settings_lost_sign_out_alert_offers_sign_out_again() {
    let mut root = settings_root();
    f::root_event(&mut root, d::Event::Act { action: d::Action::Pages(d::pages::Act::SignOut) });
    f::root_event(&mut root, d::Event::Client { event: jig_client::Event::SignOutLost });
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let _alert = main_find(view.tree(), Role::Alert, b"Sign out did not reach the engine.");
        let button = main_find(view.tree(), Role::Button, b"Sign out");
        assert_eq!(crate::act(&mut view, button, Act::Press), Some(d::Action::Pages(d::pages::Act::SignOut)));
        assert!(skein_lib::bytes::find(&printed(&view), b"Sign out did not reach the engine.").is_some());
    }
}

fn subtree(tree: &skein_ui::Tree, id: skein_ui::NodeId) -> &[skein_ui::Node] {
    for (at, node) in tree.nodes().iter().enumerate() {
        if node.id == id {
            let end = at.checked_add(usize::try_from(node.size).expect("subtree size")).expect("subtree end");
            return &tree.nodes()[at..end];
        }
    }
    panic!("subtree root exists")
}

use jig_client::contract as c;
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn drive(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        event,
        &mut out,
    );
    out
}
fn deliver(root: &mut d::Domain, watch: Token, value: c::Delivery) {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
            },
        },
    );
}
fn home_root(rows: List<w::ForgeRow>, configured: bool, chat: bool) -> (d::Domain, Token) {
    home_with_needs(rows, configured, chat, None)
}
struct NeedsFixture<'a> {
    objects: &'a [c::ObjectKey],
    cut: bool,
    read: bool,
}
fn home_with_needs(
    rows: List<w::ForgeRow>,
    configured: bool,
    chat: bool,
    needs: Option<NeedsFixture<'_>>,
) -> (d::Domain, Token) {
    home_documents(rows, configured, chat, needs, None)
}
struct ChatFixture {
    live: c::Tasks,
    ended: c::TaskPage,
}
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn home_documents(
    rows: List<w::ForgeRow>,
    configured: bool,
    chat: bool,
    needs: Option<NeedsFixture<'_>>,
    chats: Option<ChatFixture>,
) -> (d::Domain, Token) {
    let (mut root, me_watch, _, _, accounts, _) = f::root_at_with_forges(
        Address::Settings { project: f::PROJECT },
        &f::domain_limits(),
        b"Shop",
        None,
        None,
        Some(rows),
    );
    let template_root = f::chat::messaging_root();
    let template_me = template_root.pages().me(template_root.client()).unwrap();
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let member = me.projects.get_mut(0).unwrap();
    member.role = c::HeldRole::Owner;
    member.templates = template_me.projects.get(0).unwrap().templates.clone();
    member.models = List::with_capacity(1);
    member
        .models
        .push(c::MeModel {
            name: c::Text::new(b"gpt-6-luna".as_slice().into(), 32).unwrap(),
            provider: Some(17),
            description: None,
            prices: c::ModelPrices { input: 0, cached: 0, cache_write: None, output: 0, unit: 1 },
            window: 200_000,
            output: 16_384,
            efforts: List::with_capacity(0),
        })
        .unwrap();
    f::deliver_me(&mut root, me_watch, me);
    let mut out = drive(&mut root, d::Event::Went { place: d::Place::Address(Address::Home { project: f::PROJECT }) });
    let mut settings = None;
    let mut live = None;
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(subject) },
                },
        } = request
        {
            match subject {
                c::Subject::Settings => settings = Some(*watch),
                c::Subject::Tasks { .. } => live = Some(*watch),
                c::Subject::Me
                | c::Subject::Task { .. }
                | c::Subject::Tree { .. }
                | c::Subject::Chips { .. }
                | c::Subject::Inbox { .. }
                | c::Subject::Run { .. }
                | c::Subject::Deployment
                | c::Subject::Accounts => {}
            }
        }
    }
    let mut lines = List::with_capacity(1);
    lines
        .push(c::SettingLine {
            setting: c::Setting::Rules,
            name: None,
            follows: if configured { c::SettingFollows::Change } else { c::SettingFollows::Default },
            last: None,
            application: None,
        })
        .unwrap();
    deliver(
        &mut root,
        settings.expect("Home settings"),
        c::Delivery::Settings(Box::new(c::SettingsDocument {
            newest: 0,
            settings: lines,
            held: List::with_capacity(0),
            pending: List::with_capacity(0),
        })),
    );
    let mut rows = List::with_capacity(1);
    if chat {
        rows.push(super::chats::line()).unwrap();
    }
    let mut refreshed = drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: live.expect("Home chats"),
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(
                        chats.as_ref().map_or(c::Tasks { rows, more: false }, |chats| chats.live.clone()),
                    ))),
                },
            },
        },
    );
    while let Some(request) = refreshed.pop() {
        out.push(request);
    }
    settle_home(&mut root, &mut out, needs.as_ref(), chats.as_ref());
    (root, accounts.unwrap())
}
fn read_done(root: &mut d::Domain, read: Token, value: c::ReadValue) -> Queue<d::Request> {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(c::ReadResult::Value(value)) },
            },
        },
    )
}
fn home_need_page(object: c::ObjectKey) -> c::ReadValue {
    let mut task = f::task::document();
    task.chip.title = f::task::document_words(b"Add a README");
    match object {
        c::ObjectKey::Question { task: number, message } => {
            task.chip.task = number;
            if message == 7 {
                task.chip.title = f::task::document_words(b"");
            }
            c::ReadValue::Question(Box::new(c::Question {
                task: task.chip,
                from: None,
                message,
                words: f::task::document_words(b"Which license?"),
                since: skein_lib::Wall::EPOCH,
                acts: List::with_capacity(0),
                ending: None,
            }))
        }
        c::ObjectKey::PersonTask(_) => c::ReadValue::PersonTask(Box::new(c::PersonTask {
            task: task.chip,
            revision: task.spec_revision,
            requester: task.requester,
            spec: task.spec,
            address: c::PersonAddress::Party(73),
            taken_by: None,
            since: Some(skein_lib::Wall::EPOCH),
            stall: None,
            deadline: None,
            contract: task.result_contract,
            results: List::with_capacity(0),
            cut: false,
            acts: List::with_capacity(0),
        })),
        c::ObjectKey::Escalation(_) => {
            c::ReadValue::Escalation(Box::new(c::Escalation { task: Box::new(task), acts: List::with_capacity(0) }))
        }
        c::ObjectKey::Proposal(_) => super::cards::batch_proposal(),
        c::ObjectKey::Task(_) | c::ObjectKey::Result(_) | c::ObjectKey::Note { .. } => {
            c::ReadValue::Task(Box::new(task))
        }
    }
}
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn settle_home(
    root: &mut d::Domain,
    out: &mut Queue<d::Request>,
    needs: Option<&NeedsFixture<'_>>,
    chats: Option<&ChatFixture>,
) {
    while let Some(request) = out.pop() {
        let d::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        let mut more = match down {
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) }
                if needs.is_some() =>
            {
                let value =
                    super::documents::inbox(List::with_capacity(0), 0, c::Count { value: 0, limited: false }, 0, false);
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch,
                                event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(value))),
                            },
                        },
                    },
                )
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { group, .. }) } => {
                let Some(needs) = needs else { continue };
                let mut rows = List::with_capacity(u32::try_from(needs.objects.len()).unwrap());
                if group == c::InboxReadGroup::Needs {
                    for object in needs.objects {
                        rows.push(super::documents::entry(*object, f::PROJECT, 1, f::NOW, None)).unwrap();
                    }
                }
                read_done(
                    root,
                    read,
                    c::ReadValue::Inbox(Box::new(c::InboxPage {
                        rows,
                        cut: group == c::InboxReadGroup::Needs && needs.cut,
                        before: None,
                        after: None,
                        selection: None,
                    })),
                )
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { task, message }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::Question { task, message }))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Proposal { proposal }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::Proposal(proposal)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Escalation { task }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::Escalation(task)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::PersonTask { task }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::PersonTask(task)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { .. }) } => read_done(
                root,
                read,
                c::ReadValue::Tasks(Box::new(chats.map_or(
                    c::TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: c::EndedGroup::Rows { rows: List::with_capacity(0), after: None, cut: false },
                    },
                    |chats| chats.ended.clone(),
                ))),
            ),
            jig_client::Down::Save { .. }
            | jig_client::Down::Erase { .. }
            | jig_client::Down::Send { .. }
            | jig_client::Down::Open { .. }
            | jig_client::Down::Continue { .. }
            | jig_client::Down::Close { .. }
            | jig_client::Down::Read { .. } => continue,
        };
        while let Some(request) = more.pop() {
            out.push(request);
        }
    }
}
fn forge_rows(in_project: bool) -> List<w::ForgeRow> {
    let mut rows = super::connect_forge::first_rows(super::connect_forge::connected(), super::connect_forge::made());
    if in_project {
        let mut parts = rows.get(0).unwrap().clone().into_parts();
        parts.repositories = List::with_capacity(1);
        parts
            .repositories
            .push(
                w::Adoption::new(
                    &w::CEILINGS,
                    w::AdoptionParts {
                        owner: b"work".as_slice().into(),
                        name: b"hello".as_slice().into(),
                        project: f::PROJECT,
                    },
                )
                .unwrap(),
            )
            .unwrap();
        *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    }
    rows
}
fn contains(text: &[u8], words: &str) {
    assert!(skein_lib::bytes::find(text, words.as_bytes()).is_some(), "missing {words}: {text:?}");
}
fn absent(text: &[u8], words: &str) {
    assert!(skein_lib::bytes::find(text, words.as_bytes()).is_none(), "unexpected {words}");
}
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
fn styled(tree: &skein_ui::Tree, style: skein_ui::Style) -> &skein_ui::Node {
    main_nodes(tree).iter().find(|node| node.styles.as_slice().contains(&style)).expect("styled Home node")
}
fn children(nodes: &[skein_ui::Node]) -> Box<[&skein_ui::Node]> {
    let mut children = List::with_capacity(u32::try_from(nodes.len()).unwrap());
    let mut at = 1;
    while at < nodes.len() {
        children.push(&nodes[at]).unwrap();
        at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    }
    children.into_boxed()
}
fn assert_action(view: &mut crate::View, role: Role, name: &[u8], act: d::pages::Act) {
    let id = main_find(view.tree(), role, name);
    let Some(d::Event::Act { action }) = crate::event(view, id, Act::Press) else { panic!("page act event") };
    assert_eq!(action, d::Action::Pages(act.clone()));
    assert_eq!(crate::act(view, id, Act::Press), Some(d::Action::Pages(act)));
}

#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
fn home_counts_steps_and_removes_done_acts_in_both_shells() {
    for configured in [false, true] {
        for repository in [false, true] {
            for chat in [false, true] {
                let (root, _) =
                    home_root(if repository { forge_rows(true) } else { List::with_capacity(0) }, configured, chat);
                let left = u8::from(!configured) + u8::from(!repository) + u8::from(!chat);
                let headline = match left {
                    3 => "You’re set up. Nothing has run yet.",
                    2 => "Two things left to begin with.",
                    1 => "One thing left to begin with.",
                    0 => "Shop",
                    _ => unreachable!(),
                };
                for view in [f::page_view(&root), f::browser_page_view(&root)] {
                    main_find(view.tree(), Role::Heading { level: 1 }, headline.as_bytes());
                    assert_eq!(
                        main_nodes(view.tree())
                            .iter()
                            .filter(|node| node.styles.as_slice().contains(&crate::look::HOME_DONE))
                            .count(),
                        if left == 0 { 0 } else { usize::from(3 - left) }
                    );
                    for (role, name, done) in
                        [(Role::Button, b"Start".as_slice(), configured), (Role::Link, b"New chat", chat)]
                    {
                        assert_eq!(
                            main_nodes(view.tree())
                                .iter()
                                .any(|node| node.role == role && node.name.as_deref() == Some(name)),
                            !done
                        );
                    }
                    let main = f::main_text(view.tree());
                    if left > 0 {
                        contains(
                            &main,
                            "In your own words: how you plan, review and land work. temper proposes how it will run that, and nothing changes until you accept.",
                        );
                        contains(&main, "Ask a question or ask for a change.");
                        if repository {
                            contains(&main, "work/hello is in Shop");
                            contains(
                                &main,
                                "temper can change its code now; checked changes land on main through temper's merge queue.",
                            );
                        }
                    }
                }
                let terminal = printed(&f::page_view(&root));
                assert!(terminal.strip_prefix(b"== ").unwrap().starts_with(headline.as_bytes()));
                contains(&terminal, "Connected");
                for words in ["To begin", "1. ", "2. ", "3. "] {
                    if left > 0 {
                        contains(&terminal, words);
                    } else {
                        absent(&terminal, words);
                    }
                }
            }
        }
    }
}

#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
fn adopting(root: &mut d::Domain, refused: Option<w::ForgeReason>) {
    let mut out = drive(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::AdoptFirst) });
    let mut stored = drive(root, d::Event::Client { event: jig_client::Event::Stored });
    while let Some(request) = stored.pop() {
        out.push(request);
    }
    if let Some(reason) = refused {
        let attempt = out
            .iter()
            .find_map(|request| match request {
                d::Request::Unlinked {
                    request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
                } => Some(*attempt),
                d::Request::Client { .. }
                | d::Request::Unlinked { .. }
                | d::Request::Show { .. }
                | d::Request::Leave { .. }
                | d::Request::SignOut
                | d::Request::Dropped { .. }
                | d::Request::Fact { .. } => None,
            })
            .expect("adoption request");
        let answer = w::Answer::new(
            &w::CEILINGS,
            w::AnswerParts {
                outcome: w::Outcome::ForgeRefused(
                    w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason }).unwrap(),
                ),
            },
        )
        .unwrap();
        drive(
            root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        why: None,
                        answer: Some(jig_client::Answered::Family(jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::answer(&answer),
                        })),
                    },
                },
            },
        );
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
#[expect(clippy::too_many_lines, reason = "the repository state matrix also checks every shared forge refusal")]
fn home_repository_states_keep_their_words_and_events() {
    for state in ["offered", "adopting", "refused", "none", "github"] {
        let mut rows = if state == "none" || state == "github" { List::with_capacity(1) } else { forge_rows(false) };
        if state == "github" {
            let mut parts = super::connect_forge::row(
                2,
                w::Kind::Github,
                w::Mode::Managed,
                super::connect_forge::connected(),
                w::Offer::NotOffered,
            )
            .into_parts();
            parts.installations = List::with_capacity(1);
            parts
                .installations
                .push(
                    w::Installation::new(
                        &w::CEILINGS,
                        w::InstallationParts {
                            id: 1,
                            account: Some(
                                w::Account::new(
                                    &w::CEILINGS,
                                    w::AccountParts { id: 73, login: b"octo".as_slice().into() },
                                )
                                .unwrap(),
                            ),
                            suspended: false,
                            stray: false,
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
            rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
        }
        let (mut root, _) = home_root(rows, false, false);
        if state == "adopting" || state == "refused" {
            adopting(&mut root, if state == "refused" { Some(w::ForgeReason::HomeHeld) } else { None });
        }
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let words = f::main_text(view.tree());
            contains(&words, "Shop has no repository yet.");
            if state == "adopting" {
                contains(&words, "Adopting work/hello into Shop");
                contains(&words, "Chats can change its code once it is in.");
                assert!(!main_nodes(view.tree()).iter().any(|node| node.name.as_deref() == Some(b"Adopt work/hello")));
            } else if state == "offered" || state == "refused" {
                contains(
                    &words,
                    "work/hello, made on your Forgejo at set up, is in no project yet. Until one is in, chats talk and plan but change no code.",
                );
                assert_action(&mut view, Role::Button, b"Adopt work/hello", d::pages::Act::AdoptFirst);
                assert_action(
                    &mut view,
                    Role::Link,
                    "Adopt another…".as_bytes(),
                    d::pages::Act::Go { address: Address::Adopt { project: f::PROJECT } },
                );
                if state == "refused" {
                    let mut b = skein_ui::Builder::new(&f::tree_limits());
                    crate::forge::page::outcome(
                        &mut b,
                        (b"work", b"hello", b"temper/", b"3"),
                        &d::forge::Answer::ForgeRefused(d::forge::Reason::HomeHeld),
                        &f::words(),
                        512,
                    );
                    let expected = f::text(&b.finish());
                    assert!(skein_lib::bytes::find(&words, &expected).is_some());
                }
            } else {
                contains(&words, "Until one is in, chats talk and plan but change no code.");
                if state == "github" {
                    contains(&words, "temper’s App reaches its repositories on octo.");
                }
                assert_action(
                    &mut view,
                    Role::Link,
                    b"Adopt",
                    d::pages::Act::Go { address: Address::Adopt { project: f::PROJECT } },
                );
            }
            contains(&words, "Project Shop · no repository yet");
            if state != "none" {
                assert_action(&mut view, Role::Link, b"Forges", d::pages::Act::Go { address: Address::Forges });
            }
            assert_action(&mut view, Role::Button, b"Start", d::pages::Act::FirstChat);
            assert_action(
                &mut view,
                Role::Link,
                b"New chat",
                d::pages::Act::Go { address: Address::Chats { filter: ChatsFilter::All, start: true } },
            );
        }
        let terminal = printed(&f::page_view(&root));
        contains(&terminal, if state == "adopting" { "Adopting work/hello into Shop" } else { "Adopt a repository" });
    }
    for (wire_reason, reason) in [
        (w::ForgeReason::NoSuchForge, d::forge::Reason::NoSuchForge),
        (w::ForgeReason::NotFound, d::forge::Reason::NotFound),
        (w::ForgeReason::NoAccess, d::forge::Reason::NoAccess),
        (w::ForgeReason::Refused, d::forge::Reason::Refused),
        (w::ForgeReason::AnotherProject, d::forge::Reason::AnotherProject),
        (w::ForgeReason::HomeHeld, d::forge::Reason::HomeHeld),
        (w::ForgeReason::PrefixOccupied, d::forge::Reason::PrefixOccupied),
        (w::ForgeReason::InFlight, d::forge::Reason::InFlight),
        (w::ForgeReason::TooLarge, d::forge::Reason::TooLarge),
        (w::ForgeReason::Unreachable, d::forge::Reason::Unreachable),
        (w::ForgeReason::InvalidAnswer, d::forge::Reason::InvalidAnswer),
    ] {
        let (mut root, _) = home_root(forge_rows(false), false, false);
        adopting(&mut root, Some(wire_reason));
        let mut b = skein_ui::Builder::new(&f::tree_limits());
        crate::forge::page::outcome(
            &mut b,
            (b"work", b"hello", b"temper/", b"3"),
            &d::forge::Answer::ForgeRefused(reason),
            &f::words(),
            512,
        );
        let expected = f::text(&b.finish());
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(skein_lib::bytes::find(&f::main_text(view.tree()), &expected).is_some());
        }
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
fn home_outline_matches_the_requested_main_and_terminal_lines_join() {
    let (mut root, accounts) = home_root(forge_rows(true), false, false);
    super::accounts::deliver(
        &mut root,
        accounts,
        super::accounts::card_document(b"openai", b"", c::AccountKind::SignIn),
    );
    let mut view = f::view_of(f::browser_words());
    let mut sign_in = crate::forge_sign_in::Forges::new();
    sign_in.read("3 forgejo http://forge.example:3000 owned");
    view.forge_sign_in(sign_in);
    f::build(&mut view, &root);
    let tree = view.tree();
    let main = main_nodes(tree);
    contains(&f::main_text(tree), "ChatGPT plan · gpt-6-luna");
    assert_eq!(main[0].styles.as_slice(), &[crate::look::MAIN, crate::look::HOME]);
    let top = children(main);
    assert_eq!(top.len(), 3);
    for (node, (style, role)) in top.iter().zip([
        (crate::look::HOME_INTRO, Role::Slot),
        (crate::look::HOME_BEGIN, Role::Region),
        (crate::look::HOME_CONNECTED, Role::Region),
    ]) {
        assert_eq!(node.styles.as_slice(), &[style]);
        assert_eq!(node.role, role);
    }
    let intro = children(subtree(tree, top[0].id));
    assert_eq!(intro.len(), 3);
    for (node, role) in intro.iter().zip([Role::Paragraph, Role::Heading { level: 1 }, Role::Paragraph]) {
        assert_eq!(node.role, role);
    }
    assert_eq!(intro[0].styles.as_slice(), &[crate::look::HOME_KICKER]);
    assert_eq!(intro[2].styles.as_slice(), &[crate::look::HOME_LEDE]);
    contains(
        &f::node_text(tree, tree.nodes().iter().position(|node| node.id == intro[2].id).unwrap()),
        "Your agents run on your ChatGPT plan. Forgejo runs on this machine.",
    );
    let begin = children(subtree(tree, top[1].id));
    assert_eq!(begin.len(), 2);
    assert_eq!(begin[0].role, Role::Heading { level: 2 });
    assert_eq!(begin[1].role, Role::List);
    assert_eq!(begin[1].styles.as_slice(), &[crate::look::HOME_STEPS]);
    let steps = children(subtree(tree, begin[1].id));
    assert_eq!(steps.len(), 3);
    for (step, done) in steps.iter().zip([false, true, false]) {
        assert_eq!(step.role, Role::Item);
        assert!(step.styles.as_slice().contains(&crate::look::HOME_STEP));
        assert_eq!(step.styles.as_slice().contains(&crate::look::HOME_DONE), done);
        let parts = children(subtree(tree, step.id));
        assert_eq!(parts[0].role, Role::Heading { level: 3 });
        assert_eq!(parts[1].role, Role::Paragraph);
        assert_eq!(parts.len(), if done { 2 } else { 3 });
        if !done {
            assert_eq!(parts[2].styles.as_slice(), &[crate::look::HOME_ACTS]);
        }
    }
    let connected = children(subtree(tree, top[2].id));
    assert_eq!(connected[1].styles.as_slice(), &[crate::look::HOME_LINES]);
    let lines = children(subtree(tree, connected[1].id));
    assert_eq!(lines.len(), 3);
    for line in &lines {
        let parts = children(subtree(tree, line.id));
        assert_eq!(parts[0].role, Role::Paragraph);
        assert_eq!(parts.len(), 2);
        let muted = subtree(tree, parts[0].id)
            .iter()
            .find(|node| node.styles.as_slice().contains(&crate::look::HOME_MUTED))
            .unwrap();
        assert_eq!(muted.role, Role::Text);
        assert_eq!(muted.size, 1);
        assert_eq!(muted.text.len(), 1);
    }
    let open = main_find(tree, Role::Link, "Open Forgejo ↗".as_bytes());
    assert_eq!(
        tree.nodes().iter().find(|node| node.id == open).unwrap().external.as_ref().unwrap().as_ref(),
        b"http://forge.example:3000/user/oauth2/temper"
    );
    assert_action(&mut view, Role::Link, b"Accounts", d::pages::Act::Go { address: Address::Accounts });
    let mut terminal = f::view();
    let mut sign_in = crate::forge_sign_in::Forges::new();
    sign_in.read("3 forgejo http://forge.example:3000 owned");
    terminal.forge_sign_in(sign_in);
    f::build(&mut terminal, &root);
    let printed = printed(&terminal);
    contains(&printed, "Forgejo on this machine · connected as temper · work/hello, in Shop\n");
    contains(&printed, "Project Shop · work/hello\n");
    absent(&printed, "Change model");
}

#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
fn home_rename_stays_in_connected_and_is_only_offered_to_owners() {
    let (mut root, _) = home_root(forge_rows(true), false, false);
    let mut view = f::browser_page_view(&root);
    assert_action(&mut view, Role::Button, b"Rename", d::pages::Act::Switcher(d::pages::switcher::Act::RenameHere));
    drive(
        &mut root,
        d::Event::Act { action: d::Action::Pages(d::pages::Act::Switcher(d::pages::switcher::Act::RenameHere)) },
    );
    assert!(root.pages().switcher().renaming_here());
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let field = main_find(view.tree(), Role::TextField, b"Rename");
        assert!(
            subtree(view.tree(), styled(view.tree(), crate::look::HOME_CONNECTED).id)
                .iter()
                .any(|node| node.id == field)
        );
        assert_eq!(
            view.tree()
                .nodes()
                .iter()
                .filter(|node| node.role == Role::TextField && node.name.as_deref() == Some(b"Rename"))
                .count(),
            1
        );
        let Some(d::Event::Act { action }) =
            crate::event(&view, field, Act::Input { value: b"Renamed".as_slice().into() })
        else {
            panic!("rename input event")
        };
        assert_eq!(
            action,
            d::Action::Pages(d::pages::Act::Switcher(d::pages::switcher::Act::RenameWords {
                words: b"Renamed".as_slice().into()
            }))
        );
        assert_action(&mut view, Role::Button, b"Save", d::pages::Act::Switcher(d::pages::switcher::Act::Save));
        assert_action(&mut view, Role::Button, b"Cancel", d::pages::Act::Switcher(d::pages::switcher::Act::Cancel));
    }
    drive(&mut root, d::Event::Went { place: d::Place::Address(Address::Settings { project: f::PROJECT }) });
    assert!(!root.pages().switcher().renaming_here());
    assert_eq!(root.pages().switcher().renaming(), None);
    let (mut other, me_watch, _) = f::root_at(Address::Home { project: f::PROJECT }, &f::domain_limits());
    let mut me = other.pages().me(other.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().role = c::HeldRole::Member;
    f::deliver_me(&mut other, me_watch, me);
    drive(
        &mut other,
        d::Event::Act { action: d::Action::Pages(d::pages::Act::Switcher(d::pages::switcher::Act::RenameHere)) },
    );
    assert!(!other.pages().switcher().renaming_here());
    assert_eq!(other.pages().switcher().renaming(), None);
    for view in [f::page_view(&other), f::browser_page_view(&other)] {
        assert!(
            !main_nodes(view.tree()).iter().any(|node| node.styles.as_slice().contains(&crate::look::PROJECT_RENAME))
        );
    }
}

#[test]
fn home_kicker_tracks_the_current_day_and_minute() {
    let (root, _) = home_root(List::with_capacity(0), false, false);
    let mut view = f::browser_page_view(&root);
    let kicker = styled(view.tree(), crate::look::HOME_KICKER).name.as_deref().unwrap();
    contains(kicker, "Thursday 1 January");
    contains(kicker, "00:00");
    absent(kicker, "Shop");
    assert_eq!(view.next_change(), Some(skein_lib::Wall::from_nanos(60_000_000_000)));
    crate::build(&mut view, &root, skein_lib::Wall::from_nanos(60_000_000_000), &mut Queue::uncapped());
    contains(styled(view.tree(), crate::look::HOME_KICKER).name.as_deref().unwrap(), "00:01");
}

#[test]
fn home_unknown_and_ended_forges_draw_only_known_facts() {
    let (mut root, _, _, _, _, watch) =
        f::root_at_with_forges(Address::Home { project: f::PROJECT }, &f::domain_limits(), b"Shop", None, None, None);
    // An ended watch cannot claim that the project has no repository.
    let watch = watch.expect("forges watch");
    drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::NotAllowed } },
        },
    );
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let text = f::main_text(view.tree());
        contains(&text, "Until one is in, chats talk and plan but change no code.");
        absent(&text, "Shop has no repository yet.");
        absent(&text, "connected as");
        assert_action(
            &mut view,
            Role::Link,
            b"Adopt",
            d::pages::Act::Go { address: Address::Adopt { project: f::PROJECT } },
        );
    }
    let mut root = d::unlinked(&f::domain_limits(), f::SEED);
    drive(
        &mut root,
        d::Event::Start { address: Address::Home { project: f::PROJECT }, kept: List::with_capacity(0), with: None },
    );
    for event in [
        jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
        jig_client::Event::SignIn { with: jig_client::With::Transport },
        jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    ] {
        drive(&mut root, d::Event::Client { event });
    }
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let text = f::main_text(view.tree());
        absent(&text, "Adopt a repository");
        absent(&text, "Until one is in");
        absent(&text, "has no repository yet");
        contains(&text, "Nothing has run yet.");
    }
}

// One featured need plus six others proves Home's five-row limit on both sides.
const QUESTIONS: [c::ObjectKey; 7] = [
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 1 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 2 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 3 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 4 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 5 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 6 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 7 },
];
fn everyday(beginning: bool, objects: &[c::ObjectKey], cut: bool, read: bool) -> d::Domain {
    home_with_needs(forge_rows(true), !beginning, true, Some(NeedsFixture { objects, cut, read })).0
}
fn inbox_act() -> d::pages::Act {
    d::pages::Act::Go {
        address: Address::Inbox { filter: d::address::Filter { project: Some(f::PROJECT), kind: None, goal: None } },
    }
}
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn section_styles(tree: &skein_ui::Tree) -> Box<[skein_ui::Style]> {
    children(main_nodes(tree)).iter().map(|node| node.styles.as_slice()[0]).collect()
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn everyday_headlines_count_known_needs_and_mark_cut_reads() {
    for (objects, cut, headline) in [
        (&QUESTIONS[..0], false, "Nothing needs you."),
        (&QUESTIONS[..1], false, "One thing needs you."),
        (&QUESTIONS[..3], false, "3 things need you."),
        (&QUESTIONS[..1], true, "1+ things need you."),
        (&QUESTIONS[..3], true, "3+ things need you."),
    ] {
        let root = everyday(false, objects, cut, true);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            main_find(view.tree(), Role::Heading { level: 1 }, headline.as_bytes());
            assert_eq!(
                section_styles(view.tree()).as_ref(),
                [
                    crate::look::HOME_INTRO,
                    crate::look::HOME_NEEDS,
                    crate::look::HOME_CHATS,
                    crate::look::HOME_CONNECTED
                ]
            );
            assert!(
                !main_nodes(view.tree()).iter().any(|node| node.styles.as_slice().contains(&crate::look::HOME_LEDE))
            );
            if objects.is_empty() {
                main_find(
                    view.tree(),
                    Role::Paragraph,
                    b"Nothing needs you. Work carries on, and this page tells you what changed when you come back.",
                );
                let heading = main_find(view.tree(), Role::Heading { level: 2 }, b"Needs you");
                assert!(!subtree(view.tree(), heading)[0].styles.as_slice().contains(&crate::look::HOME_SIGNAL));
            }
        }
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn first_need_keeps_card_order_navigation_and_terminal_words() {
    for beginning in [false, true] {
        let root = everyday(beginning, &QUESTIONS[..1], false, true);
        for (mut view, browser) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
            let tree = view.tree();
            let top = section_styles(tree);
            let expected: &[skein_ui::Style] = if beginning {
                &[
                    crate::look::HOME_INTRO,
                    crate::look::HOME_BEGIN,
                    crate::look::HOME_NEEDS,
                    crate::look::HOME_CHATS,
                    crate::look::HOME_CONNECTED,
                ]
            } else {
                &[
                    crate::look::HOME_INTRO,
                    crate::look::HOME_NEEDS,
                    crate::look::HOME_CHATS,
                    crate::look::HOME_CONNECTED,
                ]
            };
            assert_eq!(top.as_ref(), expected);
            let region = styled(tree, crate::look::HOME_NEEDS);
            assert_eq!(region.role, Role::Region);
            assert_eq!(region.name.as_deref(), Some(b"Needs you".as_slice()));
            let parts = children(subtree(tree, region.id));
            assert_eq!(parts.len(), if browser { 3 } else { 2 });
            assert_eq!(parts[0].styles.as_slice(), &[crate::look::HOME_HEAD]);
            assert_eq!(parts[1].styles.as_slice(), &[crate::look::ASK]);
            let heading = children(subtree(tree, parts[0].id));
            assert_eq!(heading.len(), 1);
            assert_eq!(heading[0].name.as_deref(), Some("Needs you · 1".as_bytes()));
            assert_eq!(heading[0].styles.as_slice(), &[crate::look::HOME_SIGNAL]);
            let card = children(subtree(tree, parts[1].id));
            assert_eq!(
                card.iter().map(|node| node.styles.as_slice()[0]).collect::<Box<[_]>>().as_ref(),
                [crate::look::ASK_META, crate::look::ASK_QUESTION, crate::look::ASK_CHOICES]
            );
            assert_eq!(card[0].name.as_deref(), Some("Add a README · a question · under 1 min".as_bytes()));
            assert_eq!(card[1].name.as_deref(), Some(b"Which license?".as_slice()));
            let choices = children(subtree(tree, card[2].id));
            assert_eq!(choices.len(), 1);
            assert_eq!(choices[0].styles.as_slice(), &[crate::look::ASK_QUIET]);
            if browser {
                assert_eq!(parts[2].styles.as_slice(), &[crate::look::HOME_CTA]);
                let spans = children(subtree(tree, parts[2].id));
                assert_eq!(spans.len(), 2);
                assert_eq!(f::nodes_text(subtree(tree, spans[0].id)).as_ref(), b"Add a README: Which license?");
                assert_eq!(spans[1].styles.as_slice(), &[crate::look::HOME_CTA_GO]);
                assert_action(&mut view, Role::Link, b"Add a README: Which license? Answer", inbox_act());
            }
            assert_action(
                &mut view,
                Role::Link,
                b"Add a README",
                d::pages::Act::Go { address: Address::Task { task: f::INBOX_TASK, filter: d::PlanFilter::All } },
            );
            assert_action(&mut view, Role::Button, b"Reply", inbox_act());
            assert_action(
                &mut view,
                Role::Link,
                b"All 1 chats",
                d::pages::Act::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } },
            );
        }
        let print = printed(&f::page_view(&root));
        for words in ["Needs you · 1", "Add a README · a question · under 1 min", "Which license?", "Reply", "Chats"]
        {
            contains(&print, words);
        }
        absent(&print, "Answer");
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn unread_need_has_only_meta_and_unknown_headline() {
    let root = everyday(false, &QUESTIONS[..1], false, false);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        main_find(view.tree(), Role::Heading { level: 1 }, b"Shop");
        let card = children(subtree(view.tree(), styled(view.tree(), crate::look::ASK).id));
        assert_eq!(card.len(), 1);
        assert_eq!(card[0].styles.as_slice(), &[crate::look::ASK_META]);
        assert_eq!(card[0].name.as_deref(), Some("T9 · a question".as_bytes()));
        assert!(!main_nodes(view.tree()).iter().any(|node| node.styles.as_slice().contains(&crate::look::HOME_CTA)));
        assert_action(
            &mut view,
            Role::Link,
            b"T9",
            d::pages::Act::Go { address: Address::Task { task: f::INBOX_TASK, filter: d::PlanFilter::All } },
        );
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn additional_needs_have_five_rows_and_browser_answer_link() {
    // Six total fits exactly; seven total must still draw only five secondary rows.
    for objects in [&QUESTIONS[..3], &QUESTIONS[..6], &QUESTIONS[..7]] {
        let root = everyday(false, objects, false, true);
        for (mut view, browser) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
            let region = styled(view.tree(), crate::look::HOME_NEEDS);
            let rows = subtree(view.tree(), region.id).iter().find(|node| node.role == Role::List).unwrap();
            let items = children(subtree(view.tree(), rows.id));
            assert_eq!(items.len(), (objects.len() - 1).min(5));
            let ids: Box<[_]> = items
                .iter()
                .map(|item| {
                    assert_eq!(item.role, Role::Item);
                    let link = children(subtree(view.tree(), item.id));
                    assert_eq!(link.len(), 1);
                    assert_eq!(link[0].role, Role::Link);
                    let spans = children(subtree(view.tree(), link[0].id));
                    assert_eq!(spans.len(), 2);
                    assert_eq!(spans[1].styles.as_slice(), &[crate::look::HOME_WHEN]);
                    link[0].id
                })
                .collect();
            for id in ids {
                assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Pages(inbox_act())));
            }
            assert_eq!(
                main_nodes(view.tree()).iter().any(|node| node.styles.as_slice().contains(&crate::look::HOME_IN_A_ROW)),
                browser
            );
            if browser {
                assert_action(&mut view, Role::Link, b"Answer in a row", inbox_act());
            }
        }
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn nonquestion_need_opens_inbox_and_keeps_its_lowercase_kind() {
    for (object, kind) in [
        (c::ObjectKey::PersonTask(f::INBOX_TASK), "a person task"),
        (c::ObjectKey::Escalation(f::INBOX_TASK), "held"),
        (
            c::ObjectKey::Proposal(c::ProposalKey {
                proposer: c::Proposer::Task(f::INBOX_TASK),
                number: f::INBOX_QUESTION,
            }),
            "a proposal",
        ),
    ] {
        let root = everyday(false, &[object], false, true);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            contains(&f::main_text(view.tree()), kind);
            assert_action(&mut view, Role::Button, b"Open", inbox_act());
            assert!(
                !main_nodes(view.tree())
                    .iter()
                    .any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Reply"))
            );
        }
    }
}

// Home's span cases build four minutes after its fixture's epoch.
const FOUR_MINUTES: skein_lib::Wall = skein_lib::Wall::from_nanos(240_000_000_000);
fn timed_view(root: &d::Domain, browser: bool) -> crate::View {
    let mut view = f::view_of(if browser { f::browser_words() } else { f::words() });
    crate::build(&mut view, root, FOUR_MINUTES, &mut Queue::uncapped());
    view
}
// Six rows is one beyond Home's five-row cap; a seventh can be an ended chat.
const CHAT_TITLES: [&[u8]; 7] = [b"README", b"Rust", b"Repository", b"Tests", b"Docs", b"Checks", b"Release"];
fn chat_fixture(live: usize, ended: usize, more: bool, after: bool) -> ChatFixture {
    let mut live_rows = List::with_capacity(u32::try_from(live).unwrap());
    let mut closed_rows = List::with_capacity(u32::try_from(ended).unwrap());
    for (index, title) in CHAT_TITLES.iter().take(live.checked_add(ended).unwrap()).enumerate() {
        let mut row = super::chats::line();
        row.chip.task = row.chip.task.checked_add(u64::try_from(index).unwrap()).unwrap();
        row.chip.title = f::task::document_words(title);
        if index == 0 {
            row.waits_on = c::WaitsOn::Party {
                whom: c::Whom::Person(73),
                entries: 1,
                task: row.chip.task,
                kind: c::WaitedFor::Question,
            };
        }
        if index < live {
            live_rows.push(row).unwrap();
        } else {
            row.ended_at = Some(skein_lib::Wall::from_nanos(180_000_000_000));
            closed_rows.push(row).unwrap();
        }
    }
    ChatFixture {
        live: c::Tasks { rows: live_rows, more },
        ended: c::TaskPage {
            live: List::with_capacity(0),
            live_after: None,
            ended: c::EndedGroup::Rows {
                rows: closed_rows,
                after: if after { Some(c::Cursor::new(b"next".as_slice().into(), 32).unwrap()) } else { None },
                cut: false,
            },
        },
    }
}
fn home_chats(chats: ChatFixture) -> d::Domain {
    home_documents(
        forge_rows(true),
        true,
        true,
        Some(NeedsFixture { objects: &[], cut: false, read: true }),
        Some(chats),
    )
    .0
}
#[test]
fn chats_keep_live_then_ended_status_age_names_and_events() {
    for (more, after, name, label) in [
        (false, false, "All 3 chats", "All 3"),
        (true, false, "All 3+ chats", "All 3+"),
        (false, true, "All 3+ chats", "All 3+"),
    ] {
        let root = home_chats(chat_fixture(2, 1, more, after));
        for browser in [false, true] {
            let mut view = timed_view(&root, browser);
            let region = styled(view.tree(), crate::look::HOME_CHATS);
            assert_eq!(region.role, Role::Region);
            assert_eq!(region.name.as_deref(), Some(b"Chats".as_slice()));
            let parts = children(subtree(view.tree(), region.id));
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0].styles.as_slice(), &[crate::look::HOME_HEAD]);
            assert_eq!(parts[1].styles.as_slice(), &[crate::look::HOME_ROWS]);
            let heading = children(subtree(view.tree(), parts[0].id));
            assert_eq!(heading[0].role, Role::Heading { level: 2 });
            assert_eq!(heading[1].name.as_deref(), Some(name.as_bytes()));
            assert_eq!(f::nodes_text(subtree(view.tree(), heading[1].id)).as_ref(), label.as_bytes());
            let rows = children(subtree(view.tree(), parts[1].id));
            assert_eq!(rows.len(), 3);
            for (row, (name, title, when, task)) in rows.iter().zip([
                ("README, waiting on you, 4 min", "README", "waiting on you · 4 min", 9),
                ("Rust, Parked, 4 min", "Rust", "Parked · 4 min", 10),
                ("Repository, closed, 1 min", "Repository", "closed · 1 min", 11),
            ]) {
                assert_eq!(row.role, Role::Item);
                let links = children(subtree(view.tree(), row.id));
                assert_eq!(links.len(), 1);
                assert_eq!(links[0].role, Role::Link);
                assert_eq!(links[0].name.as_deref(), Some(name.as_bytes()));
                let spans = children(subtree(view.tree(), links[0].id));
                assert_eq!(f::nodes_text(subtree(view.tree(), spans[0].id)).as_ref(), title.as_bytes());
                assert_eq!(spans[1].styles.as_slice(), &[crate::look::HOME_WHEN]);
                contains(&f::nodes_text(subtree(view.tree(), spans[1].id)), when);
                let id = links[0].id;
                let Some(d::Event::Act { action }) = crate::event(&view, id, Act::Press) else {
                    panic!("chat navigation");
                };
                assert_eq!(action, d::Action::Pages(d::pages::Act::Go { address: Address::Chat { task } }));
            }
            assert_action(
                &mut view,
                Role::Link,
                name.as_bytes(),
                d::pages::Act::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } },
            );
            if !browser {
                let print = printed(&view);
                contains(&print, "README · waiting on you · 4 min");
                contains(&print, "Repository · closed · 1 min");
            }
        }
    }
}
#[test]
fn chat_rows_stop_at_five_without_changing_the_total() {
    for (live, ended, label) in [(5, 0, "All 5 chats"), (6, 0, "All 6 chats"), (2, 4, "All 6 chats")] {
        let root = home_chats(chat_fixture(live, ended, false, false));
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let region = styled(view.tree(), crate::look::HOME_CHATS);
            let parts = children(subtree(view.tree(), region.id));
            assert_eq!(children(subtree(view.tree(), parts[1].id)).len(), 5);
            main_find(view.tree(), Role::Link, label.as_bytes());
            absent(&f::main_text(view.tree()), "Checks");
        }
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn empty_sections_are_hidden_only_while_to_begin_shows() {
    for shown in [false, true] {
        let root = if shown {
            home_chats(chat_fixture(0, 0, true, false))
        } else {
            home_with_needs(forge_rows(true), false, false, Some(NeedsFixture { objects: &[], cut: false, read: true }))
                .0
        };
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let styles = section_styles(view.tree());
            assert_eq!(styles.contains(&crate::look::HOME_NEEDS), shown);
            assert_eq!(styles.contains(&crate::look::HOME_CHATS), shown);
            if shown {
                main_find(view.tree(), Role::Paragraph, b"No chats yet.");
                assert!(!main_nodes(view.tree()).iter().any(|node| node.role == Role::Link
                    && node.name.as_deref().is_some_and(|name| name.starts_with(b"All "))));
            }
        }
    }
}

#[test]
fn read_need_without_a_title_links_by_task_number() {
    let root = everyday(false, &QUESTIONS[6..], false, true);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_action(
            &mut view,
            Role::Link,
            b"T9",
            d::pages::Act::Go { address: Address::Task { task: f::INBOX_TASK, filter: d::PlanFilter::All } },
        );
        assert_action(&mut view, Role::Button, b"Reply", inbox_act());
    }
}
#[test]
fn phone_answer_name_holds_the_joined_title_and_words() {
    // The fixture's ordinary Name bound is 2 * 64 + 1; two Names,
    // the colon and space, and " Answer" are 267 bytes (README.md, 3.5).
    const ANSWER_NAME: usize = 267;
    let w = f::browser_words();
    let mut title = skein_lib::bytes::zeroed(usize::try_from(w.name_bytes).unwrap());
    title.fill(b'T');
    let mut question = skein_lib::bytes::zeroed(usize::try_from(w.name_bytes).unwrap());
    question.fill(b'Q');
    let line = w.say(
        crate::words::Word::HomeAskLine,
        &[skein_ui::phrases::Hole::Name(title), skein_ui::phrases::Hole::Name(question)],
    );
    let name = w.say(crate::words::Word::HomeAnswerName, &[skein_ui::phrases::Hole::Name(line)]);
    assert_eq!(name.len(), ANSWER_NAME);
    assert!(name.ends_with(b" Answer"));
}
