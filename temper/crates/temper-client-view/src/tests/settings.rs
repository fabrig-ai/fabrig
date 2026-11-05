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
                [b"How work runs".as_slice(), b"Repositories", b"Forges", b"Accounts"]
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
