//! Settings' tabs, repository facts and sign out (temper/docs/design/client/README.md, 3.5).
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
            "Everything the other pages do for this project comes from here, so each project, and each person, can work its own way.",
            0,
        ),
        (repositories(), "What Shop works on.", "The repositories chats and goals in Shop read and change.", 1),
        (
            Address::Forges,
            "Where your code lives.",
            "Each forge temper is connected to, or setting up. Add one to bring its repositories to temper.",
            2,
        ),
        (
            Address::Accounts,
            "What agents run on.",
            "The LLM accounts temper’s agents run on for you. A task keeps its account: while the account is limited, the task waits for it.",
            3,
        ),
        (
            Address::Models { project: f::PROJECT },
            "What the picker offers.",
            "The models your accounts can run, by what pays for them. Each model can be left off the picker; a task keeps the model it has.",
            4,
        ),
        (
            Address::Builders { project: f::PROJECT },
            "Where agents work.",
            "The machines temper’s agents run on, each with its slots and how many are in use.",
            5,
        ),
    ] {
        let root = root(address);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            assert_eq!(skein_ui::outline::check(tree), None);
            assert_eq!(main(tree)[0].styles.as_slice(), &[look::MAIN, look::SETTINGS]);
            let col = styled(tree, look::SETTINGS_COL);
            let intro = styled(tree, look::SETTINGS_INTRO);
            assert_eq!(col[1].id, intro[0].id);
            let text = String::from_utf8_lossy(&f::nodes_text(intro)).into_owned();
            assert_eq!(text, format!("Shop · settings{heading}{lede}"));
            assert_eq!(intro.iter().filter(|n| n.role == Role::Heading { level: 1 }).count(), 1);
            let nav = styled(tree, look::SETTINGS_TABS);
            assert_eq!(nav[0].role, Role::Navigation);
            assert_eq!(nav[0].name.as_deref(), Some(b"Settings".as_slice()));
            let links: Vec<_> = nav.iter().filter(|n| n.role == Role::Link).collect();
            assert_eq!(
                links.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
                [b"How work runs".as_slice(), b"Repositories", b"Forges", b"Accounts", b"Models", b"Builders"]
            );
            let ids: Vec<_> = links
                .iter()
                .enumerate()
                .map(|(index, n)| {
                    assert_eq!(n.states.as_slice().contains(&State::Current), index == current);
                    n.id
                })
                .collect();
            for (id, address) in ids.into_iter().zip([
                Address::Settings { project: f::PROJECT },
                repositories(),
                Address::Forges,
                Address::Accounts,
                Address::Models { project: f::PROJECT },
                Address::Builders { project: f::PROJECT },
            ]) {
                assert_eq!(crate::href(&mut view, id), Some(d::address::print(&address)));
                assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Pages(pages::Act::Go { address })));
            }
        }
    }
}
#[test]
fn start_goes_to_configure_and_go_on_to_the_open_chat() {
    let (mut root, me_watch, _) = f::root_at(Address::Settings { project: f::PROJECT }, &f::domain_limits());
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let start = find(view.tree(), Role::Link, b"Start");
        assert_eq!(crate::href(&mut view, start).as_deref(), Some(b"/projects/1/settings/configure".as_slice()));
        assert_eq!(
            crate::act(&mut view, start, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::Configure { project: f::PROJECT } }))
        );
        let row = styled(view.tree(), look::SETTINGS_SETTING);
        assert!(row.iter().any(|n| n.name.as_deref() == Some(b"Configure temper")));
        assert!(row.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_DETAIL) && n.name.as_deref() == Some(b"Tell it how you work, in your own words or by pointing at a document. It proposes the settings; nothing changes until you accept.")));
        assert!(row.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_END)));
        assert!(!main(view.tree()).iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Sign out")));
    }
    let template = f::chat::messaging_root();
    f::deliver_me(&mut root, me_watch, f::configuring_me(template.pages().me(template.client()).unwrap().clone()));
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
        let go = find(view.tree(), Role::Link, b"Go on");
        assert_eq!(crate::href(&mut view, go), Some(d::address::print(&Address::Chat { task })));
        assert_eq!(
            crate::act(&mut view, go, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::Chat { task } }))
        );
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
        let adopt = find(view.tree(), Role::Link, b"Adopt a repository");
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
        assert_eq!(f::nodes_text(band).as_ref(), b"No repository yetChats in Shop talk and plan, but change no code until a repository is in.Adopt a repository");
        let links: Vec<_> = main(view.tree())
            .iter()
            .filter(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Adopt a repository"))
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
fn accounts_are_in_the_settings_column_with_or_without_a_project() {
    for (address, heading, style) in [(Address::Accounts, b"Accounts".as_slice(), look::ACCOUNTS_PAGE)] {
        let (mut root, watch, _) = f::root_at(address, &f::domain_limits());
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_COL)));
            assert!(
                !main(view.tree())
                    .iter()
                    .any(|n| n.role == Role::Heading { level: 1 } && n.name.as_deref() == Some(heading))
            );
            assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::PAGEBAR)));
            styled(view.tree(), style);
        }
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.projects.clear();
        f::deliver_me(&mut root, watch, me);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            find(view.tree(), Role::Heading { level: 1 }, heading);
            assert!(main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_COL)));
            assert_eq!(main(view.tree())[0].styles.as_slice(), &[look::MAIN, look::SETTINGS]);
        }
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
const SHOWN: Ways = Ways { drafts: true, goals: true, changes: true, away: true, way: 11 };
const SWITCHES: [(&str, &str, &str); 4] = [
    (
        "Chats can write drafts",
        "A chat keeps a document beside it, in docs/design/draft/, that a goal can start from.",
        "On",
    ),
    ("Goals", "Bigger work from a draft: a plan, tasks, landing.", "Shown"),
    ("Changes", "The landing queue, as a page.", "Shown"),
    ("Away switch", "Lets agents decide within limits while you’re away.", "Shown"),
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
    Ways {
        drafts: value.drafts(),
        goals: value.goals(),
        changes: value.changes(),
        away: value.away(),
        way: value.way(),
    }
}
#[test]
fn the_shows_section_reads_kind_seven() {
    for shown in [Ways::STOCK, SHOWN] {
        let root = how(Some(shown));
        for browser in [false, true] {
            let view = how_view(&root, browser);
            assert_eq!(skein_ui::outline::check(view.tree()), None);
            let section = section_nodes(view.tree(), b"What the project shows");
            assert_eq!(section.iter().filter(|n| n.role == Role::Check).count(), 4);
            for (index, (label, small, on)) in SWITCHES.iter().enumerate() {
                let row = setting_row(view.tree(), label.as_bytes());
                assert!(row.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_DETAIL)
                    && n.name.as_deref() == Some(small.as_bytes())));
                let check = node(view.tree(), find(view.tree(), Role::Check, label.as_bytes()));
                assert_eq!(check.states.as_slice().contains(&State::Pressed), shown == SHOWN, "{label}");
                assert!(!check.states.as_slice().contains(&State::Disabled));
                assert!(check.styles.as_slice().contains(&look::SETTINGS_SWITCH));
                let toggle = row.iter().find(|n| n.styles.as_slice().contains(&look::SETTINGS_TOGGLE_ROW)).unwrap();
                let text = f::nodes_text(subtree(view.tree(), toggle.id));
                assert_eq!(
                    text.as_ref(),
                    if shown == SHOWN {
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
                3 => expected.away = false,
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
            ("Answered by the goal, kept in its decisions", w::QuestionsRoute::Foreman, b"foreman".as_slice()),
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
                w::QuestionsRoute::Foreman => Route::Foreman,
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
        assert_eq!(controls.len(), 6);
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
fn disabled_sections_say_so_once_and_their_controls_are_disabled() {
    let root = how(Some(SHOWN));
    for browser in [false, true] {
        let view = how_view(&root, browser);
        for (title, labels, values) in [
            (
                "Status",
                vec!["Status lines and reports", "Rewritten", "The brief on Home"],
                vec!["Written by each goal’s agent", "When someone opens it", "On"],
            ),
            ("Telling you", vec!["Notifications", "Quiet hours"], vec!["Off", "None"]),
        ] {
            let section = section_nodes(view.tree(), title.as_bytes());
            let facts: Vec<_> =
                section.iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_FACTS)).collect();
            assert_eq!(facts.len(), 1);
            assert_eq!(facts[0].name.as_deref(), Some(b"Not built yet: this is what temper does today.".as_slice()));
            assert_eq!(section.last().unwrap().id, subtree(view.tree(), facts[0].id).last().unwrap().id);
            for (label, value) in labels.into_iter().zip(values) {
                let id = find(view.tree(), Role::Select, label.as_bytes());
                assert!(node(view.tree(), id).states.as_slice().contains(&State::Disabled));
                assert_eq!(selected(view.tree(), label.as_bytes()), value.as_bytes());
                assert_eq!(subtree(view.tree(), id).iter().filter(|n| n.role == Role::Option).count(), 1);
            }
        }
    }
}
#[test]
fn funds_says_so_in_its_small() {
    let root = how(Some(SHOWN));
    let view = how_view(&root, true);
    let row = setting_row(view.tree(), b"Funds");
    assert!(row.iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_DETAIL)
        && n.name.as_deref() == Some(b"Not built yet: this is what temper does today.")));
    assert_eq!(selected(view.tree(), b"Funds"), b"Ask when a budget is used up");
    assert!(node(view.tree(), find(view.tree(), Role::Select, b"Funds")).states.as_slice().contains(&State::Disabled));
}
#[test]
fn until_read_the_switches_are_disabled_and_unchecked() {
    for root in f::pair(how(None), f::root_with_refused_ways(settings())) {
        for browser in [false, true] {
            let mut view = how_view(&root, browser);
            for (label, ..) in SWITCHES {
                let id = find(view.tree(), Role::Check, label.as_bytes());
                let check = node(view.tree(), id);
                assert!(check.states.as_slice().contains(&State::Disabled));
                assert!(!check.states.as_slice().contains(&State::Pressed));
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
fn alone_it_keeps_its_heading_and_lede_with_no_tabs() {
    let (mut root, watch, _) = f::root_at(Address::Forges, &f::domain_limits());
    for project in [true, false] {
        if !project {
            let mut me = root.pages().me(root.client()).unwrap().clone();
            me.projects.clear();
            f::deliver_me(&mut root, watch, me);
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(main(view.tree())[0].styles.as_slice(), &[look::MAIN, look::SETTINGS]);
            let col = styled(view.tree(), look::SETTINGS_COL);
            let intro = styled(view.tree(), look::SETTINGS_INTRO);
            assert!(col.len() > intro.len());
            find(view.tree(), Role::Heading { level: 1 }, if project { b"Where your code lives." } else { b"Forges" });
            assert_eq!(
                intro.iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_KICKER)).count(),
                usize::from(project)
            );
            assert_eq!(
                main(view.tree()).iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_TABS)).count(),
                usize::from(project)
            );
            find(
                view.tree(),
                Role::Paragraph,
                b"Each forge temper is connected to, or setting up. Add one to bring its repositories to temper.",
            );
        }
    }
}

#[test]
fn each_flow_draws_its_path_in_the_browser_and_none_in_the_terminal() {
    for (address, tab, to, here) in [
        (Address::ConnectForge { forge: None }, b"Forges".as_slice(), b"/forges".as_slice(), b"Add a forge".as_slice()),
        (
            Address::ConnectModel { way: None },
            b"Accounts".as_slice(),
            b"/accounts".as_slice(),
            b"Connect an account".as_slice(),
        ),
        (
            Address::Adopt { project: f::PROJECT },
            b"Repositories".as_slice(),
            b"/projects/1/settings/repositories".as_slice(),
            b"Adopt a repository".as_slice(),
        ),
    ] {
        let root = root(address);
        let terminal = f::page_view(&root);
        assert!(!terminal.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_PATH)));
        let mut browser = f::browser_page_view(&root);
        let path = styled(browser.tree(), look::SETTINGS_PATH);
        assert!(path.iter().any(|n| n.role == Role::Navigation && n.name.as_deref() == Some(b"Where")));
        let links: Vec<_> = path.iter().filter(|n| n.role == Role::Link).map(|n| (n.id, n.name.clone())).collect();
        let expected = [
            (tab, to),
            (b"Shop".as_slice(), b"/projects/1".as_slice()),
            (b"Settings".as_slice(), b"/projects/1/settings".as_slice()),
            (tab, to),
        ];
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
                .any(|n| n.name.as_deref() == Some(b"Settings") && n.states.as_slice().contains(&State::Current))
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

const AWAY_TITLE: &str = "While you’re away, agents may decide";
const AWAY_KINDS: [(&str, u8); 4] = [
    ("Implementation choices within a task", 1),
    ("Retries and splitting a failing task", 2),
    ("A design document’s wording", 4),
    ("Adding a dependency", 8),
];
const LOCKED: &str = "Budgets, sign-ins, machines, what is hard to reverse: always wait";
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
fn away_row(kinds: u8, until: Option<u64>, off: Option<u64>) -> w::AwayRowParts {
    w::AwayRowParts { project: f::PROJECT, since: 0, until, kinds, landed: 0, off }
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
fn away_send(root: &mut d::Domain, view: &mut crate::View, role: Role, name: &[u8]) -> (Token, w::SetAwayParts) {
    let id = find(view.tree(), role, name);
    let event = crate::act_event(view, id, if role == Role::Check { Act::Toggle } else { Act::Press }).unwrap();
    let mut out = drive(root, event);
    let mut stored = drive(root, d::Event::Client { event: jig_client::Event::Stored });
    while let Some(request) = stored.pop() {
        out.push(request);
    }
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
fn away_answer(root: &mut d::Domain, attempt: Token) {
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
                            &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Away }).unwrap(),
                        ),
                    })),
                },
            },
        },
    );
}
fn drawn_choices(view: &crate::View, kinds: u8, until: &[u8]) {
    for (label, bit) in AWAY_KINDS {
        let id = find(view.tree(), Role::Check, label.as_bytes());
        assert_eq!(node(view.tree(), id).states.as_slice().contains(&State::Pressed), kinds & bit != 0, "{label}");
    }
    assert_eq!(selected(view.tree(), b"Away until"), until);
}
#[test]
fn the_section_follows_what_needs_you() {
    let (root, _) = away_settings(None);
    for view in [how_view(&root, false), how_view(&root, true)] {
        let headings: Vec<_> = main(view.tree())
            .iter()
            .filter(|n| n.role == Role::Heading { level: 2 })
            .map(|n| n.name.as_deref().unwrap())
            .collect();
        let at = headings.iter().position(|name| *name == AWAY_TITLE.as_bytes()).unwrap();
        assert_eq!(headings[at - 1], b"What needs you");
        assert_eq!(headings[at + 1], b"Telling you");
        assert_eq!(
            section_nodes(view.tree(), AWAY_TITLE.as_bytes())
                .iter()
                .filter(|n| n.styles.as_slice().contains(&look::SETTINGS_SETTING))
                .count(),
            2
        );
        assert_eq!(
            f::nodes_text(setting_row(view.tree(), b"Kinds of decision")).as_ref(),
            b"Kinds of decisionEach shows afterwards as decided for you, with Overturn."
        );
        assert_eq!(
            f::nodes_text(setting_row(view.tree(), b"Away until")).as_ref(),
            "Away untilWhen it isn’t turned off by hand.".as_bytes()
        );
    }
}
#[test]
fn no_section_while_no_switch() {
    let root = how(Some(Ways::STOCK));
    for view in [how_view(&root, false), how_view(&root, true)] {
        assert!(!main(view.tree()).iter().any(|n| n.name.as_deref() == Some(AWAY_TITLE.as_bytes())));
    }
}
#[test]
fn defaults_with_no_row() {
    let (root, _) = away_settings(None);
    for view in [how_view(&root, false), how_view(&root, true)] {
        drawn_choices(&view, 7, EIGHT);
    }
}
#[test]
fn a_kind_writes_through_set_away() {
    for browser in [false, true] {
        for (label, bit) in AWAY_KINDS {
            let (mut root, _) = away_settings(Some(away_row(10, None, None)));
            let mut view = how_view(&root, browser);
            let (_, sent) = away_send(&mut root, &mut view, Role::Check, label.as_bytes());
            assert_eq!(sent, w::SetAwayParts { project: f::PROJECT, on: true, until: None, kinds: 0b1010 ^ bit });
        }
    }
}
#[test]
fn the_select_writes_until() {
    for browser in [false, true] {
        for (label, until) in [(EIGHT, Some(28_800_000_000_000)), (b"I turn it off".as_slice(), None)] {
            let (mut root, _) = away_settings(Some(away_row(8, None, None)));
            let mut view = how_view(&root, browser);
            let (_, sent) = away_send(&mut root, &mut view, Role::Option, label);
            assert_eq!(sent, w::SetAwayParts { project: f::PROJECT, on: true, until, kinds: 8 });
        }
    }
}
#[test]
fn it_works_while_off() {
    for browser in [false, true] {
        for (role, label, kinds, eight, until) in [
            (Role::Check, AWAY_KINDS[0].0.as_bytes(), 6, true, Some(28_800_000_000_000)),
            (Role::Check, AWAY_KINDS[1].0.as_bytes(), 5, true, Some(28_800_000_000_000)),
            (Role::Check, AWAY_KINDS[2].0.as_bytes(), 3, true, Some(28_800_000_000_000)),
            (Role::Check, AWAY_KINDS[3].0.as_bytes(), 15, true, Some(28_800_000_000_000)),
            (Role::Option, EIGHT, 7, true, Some(28_800_000_000_000)),
            (Role::Option, b"I turn it off".as_slice(), 7, false, None),
        ] {
            let (mut root, watch) = away_settings(Some(away_row(7, Some(28_800_000_000_000), Some(0))));
            let mut view = how_view(&root, browser);
            let (attempt, sent) = away_send(&mut root, &mut view, role, label);
            assert_eq!(sent, w::SetAwayParts { project: f::PROJECT, on: false, until, kinds });
            away_answer(&mut root, attempt);
            away_deliver(&mut root, watch, Some(away_row(sent.kinds, sent.until, Some(0))));
            let view = how_view(&root, browser);
            drawn_choices(&view, kinds, if eight { EIGHT } else { b"I turn it off" });
            assert!(!root.away_on(f::PROJECT, f::NOW));
        }
    }
}
#[test]
fn locked_stays_unticked_and_disabled() {
    let (root, _) = away_settings(Some(away_row(15, None, None)));
    for mut view in [how_view(&root, false), how_view(&root, true)] {
        let id = find(view.tree(), Role::Check, LOCKED.as_bytes());
        assert_eq!(node(view.tree(), id).states.as_slice(), &[State::Disabled]);
        assert_eq!(crate::act(&mut view, id, Act::Toggle), None);
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
    let (root, _) = away_settings(Some(away_row(8, Some(0), None)));
    for view in [how_view(&root, false), how_view(&root, true)] {
        drawn_choices(&view, 8, EIGHT);
    }
}
#[test]
fn away_controls_are_numbered_in_the_terminal() {
    let (root, _) = away_settings(None);
    let view = how_view(&root, false);
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    let controls: Vec<_> = print.numbers.as_slice().to_vec();
    for (role, label) in [
        (Role::Check, AWAY_KINDS[0].0.as_bytes()),
        (Role::Check, AWAY_KINDS[1].0.as_bytes()),
        (Role::Check, AWAY_KINDS[2].0.as_bytes()),
        (Role::Check, AWAY_KINDS[3].0.as_bytes()),
        (Role::Option, EIGHT),
        (Role::Option, b"I turn it off".as_slice()),
    ] {
        assert!(controls.contains(&find(view.tree(), role, label)), "numbered {label:?}");
    }
}
