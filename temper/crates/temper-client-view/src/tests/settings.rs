//! Settings' tabs, repository facts and sign out (temper/docs/design/client/pages/README.md).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic subtrees and their links"
)]
use super::{connect_forge as cf, fixtures as f};
use crate::look;
use jig_client::contract as c;
use skein_lib::{List, Queue};
use skein_ui::{Act, Node, Role, State, Style, Tree};
use temper_client_domain::{self as d, Address, pages};
use temper_client_protocol_wire::v1 as w;

fn subtree(tree: &Tree, id: skein_ui::NodeId) -> &[Node] {
    let at = tree.nodes().iter().position(|n| n.id == id).unwrap();
    &tree.nodes()[at..at.checked_add(usize::try_from(tree.nodes()[at].size).unwrap()).unwrap()]
}
fn main(tree: &Tree) -> &[Node] {
    let node = tree.nodes().iter().find(|n| n.role == Role::Main).unwrap();
    subtree(tree, node.id)
}
fn styled(tree: &Tree, style: Style) -> &[Node] {
    let node = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&style)).unwrap();
    subtree(tree, node.id)
}
fn find(tree: &Tree, role: Role, name: &[u8]) -> skein_ui::NodeId {
    main(tree).iter().find(|n| n.role == role && n.name.as_deref() == Some(name)).unwrap().id
}
fn root(address: Address) -> d::Domain {
    f::root_at(address, &f::domain_limits()).0
}
fn repositories() -> Address {
    Address::Repositories { project: f::PROJECT }
}
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
#[test]
fn every_tab_draws_its_head_and_tabs_in_order() {
    for (address, heading, lede, current) in [
        (
            Address::Settings { project: f::PROJECT },
            "How work runs in Shop.",
            "Ask temper to change any of it, in words; an owner accepts its proposal, and new work follows it.",
            0,
        ),
        (repositories(), "What Shop works on.", "The repositories chats and goals in Shop read and change.", 1),
        (
            Address::Forges,
            "Where your code lives.",
            "Each forge temper is connected to, or setting up. Add one to bring its repositories to temper.",
            1,
        ),
        (
            Address::Accounts,
            "What agents run on.",
            "The LLM accounts temper’s agents run on for you. A task keeps its account: while the account is limited, the task waits for it.",
            0,
        ),
        (
            Address::Models { project: f::PROJECT },
            "What the picker offers.",
            "The models your accounts can run, by what pays for them. Each model can be left off the picker; a task keeps the model it has.",
            2,
        ),
        (
            Address::Builders { project: f::PROJECT },
            "Where agents work.",
            "The machines temper’s agents run on, each with its slots and how many are in use.",
            3,
        ),
        (
            Address::Members,
            "Who may do what.",
            "Each project’s people and services, and the role each holds there. Who owns an area of a project is its Team’s.",
            4,
        ),
        (
            Address::Limits,
            "What this deployment holds.",
            "The capacities temper’s engine runs within, as it reports them. Its owner sets them in its configuration.",
            5,
        ),
    ] {
        let how = address == settings() || address == repositories();
        let root = root(address);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            assert_eq!(skein_ui::outline::check(tree), None);
            assert_eq!(main(tree)[0].styles.as_slice(), &[look::MAIN, look::SETTINGS]);
            let col = styled(tree, look::SETTINGS_COL);
            let intro = styled(tree, look::SETTINGS_INTRO);
            assert_eq!(col[1].id, intro[0].id);
            let text = String::from_utf8_lossy(&f::nodes_text(intro)).into_owned();
            assert_eq!(text, format!("{}{heading}{lede}", if how { "Shop · how work runs" } else { "Deployment" }));
            assert_eq!(intro.iter().filter(|n| n.role == Role::Heading { level: 1 }).count(), 1);
            let nav = styled(tree, look::SETTINGS_TABS);
            assert_eq!(nav[0].role, Role::Navigation);
            assert_eq!(
                nav[0].name.as_deref(),
                Some(if how { b"How work runs".as_slice() } else { b"Deployment".as_slice() })
            );
            let links: Vec<_> = nav.iter().filter(|n| n.role == Role::Link).collect();
            assert_eq!(
                links.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
                if how {
                    vec![b"How work runs".as_slice(), b"Repositories"]
                } else {
                    vec![b"Accounts".as_slice(), b"Forges", b"Models", b"Workers", b"Members", b"Limits"]
                }
            );
            let ids: Vec<_> = links
                .iter()
                .enumerate()
                .map(|(index, n)| {
                    assert_eq!(n.states.as_slice().contains(&State::Current), index == current);
                    n.id
                })
                .collect();
            let addresses = if how {
                vec![Address::Settings { project: f::PROJECT }, repositories()]
            } else {
                vec![
                    Address::Accounts,
                    Address::Forges,
                    Address::Models { project: f::PROJECT },
                    Address::Builders { project: f::PROJECT },
                    Address::Members,
                    Address::Limits,
                ]
            };
            for (id, address) in ids.into_iter().zip(addresses) {
                assert_eq!(crate::href(&mut view, id), Some(d::address::print(&address)));
                assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Pages(pages::Act::Go { address })));
            }
        }
    }
}
#[test]
fn start_goes_to_ask_and_go_on_to_the_open_chat() {
    let (mut root, me_watch, _) = f::root_at(Address::Settings { project: f::PROJECT }, &f::domain_limits());
    let template = f::chat::messaging_root();
    let offered = f::asking_me(template.pages().me(template.client()).unwrap().clone());
    let mut absent = offered.clone();
    let project = absent.projects.get_mut(0).unwrap();
    project.templates =
        List::from_boxed(project.templates.iter().filter(|template| template.number != 5).cloned().collect());
    f::deliver_me(&mut root, me_watch, absent);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(!view.tree().nodes().iter().any(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Ask temper")));
    }
    f::deliver_me(&mut root, me_watch, offered);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let links: Vec<_> = view
            .tree()
            .nodes()
            .iter()
            .filter(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Ask temper"))
            .map(|n| n.id)
            .collect();
        assert!(!links.is_empty());
        for start in links {
            assert_eq!(crate::href(&mut view, start).as_deref(), Some(b"/projects/1/ask".as_slice()));
            assert_eq!(
                crate::act(&mut view, start, Act::Press),
                Some(d::Action::Pages(pages::Act::Go { address: Address::Ask { project: f::PROJECT } }))
            );
        }
        assert!(!main(view.tree()).iter().any(|n| n.name.as_deref() == Some(b"Ask temper")));
    }
    drive(&mut root, d::Event::Went { place: d::Place::Address(Address::Accounts) });
    let mut out =
        drive(&mut root, d::Event::Went { place: d::Place::Address(Address::Settings { project: f::PROJECT }) });
    let mut task_watch = None;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { .. }) },
                },
        } = request
        {
            task_watch = Some(watch);
        }
    }
    let mut line = super::chats::line();
    line.chip.executor = c::ChipExecutor::Agent { charter: 37 };
    let task = line.chip.task;
    let mut rows = List::with_capacity(1);
    rows.push(line).unwrap();
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: task_watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false }))),
                },
            },
        },
    );
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let links: Vec<_> = view
            .tree()
            .nodes()
            .iter()
            .filter(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Ask temper"))
            .map(|n| n.id)
            .collect();
        assert!(!links.is_empty());
        for go in links {
            assert_eq!(crate::href(&mut view, go), Some(d::address::print(&Address::Chat { task })));
            assert_eq!(
                crate::act(&mut view, go, Act::Press),
                Some(d::Action::Pages(pages::Act::Go { address: Address::Chat { task } }))
            );
        }
    }
}

fn forge_rows() -> List<w::ForgeRow> {
    let mut rows = List::with_capacity(3);
    for (number, kind, mode, repos) in [
        (
            3,
            w::Kind::Forgejo,
            w::Mode::Owned,
            vec![(f::PROJECT, "work", "one"), (f::SECOND_PROJECT, "else", "private")],
        ),
        (4, w::Kind::Github, w::Mode::Connected, vec![(f::PROJECT, "work", "two")]),
        (5, w::Kind::Forgejo, w::Mode::Fake, vec![(f::PROJECT, "fake", "hidden")]),
    ] {
        let mut row = cf::row(number, kind, mode, cf::connected(), w::Offer::Offered).into_parts();
        row.repositories = List::with_capacity(u32::try_from(repos.len()).unwrap());
        for (project, owner, name) in repos {
            row.repositories
                .push(
                    w::Adoption::new(
                        &w::CEILINGS,
                        w::AdoptionParts { project, owner: owner.as_bytes().into(), name: name.as_bytes().into() },
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        rows.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    }
    rows
}

#[test]
fn repositories_lists_the_projects_and_no_others() {
    let root = f::forge_root(repositories(), forge_rows());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        find(view.tree(), Role::Heading { level: 2 }, "Repositories · 2".as_bytes());
        let adopt = find(view.tree(), Role::Link, b"Add repositories");
        assert_eq!(node(view.tree(), adopt).styles.as_slice(), [look::SETTINGS_BTN, look::SETTINGS_SMALL]);
        let rows = styled(view.tree(), look::SETTINGS_LINES);
        assert_eq!(f::nodes_text(rows).as_ref(), b"work/oneForgejo on this machinework/twoGitHub");
        assert_eq!(rows.iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_WHEN)).count(), 2);
        assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_BAND)));
    }
}
#[test]
fn repositories_says_none_with_a_quiet_band() {
    let root = f::forge_root(repositories(), List::with_capacity(0));
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        find(view.tree(), Role::Heading { level: 2 }, "Repositories · 0".as_bytes());
        let band = styled(view.tree(), look::SETTINGS_QUIET_BAND);
        assert!(band[0].styles.as_slice().contains(&look::SETTINGS_BAND));
        assert_eq!(f::nodes_text(band).as_ref(), b"No repository yetChats in Shop talk and plan, but change no code until a repository is in.Add repositories");
        let links: Vec<_> = main(view.tree())
            .iter()
            .filter(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Add repositories"))
            .map(|n| n.id)
            .collect();
        assert_eq!(links.len(), 2);
        for link in links {
            assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/projects/1/adopt".as_slice()));
        }
    }
}
#[test]
fn repositories_wait_for_the_watch_without_claiming_none() {
    let template = root(repositories());
    let mut unknown = d::unlinked(&f::domain_limits(), f::SEED);
    drive(&mut unknown, d::Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } });
    let mut out = drive(
        &mut unknown,
        d::Event::Start {
            address: repositories(),
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
    );
    let mut served = drive(
        &mut unknown,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
        },
    );
    while let Some(request) = served.pop() {
        out.push(request);
    }
    let mut me = None;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) },
                },
        } = request
        {
            me = Some(watch);
        }
    }
    f::deliver_me(&mut unknown, me.unwrap(), template.pages().me(template.client()).unwrap().clone());
    let (mut refused, watch) = f::forge_root_watch(repositories(), forge_rows());
    f::root_event(
        &mut refused,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::NotAllowed } },
        },
    );
    let (mut ended, watch) = f::forge_root_watch(repositories(), forge_rows());
    f::root_event(
        &mut ended,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::Closed } },
        },
    );
    for root in vec![unknown, refused, ended].into_boxed_slice() {
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            find(view.tree(), Role::Heading { level: 2 }, b"Repositories");
            assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_LINES)
                || n.styles.as_slice().contains(&look::SETTINGS_BAND)));
        }
    }
}
#[test]
fn the_playgrounds_project_shows_its_repository() {
    let root = f::root_at_with_playground(repositories(), &f::domain_limits(), b"Shop", Some((f::PROJECT, true))).0;
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        find(view.tree(), Role::Heading { level: 2 }, "Repositories · 1".as_bytes());
        assert_eq!(f::nodes_text(styled(view.tree(), look::SETTINGS_LINES)).as_ref(), b"acme/shopPlayground forge");
    }
}
#[test]
fn a_repository_row_is_no_link() {
    let root = f::forge_root(repositories(), forge_rows());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let rows = styled(view.tree(), look::SETTINGS_LINES);
        assert_eq!(rows.iter().filter(|n| n.role == Role::Slot).count(), 3);
        assert!(rows.iter().all(|n| n.role != Role::Link && n.binding.is_none() && n.external.is_none()));
    }
}
#[test]
fn accounts_are_in_the_projects_settings_column() {
    let root = f::root_at(Address::Accounts, &f::domain_limits()).0;
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_COL)));
        assert!(
            !main(view.tree())
                .iter()
                .any(|n| n.role == Role::Heading { level: 1 } && n.name.as_deref() == Some(b"Accounts"))
        );
        assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::PAGEBAR)));
        styled(view.tree(), look::ACCOUNTS_PAGE);
    }
}

#[test]
fn sign_out_is_at_the_foot_of_accounts() {
    let root = root(Address::Accounts);
    let me = root.pages().me(root.client()).unwrap();
    let signed_in = format!("Signed in as {}", core::str::from_utf8(me.party.name.bytes()).unwrap());
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let row = styled(view.tree(), look::SETTINGS_SETTING);
        assert!(row.iter().any(|n| n.name.as_deref() == Some(signed_in.as_bytes())));
        assert!(row.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_DETAIL)
            && n.name.as_deref() == Some(me.party.login.bytes())));
        assert_eq!(main(view.tree()).last().unwrap().id, row.last().unwrap().id);
        let sign_out = find(view.tree(), Role::Button, b"Sign out");
        assert_eq!(crate::act(&mut view, sign_out, Act::Press), Some(d::Action::Pages(pages::Act::SignOut)));
        let (node, act) =
            f::terminal_command(view.tree(), &view.limits().tree, sign_out, b"Sign out", b"What agents run on.");
        assert_eq!(crate::act(&mut view, node, act), Some(d::Action::Pages(pages::Act::SignOut)));
    }
}
#[test]
fn sign_out_lost_moves_with_it() {
    let mut root = root(Address::Accounts);
    f::root_event(&mut root, d::Event::Act { action: d::Action::Pages(pages::Act::SignOut) });
    f::root_event(&mut root, d::Event::Client { event: jig_client::Event::SignOutLost });
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(
            styled(view.tree(), look::SETTINGS_SETTING)
                .iter()
                .any(|n| n.role == Role::Alert && n.name.as_deref() == Some(b"Sign out did not reach the engine."))
        );
        assert!(
            styled(view.tree(), look::RAIL_FOOT)
                .iter()
                .any(|n| n.role == Role::Alert && n.name.as_deref() == Some(b"Sign out did not reach the engine."))
        );
    }
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Settings { project: f::PROJECT }) });
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(
            !main(view.tree())
                .iter()
                .any(|n| n.role == Role::Alert && n.name.as_deref() == Some(b"Sign out did not reach the engine."))
        );
        assert!(styled(view.tree(), look::RAIL_FOOT).iter().any(|n| n.role == Role::Alert));
    }
}

use d::forge::ways::{Asked, Control, Route, Ways};
use skein_lib::Token;
const SHOWN: Ways = Ways { drafts: true, goals: true, changes: true, way: 11 };
const SWITCHES: [(&str, &str, &str); 3] = [
    (
        "Chats can write drafts",
        "A chat keeps a document beside it, in docs/design/draft/, that a goal can start from.",
        "On",
    ),
    ("Goals", "Bigger work from a draft: a plan, tasks, landing.", "Shown"),
    ("Changes", "The landing queue, as Work's filter.", "Shown"),
];
const QUESTIONS: &str = "Questions a goal can answer from its plan";
fn settings() -> Address {
    Address::Settings { project: f::PROJECT }
}
fn how(ways: Option<Ways>) -> d::Domain {
    let (mut root, watch, ..) = f::root_at_with_values(
        settings(),
        &f::domain_limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        ways,
        false,
        true,
    );
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let stock = f::chat::messaging_root();
    let template = stock.pages().me(stock.client()).unwrap().projects.get(0).unwrap().templates.get(0).unwrap();
    let project = me.projects.get_mut(0).unwrap();
    project.templates = List::with_capacity(2);
    for (number, name) in [(11, "One agent, one branch"), (4, "goal for a draft")] {
        let mut template = template.clone();
        template.number = number;
        template.name = c::DocumentText::complete(c::Text::new(name.as_bytes().into(), 32).unwrap());
        template.tracked = Some(1);
        project.templates.push(template).unwrap();
    }
    f::deliver_me(&mut root, watch, me);
    root
}
fn how_view(root: &d::Domain, browser: bool) -> crate::View {
    let words = if browser { f::browser_words() } else { f::words() };
    let limits = crate::Limits::of(&f::domain_limits(), &words).unwrap();
    let mut view = crate::View::new(&limits, &f::domain_limits(), words, f::FORGE);
    f::build(&mut view, root);
    view
}
fn setting_row<'a>(tree: &'a Tree, name: &[u8]) -> &'a [Node] {
    main(tree)
        .iter()
        .filter(|n| n.styles.as_slice().contains(&look::SETTINGS_SETTING))
        .map(|n| subtree(tree, n.id))
        .find(|nodes| nodes.iter().any(|n| n.role == Role::Text && n.name.as_deref() == Some(name)))
        .unwrap()
}
fn section_nodes<'a>(tree: &'a Tree, title: &[u8]) -> &'a [Node] {
    main(tree)
        .iter()
        .filter(|n| n.styles.as_slice().contains(&look::SETTINGS_SECTION))
        .map(|n| subtree(tree, n.id))
        .find(|nodes| nodes.iter().any(|n| n.role == Role::Heading { level: 2 } && n.name.as_deref() == Some(title)))
        .unwrap()
}
fn node(tree: &Tree, id: skein_ui::NodeId) -> &Node {
    tree.nodes().iter().find(|n| n.id == id).unwrap()
}
fn selected(tree: &Tree, name: &[u8]) -> Vec<u8> {
    let id = find(tree, Role::Select, name);
    subtree(tree, id)
        .iter()
        .find(|n| n.role == Role::Option && n.states.as_slice().contains(&State::Selected))
        .unwrap()
        .name
        .as_deref()
        .unwrap()
        .to_vec()
}
fn send(root: &mut d::Domain, view: &mut crate::View, id: skein_ui::NodeId, act: Act) -> (Token, c::ChangeSettings) {
    let event = crate::act_event(view, id, act).expect("offered act");
    let mut out = drive(root, event);
    let mut stored = drive(root, d::Event::Client { event: jig_client::Event::Stored });
    while let Some(request) = stored.pop() {
        out.push(request);
    }
    let sent: Vec<_> = out
        .iter()
        .filter_map(|request| {
            if let d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Send { attempt, request: jig_client::Ask::Jig(ask), .. },
                    },
            } = request
            {
                let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("change settings") };
                Some((*attempt, change.clone()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(sent.len(), 1, "one act sends one request");
    sent.into_iter().next().unwrap()
}
fn refused_answer(root: &mut d::Domain, attempt: Token, refusal: c::Refusal) -> Queue<d::Request> {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused { refusal, at: None })),
                    why: None,
                },
            },
        },
    )
}
fn sent_value(change: &c::ChangeSettings, kind: u32) -> &[u8] {
    assert_eq!(change.items.len(), 1);
    assert_eq!(change.running, c::Running::Stays);
    let item = change.items.get(0).unwrap();
    assert_eq!(item.setting, c::Setting::Application { kind, scope: f::PROJECT, number: 1 });
    let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(bytes) }) = &item.edit else {
        panic!("whole application value")
    };
    bytes
}
fn changed_ways(change: &c::ChangeSettings) -> Ways {
    let mut reader = skein_lib::Reader::new(sent_value(change, 7));
    let value = w::ProjectWays::decode(&w::CEILINGS, &mut reader).unwrap();
    assert!(reader.is_empty());
    Ways { drafts: value.drafts(), goals: value.goals(), changes: value.changes(), way: value.way() }
}
#[test]
fn the_shows_section_reads_kind_seven() {
    for (shown, pressed) in
        [(Ways::STOCK, [true, false, false]), (Ways { drafts: false, ..Ways::STOCK }, [false; 3]), (SHOWN, [true; 3])]
    {
        let root = how(Some(shown));
        for browser in [false, true] {
            let view = how_view(&root, browser);
            assert_eq!(skein_ui::outline::check(view.tree()), None);
            let section = section_nodes(view.tree(), b"What the project shows");
            assert_eq!(section.iter().filter(|n| n.role == Role::Check).count(), 3);
            for (index, (label, small, on)) in SWITCHES.iter().enumerate() {
                let row = setting_row(view.tree(), label.as_bytes());
                assert!(row.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_DETAIL)
                    && n.name.as_deref() == Some(small.as_bytes())));
                let check = node(view.tree(), find(view.tree(), Role::Check, label.as_bytes()));
                assert_eq!(check.states.as_slice().contains(&State::Pressed), pressed[index], "{label}");
                assert!(!check.states.as_slice().contains(&State::Disabled));
                assert!(check.styles.as_slice().contains(&look::SETTINGS_SWITCH));
                let toggle = row.iter().find(|n| n.styles.as_slice().contains(&look::SETTINGS_TOGGLE_ROW)).unwrap();
                let text = f::nodes_text(subtree(view.tree(), toggle.id));
                assert_eq!(
                    text.as_ref(),
                    if pressed[index] {
                        on.as_bytes()
                    } else if index == 0 {
                        b"Off".as_slice()
                    } else {
                        b"Hidden".as_slice()
                    }
                );
            }
        }
    }
}
#[test]
fn a_switch_sends_the_whole_record_with_its_part_changed() {
    for browser in [false, true] {
        for (index, (label, ..)) in SWITCHES.iter().enumerate() {
            let mut root = how(Some(SHOWN));
            let mut view = how_view(&root, browser);
            let id = find(view.tree(), Role::Check, label.as_bytes());
            let (_, change) = send(&mut root, &mut view, id, Act::Toggle);
            let mut expected = SHOWN;
            match index {
                0 => expected.drafts = false,
                1 => expected.goals = false,
                2 => expected.changes = false,
                _ => unreachable!(),
            }
            assert_eq!(changed_ways(&change), expected);
            assert_eq!(change.against, 0);
        }
    }
}
#[test]
fn the_way_is_chosen_and_sends_the_whole_record() {
    for browser in [false, true] {
        let mut root = how(Some(SHOWN));
        let mut view = how_view(&root, browser);
        let row = setting_row(view.tree(), b"Plan, tasks, land whole");
        let id = row.iter().find(|n| n.role == Role::Choice).unwrap().id;
        let (_, change) = send(&mut root, &mut view, id, Act::Toggle);
        assert_eq!(changed_ways(&change), Ways { way: 4, ..SHOWN });
    }
}
#[test]
fn the_questions_select_sends_its_route() {
    for browser in [false, true] {
        for (label, route, value) in [
            ("Asked of me too", w::QuestionsRoute::Up, b"up".as_slice()),
            ("Not asked: each agent decides alone", w::QuestionsRoute::None, b"none".as_slice()),
        ] {
            let mut root = how(Some(SHOWN));
            let mut view = how_view(&root, browser);
            let id = find(view.tree(), Role::Option, label.as_bytes());
            let (_, change) = send(&mut root, &mut view, id, Act::Press);
            let decoded =
                w::ProjectQuestions::decode(&w::CEILINGS, &mut skein_lib::Reader::new(sent_value(&change, 6))).unwrap();
            assert_eq!(*decoded.route(), route);
            // The binding table also accepts a route supplied as whole select input.
            let mut bindings = crate::forge::bindings::Bindings::new(1, 1, f::FORGE);
            let bound = bindings.insert(crate::forge::bindings::Meaning::Ways(d::forge::ways::Act::Set {
                project: f::PROJECT,
                asked: Asked::Questions(Route::None),
            }));
            let expected = match route {
                w::QuestionsRoute::Up => Route::Up,
                w::QuestionsRoute::None => Route::None,
            };
            assert_eq!(
                crate::forge::bindings::decode(&bindings, bound.token, Act::Input { value: value.into() }),
                Some(d::Action::Forge(d::forge::Act::Ways(d::forge::ways::Act::Set {
                    project: f::PROJECT,
                    asked: Asked::Questions(expected)
                })))
            );
            assert_eq!(
                crate::forge::bindings::decode(
                    &bindings,
                    bound.token,
                    Act::Input { value: b"invalid".as_slice().into() }
                ),
                None
            );
        }
    }
}
#[test]
fn kinds_of_seven_are_disabled_together_while_one_is_out() {
    for browser in [false, true] {
        let mut root = how(Some(SHOWN));
        let mut view = how_view(&root, browser);
        let id = find(view.tree(), Role::Check, b"Changes");
        send(&mut root, &mut view, id, Act::Toggle);
        f::build(&mut view, &root);
        let controls: Vec<_> = main(view.tree())
            .iter()
            .filter(|n| n.role == Role::Choice || n.styles.as_slice().contains(&look::SETTINGS_SWITCH))
            .collect();
        assert_eq!(controls.len(), 5);
        let ids: Vec<_> = controls
            .iter()
            .map(|n| {
                assert!(n.states.as_slice().contains(&State::Disabled));
                n.id
            })
            .collect();
        for id in ids {
            assert_eq!(crate::act(&mut view, id, Act::Toggle), None);
        }
        assert!(
            !node(view.tree(), find(view.tree(), Role::Select, QUESTIONS.as_bytes()))
                .states
                .as_slice()
                .contains(&State::Disabled)
        );
    }
}
#[test]
fn the_value_asked_shows_while_out() {
    for browser in [false, true] {
        let mut root = how(Some(Ways::STOCK));
        let mut view = how_view(&root, browser);
        let id = find(view.tree(), Role::Check, b"Goals");
        send(&mut root, &mut view, id, Act::Toggle);
        f::build(&mut view, &root);
        let goals = node(view.tree(), find(view.tree(), Role::Check, b"Goals"));
        assert!(goals.states.as_slice().contains(&State::Pressed));
        assert!(goals.states.as_slice().contains(&State::Disabled));
        section_nodes(view.tree(), b"Ways of working");
        let id = find(view.tree(), Role::Option, b"Asked of me too");
        send(&mut root, &mut view, id, Act::Press);
        f::build(&mut view, &root);
        assert_eq!(selected(view.tree(), QUESTIONS.as_bytes()), b"Asked of me too");
        assert!(
            node(view.tree(), find(view.tree(), Role::Select, QUESTIONS.as_bytes()))
                .states
                .as_slice()
                .contains(&State::Disabled)
        );
        let id = find(view.tree(), Role::Option, b"Answered by the goal, kept in its decisions");
        assert_eq!(crate::act(&mut view, id, Act::Press), None);
    }
}
#[test]
fn a_refusal_is_said_under_its_row_and_the_control_goes_back() {
    for browser in [false, true] {
        for (label, ..) in SWITCHES {
            for refusal in [
                c::Refusal::NotOffered,
                c::Refusal::Role { held: None },
                c::Refusal::Full(c::RefusalFull { revision: 0, tasks: List::with_capacity(0), more: 0 }),
            ] {
                let mut root = how(Some(SHOWN));
                let mut view = how_view(&root, browser);
                let id = find(view.tree(), Role::Check, label.as_bytes());
                let (attempt, _) = send(&mut root, &mut view, id, Act::Toggle);
                refused_answer(&mut root, attempt, refusal.clone());
                f::build(&mut view, &root);
                let row = setting_row(view.tree(), label.as_bytes());
                let holes = if let c::Refusal::Full(full) = &refusal {
                    vec![skein_ui::phrases::Hole::Number(u64::from(full.revision))]
                } else {
                    vec![]
                };
                let said = f::words().say_word(jig_client::words::Word::refusal(&refusal), &holes);
                assert!(row.iter().any(|n| n.role == Role::Status && n.name.as_deref() == Some(said.as_ref())));
                let check = node(view.tree(), find(view.tree(), Role::Check, label.as_bytes()));
                assert!(check.states.as_slice().contains(&State::Pressed));
                assert!(!check.states.as_slice().contains(&State::Disabled));
                assert_eq!(
                    main(view.tree()).iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_REFUSAL)).count(),
                    1
                );
            }
        }
    }
}
#[test]
fn stale_goes_unseen() {
    let mut root = how(Some(SHOWN));
    let mut view = how_view(&root, true);
    let id = find(view.tree(), Role::Check, b"Changes");
    let (attempt, _) = send(&mut root, &mut view, id, Act::Toggle);
    let mut out = refused_answer(&mut root, attempt, c::Refusal::Stale(c::RefusalStale { newest: 8 }));
    f::build(&mut view, &root);
    assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_REFUSAL)));
    assert!(root.forge().ways_asked(Control::Ways).is_some());
    let mut stored = drive(&mut root, d::Event::Client { event: jig_client::Event::Stored });
    while let Some(request) = stored.pop() {
        out.push(request);
    }
    let (attempt, change) = out
        .iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Send { attempt, request: jig_client::Ask::Jig(ask), .. },
                    },
            } = request
            {
                let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("retry") };
                Some((*attempt, change.clone()))
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(change.against, 8);
    assert_eq!(changed_ways(&change), Ways { changes: false, ..SHOWN });
    refused_answer(&mut root, attempt, c::Refusal::Stale(c::RefusalStale { newest: 9 }));
    f::build(&mut view, &root);
    assert!(setting_row(view.tree(), b"Changes").iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_REFUSAL)));
    assert!(root.forge().ways_asked(Control::Ways).is_none());
}
#[test]
fn ways_of_working_only_while_goals_shown() {
    for value in [Ways::STOCK, SHOWN] {
        let root = how(Some(value));
        for browser in [false, true] {
            let view = how_view(&root, browser);
            assert_eq!(
                main(view.tree())
                    .iter()
                    .any(|n| n.role == Role::Heading { level: 2 } && n.name.as_deref() == Some(b"Ways of working")),
                value.goals
            );
        }
    }
}
#[test]
fn the_stored_way_or_the_first_is_the_default() {
    for (way, expected) in [
        (11, "One agent, one branch"),
        (4, "Plan, tasks, land whole"),
        (0, "Plan, tasks, land whole"),
        (99, "Plan, tasks, land whole"),
    ] {
        let root = how(Some(Ways { way, ..SHOWN }));
        for browser in [false, true] {
            let view = how_view(&root, browser);
            let group = section_nodes(view.tree(), b"Ways of working");
            assert_eq!(group.iter().filter(|n| n.role == Role::ChoiceGroup).count(), 1);
            assert_eq!(
                group
                    .iter()
                    .filter(|n| n.role == Role::Choice && n.states.as_slice().contains(&State::Selected))
                    .count(),
                1
            );
            assert!(
                setting_row(view.tree(), expected.as_bytes())
                    .iter()
                    .any(|n| n.role == Role::Choice && n.states.as_slice().contains(&State::Selected))
            );
            let names: Vec<_> = group
                .iter()
                .filter(|n| n.role == Role::Text && n.name.is_some())
                .map(|n| n.name.as_deref().unwrap())
                .collect();
            assert_eq!(names[0], b"Plan, tasks, land whole");
        }
    }
}
#[test]
fn no_used_by_with_no_template_recorded() {
    let root = how(Some(SHOWN));
    let view = how_view(&root, true);
    let text = String::from_utf8_lossy(&f::nodes_text(section_nodes(view.tree(), b"Ways of working"))).into_owned();
    assert!(
        text.contains("A plan from the draft, its tasks on a goal branch, landed as one change once all are checked.")
    );
    let custom = setting_row(view.tree(), b"One agent, one branch");
    assert!(!custom.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_DETAIL)));
    assert!(!text.contains("Used by"));
}
#[test]
fn until_read_the_switches_are_disabled_and_show_stock() {
    for root in f::pair(how(None), f::root_with_refused_ways(settings())) {
        for browser in [false, true] {
            let mut view = how_view(&root, browser);
            for (index, (label, ..)) in SWITCHES.iter().enumerate() {
                let id = find(view.tree(), Role::Check, label.as_bytes());
                let check = node(view.tree(), id);
                assert!(check.states.as_slice().contains(&State::Disabled));
                assert_eq!(check.states.as_slice().contains(&State::Pressed), index == 0);
                assert_eq!(crate::act(&mut view, id, Act::Toggle), None);
            }
            assert!(
                node(view.tree(), find(view.tree(), Role::Select, QUESTIONS.as_bytes()))
                    .states
                    .as_slice()
                    .contains(&State::Disabled)
            );
        }
    }
}

#[test]
fn forges_keep_the_projects_heading_lede_and_tabs() {
    let (root, _, _) = f::root_at(Address::Forges, &f::domain_limits());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(main(view.tree())[0].styles.as_slice(), &[look::MAIN, look::SETTINGS]);
        let col = styled(view.tree(), look::SETTINGS_COL);
        let intro = styled(view.tree(), look::SETTINGS_INTRO);
        assert!(col.len() > intro.len());
        find(view.tree(), Role::Heading { level: 1 }, b"Where your code lives.");
        assert_eq!(intro.iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_KICKER)).count(), 1);
        assert_eq!(main(view.tree()).iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_TABS)).count(), 1);
        find(
            view.tree(),
            Role::Paragraph,
            b"Each forge temper is connected to, or setting up. Add one to bring its repositories to temper.",
        );
    }
}

#[test]
fn each_flow_draws_its_path_in_the_browser_and_none_in_the_terminal() {
    // The providers offered by connect_model::plans_root.
    const CHATGPT_PROVIDER: u32 = 17;
    const CLAUDE_PROVIDER: u32 = 18;
    let offered = super::connect_model::plans_root();
    let providers = offered.pages().me(offered.client()).unwrap().providers.clone();
    for (address, tab, to, choices, here) in [
        (
            Address::ConnectForge { forge: None },
            b"Forges".as_slice(),
            b"/forges".as_slice(),
            None,
            b"Connect a forge".as_slice(),
        ),
        (
            Address::ConnectModel { way: None },
            b"Accounts".as_slice(),
            b"/accounts".as_slice(),
            None,
            b"Connect a model".as_slice(),
        ),
        (
            Address::ConnectModel {
                way: Some(d::ModelWay { provider: CHATGPT_PROVIDER, kind: d::WayKind::SignIn, source: None }),
            },
            b"Accounts".as_slice(),
            b"/accounts".as_slice(),
            Some((b"Connect a model".as_slice(), b"/accounts/connect".as_slice())),
            b"ChatGPT".as_slice(),
        ),
        (
            Address::ConnectModel {
                way: Some(d::ModelWay { provider: CLAUDE_PROVIDER, kind: d::WayKind::Program, source: None }),
            },
            b"Accounts".as_slice(),
            b"/accounts".as_slice(),
            Some((b"Connect a model".as_slice(), b"/accounts/connect".as_slice())),
            b"Claude".as_slice(),
        ),
        (
            Address::ConnectModel {
                way: Some(d::ModelWay { provider: CHATGPT_PROVIDER, kind: d::WayKind::Key, source: None }),
            },
            b"Accounts".as_slice(),
            b"/accounts".as_slice(),
            Some((b"Connect a model".as_slice(), b"/accounts/connect".as_slice())),
            b"OpenAI API key".as_slice(),
        ),
    ] {
        let (mut root, watch, _) = f::root_at(address, &f::domain_limits());
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.providers = providers.clone();
        f::deliver_me(&mut root, watch, me);
        let terminal = f::page_view(&root);
        assert!(!terminal.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_PATH)));
        let mut browser = f::browser_page_view(&root);
        let path = styled(browser.tree(), look::SETTINGS_PATH);
        assert!(path.iter().any(|n| n.role == Role::Navigation && n.name.as_deref() == Some(b"Where")));
        let links: Vec<_> = path.iter().filter(|n| n.role == Role::Link).map(|n| (n.id, n.name.clone())).collect();
        let mut expected =
            vec![choices.unwrap_or((tab, to)), (b"Deployment".as_slice(), b"/accounts".as_slice()), (tab, to)];
        expected.extend(choices);
        assert_eq!(links.len(), expected.len());
        let last = path.iter().find(|n| n.styles.as_slice().contains(&look::SETTINGS_DETAIL)).unwrap();
        assert_eq!(last.role, Role::Text);
        assert_eq!(last.name.as_deref(), Some(here));
        assert_eq!(f::nodes_text(subtree(browser.tree(), last.id)).as_ref(), here);
        assert!(last.binding.is_none() && last.external.is_none());
        assert!(path.iter().find(|n| n.styles.as_slice().contains(&look::BACK)).unwrap().role == Role::Link);
        for ((id, name), (label, to)) in links.into_iter().zip(expected) {
            assert_eq!(name.as_deref(), Some(label));
            assert_eq!(crate::href(&mut browser, id).as_deref(), Some(to));
        }
        assert!(
            styled(browser.tree(), look::RAIL)
                .iter()
                .any(|n| n.name.as_deref() == Some(b"Deployment") && n.states.as_slice().contains(&State::Current))
        );
    }
}

#[test]
fn no_project_draws_no_path() {
    for address in [Address::ConnectForge { forge: None }, Address::ConnectModel { way: None }] {
        let (mut root, watch, _) = f::root_at(address, &f::domain_limits());
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.projects.clear();
        f::deliver_me(&mut root, watch, me);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_PATH)));
        }
    }
}

const AWAY_TITLE: &str = "What reaches you";
const EIGHT: &[u8] = b"08:00, the end of quiet hours";
fn away_settings(row: Option<w::AwayRowParts>) -> (d::Domain, Token) {
    let (mut root, _, _, _, _, _, _, _, watch) = f::root_at_with_values(
        settings(),
        &f::domain_limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        Some(SHOWN),
        false,
        true,
    );
    let watch = watch.unwrap();
    away_deliver(&mut root, watch, row);
    (root, watch)
}
fn away_row(until: Option<u64>, off: Option<u64>) -> w::AwayRowParts {
    w::AwayRowParts { project: f::PROJECT, since: 0, until, landed: 0, off }
}
fn away_deliver(root: &mut d::Domain, watch: Token, row: Option<w::AwayRowParts>) {
    let mut rows = List::with_capacity(1);
    if let Some(row) = row {
        rows.push(w::AwayRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    }
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: f::FORGE,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                        bytes: temper_client_protocol_wire::aways(
                            &w::Aways::new(&w::CEILINGS, w::AwaysParts { rows }).unwrap(),
                        ),
                    },
                },
            },
        },
    );
}
fn settings_send(root: &mut d::Domain, view: &mut crate::View, role: Role, name: &[u8]) -> Queue<d::Request> {
    let id = find(view.tree(), role, name);
    let event = crate::act_event(view, id, if role == Role::Check { Act::Toggle } else { Act::Press }).unwrap();
    let mut out = drive(root, event);
    let mut stored = drive(root, d::Event::Client { event: jig_client::Event::Stored });
    while let Some(request) = stored.pop() {
        out.push(request);
    }
    out
}
fn away_send(root: &mut d::Domain, view: &mut crate::View, role: Role, name: &[u8]) -> (Token, w::SetAwayParts) {
    let out = settings_send(root, view, role, name);
    let sent: Vec<_> =
        out.iter()
            .filter_map(|request| {
                if let d::Request::Unlinked {
                    request:
                        jig_client::UnlinkedRequest::Down {
                            down:
                                jig_client::Down::Send {
                                    attempt, request: jig_client::Ask::Family { scope, record }, ..
                                },
                        },
                } = request
                {
                    let w::Request::SetAway(value) =
                        temper_client_protocol_wire::decode_requesting(&record.bytes, &w::CEILINGS)
                            .unwrap()
                            .into_parts()
                            .request
                    else {
                        panic!("SetAway")
                    };
                    assert_eq!(*scope, c::Scope::Project(f::PROJECT));
                    Some((*attempt, value.into_parts()))
                } else {
                    None
                }
            })
            .collect();
    assert_eq!(sent.len(), 1);
    sent.into_iter().next().unwrap()
}
fn settings_answer(root: &mut d::Domain, attempt: Token, outcome: w::Outcome) {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: f::FORGE,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                        bytes: temper_client_protocol_wire::answer(
                            &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap(),
                        ),
                    })),
                },
            },
        },
    );
}
fn away_answer(root: &mut d::Domain, attempt: Token) {
    settings_answer(root, attempt, w::Outcome::Away);
}
fn drawn_choices(view: &crate::View, until: &[u8]) {
    assert!(
        section_nodes(view.tree(), AWAY_TITLE.as_bytes())
            .iter()
            .any(|node| node.role == Role::Select && node.name.as_deref() == Some(b"Away until"))
    );
    assert_eq!(selected(view.tree(), b"Away until"), until);
}
const DECIDE: &str = "Agents decide, then tell you";
const ASK: &str = "Asks you";
const WAIT: &str = "Always waits for you";
const DECIDE_NOTE: &str = "Listed in Decided for you for 24 hours, to overturn.";
const AWAY_ALL: &str = "While you’re away, they are decided with the recommended choice and listed.";
const AWAY_NONE: &str = "They ask even while you’re away.";
const MOVABLE_LINES: [&str; 6] = [
    "wording and names inside a spec",
    "retrying or splitting a task",
    "implementation choices a spec leaves open",
    "test-only changes",
    "a new dependency",
    "a departure from a landed design",
];
const FIXED_LINES: [&str; 4] = [
    "money past a budget",
    "what can’t be undone: rewriting history, a force-push, deleting unlanded work",
    "changes to this page",
    "anything an agent can’t place",
];
fn lines(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}
struct ReachesCase {
    name: &'static str,
    classes: Option<temper_protocol_reaches::Classes>,
    columns: [Vec<String>; 3],
    notes: [Option<&'static str>; 3],
    away: bool,
    several: bool,
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table checks the columns, wording, unread state and full table in both shells"
)]
fn reaches_columns_follow_the_project_table() {
    use temper_protocol_reaches::{Class, Classes, Item, Kind, PathSet, Subject, Way};
    let review = Classes::stock(Way::AgentsReview);
    let story = Classes {
        items: Box::new([
            Item { subject: Subject::Kind(Kind::Wording), words: Box::default(), class: Class::Decide },
            Item { subject: Subject::Kind(Kind::Retry), words: Box::default(), class: Class::Decide },
            Item { subject: Subject::Kind(Kind::Implementation), words: Box::default(), class: Class::Decide },
            Item { subject: Subject::Kind(Kind::Tests), words: Box::default(), class: Class::Decide },
            Item { subject: Subject::Set(1), words: Box::default(), class: Class::AskAway },
            Item { subject: Subject::Kind(Kind::Dependency), words: Box::default(), class: Class::Ask },
            Item { subject: Subject::Kind(Kind::Departure), words: Box::default(), class: Class::Ask },
        ]),
        sets: Box::new([PathSet {
            number: 1,
            name: "vsr’s protocol core".as_bytes().into(),
            patterns: Box::new([b"include/vsr.h".as_slice().into()]),
        }]),
    };
    let mut words = review.clone();
    let mut items = words.items.to_vec();
    items.push(Item { subject: Subject::Kind(Kind::Tests), words: b"WORDS".as_slice().into(), class: Class::Ask });
    words.items = items.into_boxed_slice();
    let decided = Classes {
        items: story
            .items
            .iter()
            .filter(|item| item.subject != Subject::Set(1))
            .map(|item| Item { class: Class::Decide, ..item.clone() })
            .collect(),
        sets: Box::default(),
    };
    let full = Classes {
        items: (0..temper_protocol_reaches::ITEMS)
            .map(|index| Item {
                subject: Subject::Kind(Kind::Wording),
                words: if index == 0 {
                    Box::default()
                } else {
                    vec![b'x'; temper_protocol_reaches::WORDS_BYTES].into_boxed_slice()
                },
                class: Class::Decide,
            })
            .collect(),
        sets: Box::default(),
    };
    let full_lines = full
        .items
        .iter()
        .enumerate()
        .map(|(index, _)| {
            if index == 0 {
                MOVABLE_LINES[0].to_owned()
            } else {
                format!("wording and names inside a spec under “{}”", "x".repeat(temper_protocol_reaches::WORDS_BYTES))
            }
        })
        .collect();
    let cases = [
        ReachesCase {
            name: "Agents review",
            classes: Some(review.clone()),
            columns: [
                lines(&[MOVABLE_LINES[0], MOVABLE_LINES[1], MOVABLE_LINES[3]]),
                lines(&[MOVABLE_LINES[2], MOVABLE_LINES[4], MOVABLE_LINES[5]]),
                lines(&FIXED_LINES),
            ],
            notes: [Some(DECIDE_NOTE), Some(AWAY_ALL), Some("Away or not.")],
            away: true,
            several: false,
        },
        ReachesCase {
            name: "You approve",
            classes: Some(Classes::stock(Way::YouApprove)),
            columns: [vec![], lines(&MOVABLE_LINES), lines(&FIXED_LINES)],
            notes: [None, Some(AWAY_NONE), Some("Away or not.")],
            away: false,
            several: false,
        },
        ReachesCase {
            name: "whole story",
            classes: Some(story),
            columns: [
                lines(&MOVABLE_LINES[..4]),
                lines(&["a change to vsr’s protocol core", MOVABLE_LINES[4], MOVABLE_LINES[5]]),
                lines(&FIXED_LINES),
            ],
            notes: [
                Some(DECIDE_NOTE),
                Some(
                    "While you’re away, all but changes to vsr’s protocol core are decided with the recommended choice and listed.",
                ),
                Some("Away or not."),
            ],
            away: true,
            several: false,
        },
        ReachesCase {
            name: "item with words",
            classes: Some(words),
            columns: [
                lines(&[MOVABLE_LINES[0], MOVABLE_LINES[1], MOVABLE_LINES[3]]),
                lines(&[MOVABLE_LINES[2], MOVABLE_LINES[4], MOVABLE_LINES[5], "test-only changes under “WORDS”"]),
                lines(&FIXED_LINES),
            ],
            notes: [Some(DECIDE_NOTE), Some(AWAY_ALL), Some("Away or not.")],
            away: true,
            several: false,
        },
        ReachesCase {
            name: "all decided",
            classes: Some(decided),
            columns: [lines(&MOVABLE_LINES), vec![], lines(&FIXED_LINES)],
            notes: [Some(DECIDE_NOTE), None, Some("Away or not.")],
            away: false,
            several: false,
        },
        ReachesCase {
            name: "full table",
            classes: Some(full),
            columns: [full_lines, lines(&MOVABLE_LINES[1..]), lines(&FIXED_LINES)],
            notes: [Some(DECIDE_NOTE), Some(AWAY_NONE), Some("Away or not.")],
            away: false,
            several: false,
        },
        ReachesCase {
            name: "unread",
            classes: None,
            columns: [vec![], vec![], vec![]],
            notes: [None, None, None],
            away: false,
            several: false,
        },
        ReachesCase {
            name: "several projects",
            classes: Some(review),
            columns: [
                lines(&[MOVABLE_LINES[0], MOVABLE_LINES[1], MOVABLE_LINES[3]]),
                lines(&[MOVABLE_LINES[2], MOVABLE_LINES[4], MOVABLE_LINES[5]]),
                lines(&FIXED_LINES),
            ],
            notes: [Some(DECIDE_NOTE), Some(AWAY_ALL), Some("Away or not.")],
            away: true,
            several: true,
        },
    ];
    for case in cases {
        let (mut root, watch, ..) = f::root_at_with_classes(
            settings(),
            &f::domain_limits(),
            b"Shop",
            None,
            None,
            None,
            Some(&[]),
            if case.classes.is_some() { Some(SHOWN) } else { None },
            false,
            true,
            case.classes.as_ref(),
        );
        let mut me = root.pages().me(root.client()).unwrap().clone();
        let first = me.projects.get(0).unwrap().clone();
        me.projects = List::uncapped();
        me.projects.push(first.clone()).unwrap();
        if case.several {
            let mut second = first;
            second.project = f::SECOND_PROJECT;
            me.projects.push(second).unwrap();
        }
        f::deliver_me(&mut root, watch, me);
        for browser in [false, true] {
            let view = how_view(&root, browser);
            let tree = view.tree();
            assert_eq!(skein_ui::outline::check(tree), None, "{}", case.name);
            let section = section_nodes(tree, AWAY_TITLE.as_bytes());
            let head = section.iter().find(|n| n.styles.as_slice().contains(&look::SETTINGS_SECTION_HEAD)).unwrap();
            let right = if case.several { "on Shop; each project has its own" } else { "on Shop" };
            assert!(
                subtree(tree, head.id)
                    .iter()
                    .any(|n| n.role == Role::Text && n.name.as_deref() == Some(right.as_bytes())),
                "{}",
                case.name
            );
            assert_eq!(
                section.iter().any(|n| n.styles.as_slice().contains(&look::HOW_REACHES)),
                case.classes.is_some()
            );
            let groups: Vec<_> = section.iter().filter(|n| n.role == Role::Group).collect();
            let expected: Vec<_> = [DECIDE, ASK, WAIT]
                .into_iter()
                .zip(&case.columns)
                .filter(|(_, lines)| !lines.is_empty())
                .map(|(title, _)| title.as_bytes())
                .collect();
            assert_eq!(
                groups.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
                expected,
                "{}",
                case.name
            );
            for ((title, expected), note) in [DECIDE, ASK, WAIT].into_iter().zip(&case.columns).zip(case.notes) {
                let group = groups.iter().find(|n| n.name.as_deref() == Some(title.as_bytes()));
                assert_eq!(group.is_some(), !expected.is_empty(), "{}: {title}", case.name);
                if let Some(group) = group {
                    assert_eq!(group.styles.as_slice(), &[look::HOW_REACH]);
                    let nodes = subtree(tree, group.id);
                    let list = nodes.iter().find(|n| n.role == Role::List).unwrap();
                    let drawn: Vec<_> = subtree(tree, list.id)
                        .iter()
                        .filter(|n| n.role == Role::Item)
                        .map(|n| core::str::from_utf8(n.name.as_deref().unwrap()).unwrap())
                        .collect();
                    assert_eq!(drawn, *expected, "{}: {title}", case.name);
                    let note_node = nodes.iter().find(|n| n.role == Role::Paragraph).unwrap();
                    assert_eq!(note_node.styles.as_slice(), &[look::HOW_REACH_NOTE]);
                    assert_eq!(note_node.name.as_deref(), note.map(str::as_bytes), "{}: {title}", case.name);
                    let chip =
                        nodes.iter().find(|n| n.role == Role::Text && n.name.as_deref() == Some(title.as_bytes()));
                    assert!(chip.is_some());
                    if let Some(chip) = chip {
                        assert_eq!(
                            chip.styles.as_slice(),
                            &[if title == DECIDE { look::HOW_WHO_AGENT } else { look::HOW_WHO_PERSON }]
                        );
                    }
                }
            }
            assert!(!section.iter().any(|n| n.role == Role::Check), "classes change by proposal");
            assert!(
                section.iter().any(|n| n.role == Role::Select
                    && n.name.as_deref() == Some(b"Questions a goal can answer from its plan"))
            );
            assert_eq!(
                section.iter().any(|n| n.role == Role::Select && n.name.as_deref() == Some(b"Away until")),
                case.away,
                "{}",
                case.name
            );
            if !browser {
                let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
                skein_ui_text::render(tree, b"temper", None, &mut print);
                let text = core::str::from_utf8(print.text.as_slice()).unwrap();
                for (title, column) in [DECIDE, ASK, WAIT].into_iter().zip(&case.columns) {
                    assert_eq!(text.matches(title).count(), usize::from(!column.is_empty()), "{}: {title}", case.name);
                    for line in column {
                        assert!(text.contains(line), "{}: {line}", case.name);
                    }
                }
            }
        }
    }
}
#[test]
fn defaults_with_no_row() {
    let (root, _) = away_settings(None);
    for view in [how_view(&root, false), how_view(&root, true)] {
        drawn_choices(&view, EIGHT);
    }
}
#[test]
fn the_select_writes_until() {
    for browser in [false, true] {
        for (label, until) in [(EIGHT, Some(28_800_000_000_000)), (b"I turn it off".as_slice(), None)] {
            let (mut root, _) = away_settings(Some(away_row(None, None)));
            let mut view = how_view(&root, browser);
            let (_, sent) = away_send(&mut root, &mut view, Role::Option, label);
            assert_eq!(sent, w::SetAwayParts { project: f::PROJECT, on: true, until });
        }
    }
}
#[test]
fn it_works_while_off() {
    for browser in [false, true] {
        for (role, label, eight, until) in [
            (Role::Option, EIGHT, true, Some(28_800_000_000_000)),
            (Role::Option, b"I turn it off".as_slice(), false, None),
        ] {
            let (mut root, watch) = away_settings(Some(away_row(Some(28_800_000_000_000), Some(0))));
            let mut view = how_view(&root, browser);
            let (attempt, sent) = away_send(&mut root, &mut view, role, label);
            assert_eq!(sent, w::SetAwayParts { project: f::PROJECT, on: false, until });
            away_answer(&mut root, attempt);
            away_deliver(&mut root, watch, Some(away_row(sent.until, Some(0))));
            let view = how_view(&root, browser);
            drawn_choices(&view, if eight { EIGHT } else { b"I turn it off" });
            assert!(!root.away_on(f::PROJECT, f::NOW));
        }
    }
}
#[test]
fn limits_focuses_the_section() {
    let (mut root, _) = away_settings(None);
    for count in 1..=2 {
        let out = drive(
            &mut root,
            d::Event::Act {
                action: d::Action::Pages(pages::Act::Away(pages::away::Act::Limits { project: f::PROJECT })),
            },
        );
        assert!(out.iter().any(|request| *request == d::Request::Show { address: settings(), replace: false }));
        drive(&mut root, d::Event::Went { place: d::Place::Address(settings()) });
        for view in [how_view(&root, false), how_view(&root, true)] {
            assert_eq!(
                node(view.tree(), find(view.tree(), Role::Heading { level: 2 }, AWAY_TITLE.as_bytes())).focus,
                Some(count)
            );
        }
    }
}
#[test]
fn an_expired_until_still_reads_eight() {
    let (root, _) = away_settings(Some(away_row(Some(0), None)));
    for view in [how_view(&root, false), how_view(&root, true)] {
        drawn_choices(&view, EIGHT);
    }
}
#[test]
fn away_controls_are_numbered_in_the_terminal() {
    let (root, _) = away_settings(None);
    let view = how_view(&root, false);
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    let controls: Vec<_> = print.numbers.as_slice().to_vec();
    for (role, label) in [(Role::Option, EIGHT), (Role::Option, b"I turn it off".as_slice())] {
        assert!(controls.contains(&find(view.tree(), role, label)), "numbered {label:?}");
    }
}

fn making_rows(copy: bool, state: w::MakingState) -> List<w::ForgeRow> {
    let mut row = cf::row(3, w::Kind::Forgejo, w::Mode::Owned, cf::connected(), w::Offer::Offered).into_parts();
    row.makings = List::with_capacity(3);
    for project in [Some(f::PROJECT), Some(f::SECOND_PROJECT), None] {
        row.makings
            .push(
                w::RepositoryMaking::new(
                    &w::CEILINGS,
                    w::RepositoryMakingParts {
                        org: b"acme".as_slice().into(),
                        name: if project == Some(f::PROJECT) {
                            b"again".as_slice().into()
                        } else {
                            b"hidden".as_slice().into()
                        },
                        from: if copy {
                            w::MakeFrom::Copy(
                                w::MakeCopy::new(
                                    &w::CEILINGS,
                                    w::MakeCopyParts {
                                        address: b"https://source.example/acme/again".as_slice().into(),
                                    },
                                )
                                .unwrap(),
                            )
                        } else {
                            w::MakeFrom::Empty
                        },
                        into: project.map(|project| {
                            w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project, home: false }).unwrap()
                        }),
                        state: state.clone(),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut rows = List::with_capacity(1);
    rows.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    rows
}
fn failed(reason: w::MakeFailure) -> w::MakingState {
    w::MakingState::Failed(
        w::MakingFailed::new(
            &w::CEILINGS,
            w::MakingFailedParts {
                step: w::MakeStep::Repository,
                reason,
                words: b"You can not import from disallowed hosts.".as_slice().into(),
            },
        )
        .unwrap(),
    )
}
#[test]
fn making_rows_name_the_state_and_only_their_project() {
    for (copy, state, expected) in [
        (true, w::MakingState::Waiting, "Copying · Forgejo on this machine"),
        (false, w::MakingState::Making, "Making · Forgejo on this machine"),
        (true, failed(w::MakeFailure::Other), "Copy failed · Forgejo on this machine"),
        (false, failed(w::MakeFailure::Other), "Make failed · Forgejo on this machine"),
    ] {
        let root = f::forge_root(repositories(), making_rows(copy, state));
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(skein_ui::outline::check(view.tree()), None);
            find(view.tree(), Role::Heading { level: 2 }, "Repositories · 0".as_bytes());
            let rows = styled(view.tree(), look::SETTINGS_LINES);
            let text = String::from_utf8_lossy(&f::nodes_text(rows)).into_owned();
            assert!(text.contains("acme/again"));
            assert!(text.contains(expected), "{text}");
            assert!(!text.contains("hidden"));
            assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_QUIET_BAND)));
        }
    }
}
#[test]
fn failed_rows_offer_their_retry_and_dismissal() {
    use d::forge::sheet::Act as Sheet;
    let forge = 3;
    let org = b"acme".as_slice();
    let name = b"again".as_slice();
    for (copy, reason, label, expected) in [
        (
            true,
            w::MakeFailure::Other,
            b"Copy again".as_slice(),
            Sheet::CopyAgain { forge, org: org.into(), name: name.into() },
        ),
        (
            false,
            w::MakeFailure::Other,
            b"Make again".as_slice(),
            Sheet::MakeAgain { forge, org: org.into(), name: name.into() },
        ),
        (
            true,
            w::MakeFailure::Taken,
            b"Adopt it".as_slice(),
            Sheet::AdoptAgain { forge, org: org.into(), name: name.into() },
        ),
    ] {
        let root = f::forge_root(repositories(), making_rows(copy, failed(reason)));
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let row = styled(view.tree(), look::CONNECT_FAILED);
            assert_eq!(row[0].role, Role::Slot);
            assert!(
                String::from_utf8_lossy(&f::nodes_text(row))
                    .contains("Forgejo said: “You can not import from disallowed hosts.”")
            );
            let retry = find(view.tree(), Role::Button, label);
            assert_eq!(node(view.tree(), retry).styles.as_slice(), &[look::CONNECT_QUIET]);
            assert_eq!(
                crate::act(&mut view, retry, Act::Press),
                Some(d::Action::Forge(d::forge::Act::Sheet(expected.clone())))
            );
            let dismiss = find(view.tree(), Role::Button, b"Dismiss");
            assert_eq!(
                crate::act(&mut view, dismiss, Act::Press),
                Some(d::Action::Forge(d::forge::Act::Sheet(Sheet::Dismiss {
                    forge,
                    org: org.into(),
                    name: name.into()
                })))
            );
        }
    }
}

#[test]
fn dismissal_refusals_stay_in_the_repository_row() {
    let (mut root, watch) = f::forge_root_watch(repositories(), making_rows(true, failed(w::MakeFailure::Other)));
    let mut view = f::page_view(&root);
    let out = settings_send(&mut root, &mut view, Role::Button, b"Dismiss");
    let attempt = out
        .iter()
        .find_map(|request| match request {
            d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } => Some(*attempt),
            d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Copy { .. }
            | d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Fact { .. } => None,
        })
        .unwrap();
    let refused = w::MakeRefused::new(&w::CEILINGS, w::MakeRefusedParts { why: w::MakeRefusal::NotFailed }).unwrap();
    settings_answer(&mut root, attempt, w::Outcome::MakeRefused(refused));
    for state in [failed(w::MakeFailure::Other), w::MakingState::Making] {
        let rows = making_rows(true, state);
        let record = jig_client::Family {
            family: f::FORGE,
            version: temper_client_protocol_wire::CLIENT_VERSION,
            bytes: temper_client_protocol_wire::forges(
                &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, linked: false }).unwrap(),
            ),
        };
        drive(
            &mut root,
            d::Event::Client { event: jig_client::Event::Up { up: jig_client::Up::FamilyDelivered { watch, record } } },
        );
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let rows = styled(view.tree(), look::SETTINGS_LINES);
            assert!(rows.iter().any(|node| node.role == Role::Alert
                && node.name.as_deref() == Some(b"This repository is not a failed making.")));
        }
    }
}

fn role_card(rule: bool) -> w::View {
    let facts: &[(&[u8], &[u8])] = if rule {
        &[(b"Covers", b"every branch")]
    } else {
        &[
            (b"Model", b"A model, high"),
            (b"Can", b"read the change"),
            (b"Asked to", b"Review the change."),
            (b"Holds", b"the merge"),
        ]
    };
    let card = w::Card::new(
        &w::CEILINGS,
        w::CardParts {
            name: if rule { b"the project's rule 1".as_slice().into() } else { b"the reviewer".as_slice().into() },
            who: if rule {
                "a landing rule · holds a merge until it is met".as_bytes().into()
            } else {
                b"an agent reviewing a change".as_slice().into()
            },
            facts: List::from_boxed(
                facts
                    .iter()
                    .map(|(label, value)| {
                        w::Fact::new(&w::CEILINGS, w::FactParts { label: (*label).into(), value: (*value).into() })
                            .unwrap()
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
        },
    )
    .unwrap();
    if rule {
        w::View::Rule(w::RuleView::new(&w::CEILINGS, w::RuleViewParts { kind: 2, number: 1, card }).unwrap())
    } else {
        w::View::Charter(w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter: 4, card }).unwrap())
    }
}
fn role_read(out: &Queue<d::Request>) -> Option<Token> {
    use temper_client_protocol_wire as wire;
    let mut found = None;
    for request in out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
            && let w::Read::View(_) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
        {
            assert!(found.is_none(), "one view read");
            found = Some(*read);
        }
    }
    found
}
fn role_answer(root: &mut d::Domain, read: Token, answer: jig_client::Readout) -> Queue<d::Request> {
    drive(
        root,
        d::Event::Client { event: jig_client::Event::Up { up: jig_client::Up::ReadDone { read, result: answer } } },
    )
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table covers the two pages, read states and revision refreshes")]
fn role_pages_draw_their_card_or_the_read_state() {
    use temper_client_protocol_wire as wire;
    let refused = jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused));
    let not_drawn = w::View::NotDrawn(
        w::NotDrawn::new(&w::CEILINGS, w::NotDrawnParts { words: b"No view".as_slice().into() }).unwrap(),
    );
    let rows = [
        (
            Address::Role { project: f::PROJECT, charter: 4 },
            Some(role_card(false)),
            None,
            "The reviewer",
            &[b"Model".as_slice(), b"Can", b"Asked to", b"Holds"][..],
        ),
        (
            Address::Rule { project: f::PROJECT, kind: 2, number: 1 },
            Some(role_card(true)),
            None,
            "The project's rule 1",
            &[b"Covers".as_slice()][..],
        ),
        (Address::Role { project: f::PROJECT, charter: 4 }, None, None, "Reading role 4…", &[][..]),
        (Address::Rule { project: f::PROJECT, kind: 2, number: 1 }, None, None, "Reading rule 2/1…", &[][..]),
        (
            Address::Role { project: f::PROJECT, charter: 4 },
            None,
            Some(refused.clone()),
            "This role can't be read now: charter 4.",
            &[][..],
        ),
        (
            Address::Rule { project: f::PROJECT, kind: 2, number: 1 },
            None,
            Some(refused),
            "This rule can't be read now: rule 2/1.",
            &[][..],
        ),
        (
            Address::Role { project: f::PROJECT, charter: 4 },
            Some(not_drawn.clone()),
            None,
            "This role can't be read now: charter 4.",
            &[][..],
        ),
        (
            Address::Rule { project: f::PROJECT, kind: 2, number: 1 },
            Some(not_drawn),
            None,
            "This rule can't be read now: rule 2/1.",
            &[][..],
        ),
    ];
    for (address, value, refusal, expected, terms) in rows {
        let (mut root, _, _, _, _, _, settings_watch, ..) = f::root_at_with_values(
            settings(),
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
        let out = drive(&mut root, d::Event::Went { place: d::Place::Address(address.clone()) });
        let read = role_read(&out).expect("view read");
        let answer = value
            .clone()
            .map(|view| jig_client::Readout::Family {
                family: wire::FORGE,
                record: Some(wire::view_answer(
                    &w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view }).unwrap(),
                )),
            })
            .or(refusal);
        let card_shown = match &value {
            Some(w::View::Charter(_) | w::View::Rule(_)) => value.as_ref(),
            _ => None,
        };
        let pending = answer.is_none();
        if let Some(answer) = answer {
            role_answer(&mut root, read, answer);
        }
        assert_role_page(&root, &address, value.as_ref(), expected, terms);
        // A settings revision rereads the open address, retaining the last card.
        let out = drive(
            &mut root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: settings_watch.unwrap(),
                        event: c::WatchEvent::Delivered(c::Delivery::Settings(Box::new(c::SettingsDocument {
                            newest: 1,
                            settings: List::with_capacity(0),
                            held: List::with_capacity(0),
                            pending: List::with_capacity(0),
                        }))),
                    },
                },
            },
        );
        assert_eq!(role_read(&out).is_some(), !pending, "{address:?}: newer revision");
        if let Some(card) = card_shown {
            let pages::roles::Read::Drawn(shown) = root.pages().roles().unwrap().read() else {
                panic!("card retained while rereading")
            };
            assert_eq!(shown, card);
        }
        if pending {
            let answer = w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view: role_card(false) }).unwrap();
            let out = role_answer(
                &mut root,
                read,
                jig_client::Readout::Family { family: wire::FORGE, record: Some(wire::view_answer(&answer)) },
            );
            assert!(role_read(&out).is_some(), "revision while pending is read after the answer");
        }
    }
}

fn assert_role_page(root: &d::Domain, address: &Address, value: Option<&w::View>, expected: &str, terms: &[&[u8]]) {
    for (browser, mut view) in [(false, f::page_view(root)), (true, f::browser_page_view(root))] {
        let tree = view.tree();
        assert_eq!(skein_ui::outline::check(tree), None);
        assert_eq!(main(tree)[0].styles.as_slice(), &[look::MAIN, look::SETTINGS]);
        let text = String::from_utf8_lossy(&f::nodes_text(main(tree))).into_owned();
        assert!(text.contains(expected), "{address:?}: {text}");
        let headings: Vec<_> = main(tree)
            .iter()
            .filter(|n| n.role == Role::Heading { level: 1 })
            .map(|n| n.name.as_deref().unwrap())
            .collect();
        let wanted_headings = if terms.is_empty() { vec![] } else { vec![expected.as_bytes()] };
        assert_eq!(headings, wanted_headings);
        assert_eq!(
            main(tree).iter().filter(|n| n.role == Role::Term).map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
            terms
        );
        if let Some(w::View::Charter(card)) = value {
            assert!(text.contains(core::str::from_utf8(card.card().who()).unwrap()));
            for fact in card.card().facts() {
                assert!(text.contains(core::str::from_utf8(fact.value()).unwrap()));
            }
        }
        if let Some(w::View::Rule(card)) = value {
            assert!(text.contains(core::str::from_utf8(card.card().who()).unwrap()));
            for fact in card.card().facts() {
                assert!(text.contains(core::str::from_utf8(fact.value()).unwrap()));
            }
        }
        let tabs = styled(tree, look::SETTINGS_TABS);
        assert!(tabs.iter().any(|n| n.role == Role::Link
            && n.name.as_deref() == Some(b"How work runs")
            && n.states.as_slice().contains(&State::Current)));
        let back = main(tree).iter().find(|n| n.styles.as_slice().contains(&look::BACK));
        assert_eq!(back.is_some(), browser);
        if let Some(back) = back {
            assert_eq!(back.name.as_deref(), Some(b"How work runs".as_slice()));
            let id = back.id;
            assert_eq!(crate::href(&mut view, id), Some(d::address::print(&settings())));
        }
    }
}

#[test]
fn the_questions_select_has_only_two_routes() {
    let root = how(Some(SHOWN));
    for browser in [false, true] {
        let view = how_view(&root, browser);
        let row = setting_row(view.tree(), b"Questions a goal can answer from its plan");
        let options: Vec<_> = row.iter().filter(|node| node.role == Role::Option).map(|node| node.name.as_deref()).collect();
        assert_eq!(options, [Some(b"Asked of me too".as_slice()), Some(b"Not asked: each agent decides alone".as_slice())]);
    }
}
