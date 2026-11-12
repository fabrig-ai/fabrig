use super::fixtures;
use skein_ui::{Act, Role};
use temper_client_domain::ChatsFilter;
use temper_client_domain::{Action, pages};

#[test]
fn the_top_bar_says_the_links_state_and_the_persons_name() {
    use temper_client_domain::{Address, Event, Step, pages::terminal::Look};
    let mut terminal = fixtures::reading_me_at(Address::NewChat { draft: false });
    fixtures::root_event(&mut terminal, Event::LookKept { look: Look { terminal: true, ..Look::default() } });
    let roots = [
        (Box::new(fixtures::no_project_root()), false),
        (Box::new(temper_client_domain::new(&fixtures::domain_limits(), fixtures::SEED)), false),
        (Box::new(fixtures::signed_out()), false),
        (Box::new(fixtures::out_of_date()), false),
        (Box::new(fixtures::reading_me_at(Address::Chats { filter: ChatsFilter::All })), true),
        (Box::new(fixtures::reading_me_at(Address::NewChat { draft: false })), true),
        (Box::new(fixtures::reading_me_at(Address::Welcome { step: Step::Welcome })), true),
        (Box::new(terminal), true),
    ];
    let mut wrong_main = skein_lib::List::uncapped();
    for (root, reading) in roots {
        for words in [fixtures::words(), fixtures::browser_words()] {
            let shell = words.shell;
            let mut view = fixtures::view_of(words);
            fixtures::build(&mut view, &root);
            assert_eq!(fixtures::styles(view.tree(), Role::Banner).as_ref(), [Box::from([crate::look::TOP])]);
            assert!(fixtures::find(view.tree(), Role::Disclosure, b"Menu").is_none());
            let mut mains = 0_u32;
            let mut wrong = false;
            for node in view.tree().nodes() {
                if node.role == Role::Main {
                    mains = mains.checked_add(1).expect("tree nodes fit");
                    wrong |= (node.size == 1) != reading
                        || node.states.as_slice().contains(&skein_ui::State::Busy) != reading;
                }
                assert_ne!(node.role, Role::Navigation);
                assert!(!node.styles.as_slice().contains(&crate::look::PROJECTS));
                if reading && node.role == Role::Region {
                    assert_eq!(node.region, Some(crate::frame::link_region()));
                }
            }
            if mains != 1 || wrong {
                wrong_main.push((root.pages().address().clone(), shell, reading)).expect("failed rows fit");
            }
        }
    }
    assert!(wrong_main.is_empty(), "one main, empty and busy exactly while reading me: {wrong_main:?}");
    let view = fixtures::page_view(&fixtures::no_project_root());
    assert!(fixtures::find(view.tree(), Role::Status, b"Live").is_some());
    assert!(fixtures::find(view.tree(), Role::Status, b"Pat").is_some());
}

#[test]
fn the_project_chosen_is_the_first_of_me_at_start() {
    let root = fixtures::root_in_project(None);
    assert_eq!(root.pages().project(), Some(fixtures::PROJECT));
}
#[test]
fn the_running_playground_project_button_is_named_by_its_record() {
    let root = fixtures::playground_root(
        temper_client_domain::Address::Chats { filter: ChatsFilter::All },
        fixtures::PROJECT,
        true,
    );
    for words in [fixtures::words(), fixtures::browser_words()] {
        let label = b"Shop";
        let mut view = fixtures::view_of(words);
        fixtures::build(&mut view, &root);
        let button = fixtures::person_find(view.tree(), Role::Button, label).expect("stored project row words");
        assert_eq!(
            crate::act(&mut view, button, Act::Press),
            Some(Action::Pages(pages::Act::Project { project: fixtures::PROJECT }))
        );
        assert!(fixtures::find(view.tree(), Role::Button, b"Playground").is_none());
        assert!(fixtures::find(view.tree(), Role::Paragraph, b"No project yet").is_none());
    }
}

#[test]
fn no_project_chosen_offers_no_adoption_link() {
    let view = fixtures::built();
    assert!(fixtures::find(view.tree(), Role::Link, b"Adopt a repository").is_none());
}
#[test]
fn adopt_a_repository_is_offered_whatever_the_persons_role() {
    let mut view = fixtures::page_view(
        &fixtures::root_at(
            temper_client_domain::Address::Repositories { project: fixtures::PROJECT },
            &fixtures::domain_limits(),
        )
        .0,
    );
    let node = fixtures::find(view.tree(), Role::Link, b"Add repositories").expect("adoption link");
    assert_eq!(
        crate::act(&mut view, node, Act::Press),
        Some(Action::Pages(pages::Act::Go {
            address: temper_client_domain::Address::Adopt { project: fixtures::PROJECT },
        }))
    );
}
#[test]
fn the_top_bar_says_connecting_until_me_then_live_then_offline() {
    let view = fixtures::built();
    assert!(fixtures::find(view.tree(), Role::Status, b"Connecting").is_some());
    let (mut root, watch) = fixtures::root_with_watch(None);
    let mut view = fixtures::view();
    assert!(fixtures::find(fixtures::build(&mut view, &root), Role::Status, b"Live").is_some());
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
        },
    );
    assert!(fixtures::find(fixtures::build(&mut view, &root), Role::Status, b"Offline").is_some());
}
#[test]
fn choosing_another_project_changes_the_navigations_project() {
    let mut root = fixtures::root_in_project(None);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: Action::Pages(pages::Act::Project { project: fixtures::SECOND_PROJECT }),
        },
    );
    let mut view = fixtures::view();
    fixtures::build(&mut view, &root);
    assert_rail(&view, b"Second");
    let link = fixtures::find(view.tree(), Role::Link, b"Home").expect("across Home");
    assert_eq!(
        crate::act(&mut view, link, Act::Press),
        Some(Action::Pages(pages::Act::Go { address: temper_client_domain::Address::AllHome }))
    );
}
#[test]
fn a_sign_out_that_did_not_reach_the_engine_is_offered_again() {
    let mut root = fixtures::root_at(temper_client_domain::Address::Accounts, &fixtures::domain_limits()).0;
    fixtures::root_event(&mut root, temper_client_domain::Event::Act { action: Action::Pages(pages::Act::SignOut) });
    fixtures::root_event(&mut root, temper_client_domain::Event::Client { event: jig_client::Event::SignOutLost });
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Alert, b"Sign out did not reach the engine.").is_some());
    assert!(fixtures::find(tree, Role::Button, b"Sign out").is_some());
    assert_rail(&view, b"Shop");
}
#[test]
fn a_page_behind_says_behind_after_the_links_state() {
    let (mut root, watch) = fixtures::root_with_watch(None);
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.projects.clear();
    fixtures::deliver_me(&mut root, watch, me);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
        },
    );
    for words in [fixtures::words(), fixtures::browser_words()] {
        let terminal = words.shell == crate::words::Shell::Terminal;
        let mut view = fixtures::view_of(words);
        let tree = fixtures::build(&mut view, &root);
        assert_eq!(
            fixtures::frame_statuses(tree).as_ref(),
            [Box::from(*b"Offline"), Box::from(*b"Behind"), Box::from(*b"Pat")]
        );
        let nodes = tree.nodes();
        let mut region = None;
        let mut behind = None;
        for (index, node) in nodes.iter().enumerate() {
            if node.region == Some(crate::frame::link_region()) {
                region = Some(index);
            }
            if node.styles.as_slice().contains(&crate::look::BEHIND) {
                behind = Some(index);
            }
        }
        let region = region.expect("link region");
        let behind = behind.expect("behind status");
        let end = region.checked_add(usize::try_from(nodes[region].size).unwrap()).unwrap();
        assert_eq!(behind < end, terminal, "Behind belongs to the link region only in the terminal");
    }
}

#[test]
fn signed_out_the_main_says_so_and_nothing_else() {
    let view = fixtures::page_view(&fixtures::signed_out());
    let nodes = view.tree().nodes();
    let mut main = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Main {
            let start = index.checked_add(1).expect("heading follows main");
            let end = index.checked_add(usize::try_from(node.size).expect("main size")).expect("main end");
            main = Some((start, end));
            break;
        }
    }
    let (start, end) = main.expect("main");
    let mut content = skein_lib::List::with_capacity(2);
    for (index, node) in nodes.iter().enumerate().take(end).skip(start) {
        if node.role != Role::Text {
            content
                .push((node.role, node.name.as_deref(), fixtures::node_text(view.tree(), index)))
                .expect("heading and paragraph");
        }
    }
    assert_eq!(
        content.as_slice(),
        [(Role::Heading { level: 1 }, Some(b"Signed out".as_slice()), Box::from(b"Signed out".as_slice()))]
    );
}
#[test]
fn signed_out_the_browser_main_says_how_to_sign_in() {
    let view = fixtures::browser_page_view(&fixtures::signed_out());
    let nodes = view.tree().nodes();
    let mut main = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Main {
            let start = index.checked_add(1).expect("heading follows main");
            let end = index.checked_add(usize::try_from(node.size).expect("main size")).expect("main end");
            main = Some((start, end));
            break;
        }
    }
    let (start, end) = main.expect("main");
    let mut content = skein_lib::List::with_capacity(2);
    for (index, node) in nodes.iter().enumerate().take(end).skip(start) {
        if node.role != Role::Text {
            content
                .push((node.role, node.name.as_deref(), fixtures::node_text(view.tree(), index)))
                .expect("heading and paragraph");
        }
    }
    assert_eq!(
        content.as_slice(),
        [
            (Role::Heading { level: 1 }, Some(b"Signed out".as_slice()), Box::from(b"Signed out".as_slice())),
            (
                Role::Paragraph,
                Some(b"To sign in, open a sign-in link temper printed.".as_slice()),
                Box::from(b"To sign in, open a sign-in link temper printed.".as_slice())
            )
        ]
    );
}
#[test]
fn a_full_link_says_the_pool_and_both_moments_in_either_shell() {
    use jig_client_protocol_wire as w;
    let answer = w::SignInAnswer::Full(
        w::SignInsFull::new(
            &w::CEILINGS,
            w::SignInsFullParts { sign_ins: 2, earliest: 3_600_000_000_000, link_until: 300_000_000_000 },
        )
        .expect("full answer"),
    );
    assert_link_main(answer, b"All 2 sign-ins are in use; the earliest ends Jan 1, 01:00 UTC. This link stays usable until Jan 1, 00:05 UTC; after that, opening it has temper serve print a fresh one.");
}

#[test]
fn each_link_reason_has_its_paragraph_in_either_shell() {
    use jig_client_protocol_wire::SignInAnswer;
    for (answer, expected) in [
        (
            SignInAnswer::Busy,
            b"temper is busy. This link stays usable until it expires: open it again in a moment.".as_slice(),
        ),
        (
            SignInAnswer::NotReady,
            b"temper is starting or stopping. This link stays usable until it expires: open it again in a moment."
                .as_slice(),
        ),
        (SignInAnswer::LinkUsed, b"This link was already used. A fresh one is in your terminal.".as_slice()),
        (SignInAnswer::LinkExpired, b"This link has expired. A fresh one is in your terminal.".as_slice()),
        (SignInAnswer::LinkUnknown, b"This link isn't from this temper. Use the one its terminal printed.".as_slice()),
    ] {
        assert_link_main(answer, expected);
    }
}

fn assert_link_main(answer: jig_client_protocol_wire::SignInAnswer, expected: &[u8]) {
    use skein_lib::{Env, List, Queue, Secret, Time, Writer};
    use temper_client_domain::{Address, Event, Request};
    let limits = fixtures::domain_limits();
    let env = Env { now: Time::ZERO, wall: fixtures::NOW, limits };
    let mut root = temper_client_domain::new(&limits, fixtures::SEED);
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Start {
            address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Link { secret: Secret::new(Box::new([0; 32])), held: None }),
        },
        &mut out,
    );
    let mut exchange = None;
    while let Some(request) = out.pop() {
        if let Request::Client {
            request: jig_client::Request::Exchange { exchange: token, message: jig_client::Message::SignInLink(_), .. },
        } = request
        {
            assert!(exchange.replace(token).is_none());
        }
    }
    let mut writer = Writer::new(usize::try_from(answer.measure()).expect("answer size"));
    answer.encode(&mut writer).expect("answer room");
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Reply { exchange: exchange.expect("link exchange"), message: writer.finish() },
        },
    );
    assert_eq!(root.client().link().state, jig_client::Link::SignedOut);
    assert!(root.client().link().not_signed_in.is_some());
    for shell in [crate::words::Shell::Browser, crate::words::Shell::Terminal] {
        for google in [false, true] {
            let view = fixtures::sign_in_view(
                &root,
                shell,
                crate::sign_in::Ways { google, origin: Some(Box::from(b"https://temper.example.com".as_slice())) },
            );
            let nodes = view.tree().nodes();
            let mut main = None;
            for (index, node) in nodes.iter().enumerate() {
                if node.role == Role::Main {
                    let start = index.checked_add(1).expect("heading follows main");
                    let end = index.checked_add(usize::try_from(node.size).expect("main size")).expect("main end");
                    main = Some((start, end));
                    break;
                }
            }
            let (start, end) = main.expect("main");
            let mut content = List::with_capacity(2);
            for (index, node) in nodes.iter().enumerate().take(end).skip(start) {
                if node.role != Role::Text {
                    content
                        .push((node.role, node.name.as_deref(), fixtures::node_text(view.tree(), index)))
                        .expect("heading and paragraph");
                }
            }
            assert_eq!(
                content.as_slice(),
                [
                    (Role::Heading { level: 1 }, Some(b"Signed out".as_slice()), Box::from(b"Signed out".as_slice())),
                    (Role::Paragraph, Some(expected), Box::from(expected))
                ]
            );
        }
    }
}
#[test]
fn out_of_date_the_browser_main_keeps_its_words() {
    let view = fixtures::browser_page_view(&fixtures::out_of_date());
    let nodes = view.tree().nodes();
    let mut main = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Main {
            let start = index.checked_add(1).expect("heading follows main");
            let end = index.checked_add(usize::try_from(node.size).expect("main size")).expect("main end");
            main = Some((start, end));
            break;
        }
    }
    let (start, end) = main.expect("main");
    let mut content = skein_lib::List::with_capacity(2);
    for (index, node) in nodes.iter().enumerate().take(end).skip(start) {
        if node.role != Role::Text {
            content
                .push((node.role, node.name.as_deref(), fixtures::node_text(view.tree(), index)))
                .expect("heading and paragraph");
        }
    }
    assert_eq!(
        content.as_slice(),
        [
            (Role::Heading { level: 1 }, Some(b"Out of date".as_slice()), Box::from(b"Out of date".as_slice())),
            (
                Role::Paragraph,
                Some(b"Run the temper the engine runs.".as_slice()),
                Box::from(b"Run the temper the engine runs.".as_slice())
            )
        ]
    );
}
#[test]
fn the_browser_words_fit_the_views_limits() {
    assert!(crate::Limits::of(&fixtures::domain_limits(), &fixtures::browser_words()).is_some());
}
#[test]
fn signed_out_the_top_bar_says_the_links_state_alone() {
    let view = fixtures::page_view(&fixtures::signed_out());
    assert_eq!(fixtures::frame_statuses(view.tree()).as_ref(), [Box::from(*b"Signed out")]);
}
#[test]
fn signed_out_the_banner_offers_no_sign_out_button() {
    let view = fixtures::page_view(&fixtures::signed_out());
    assert!(fixtures::find(view.tree(), Role::Button, b"Sign out").is_none());
}
#[test]
fn signed_out_the_page_offers_no_navigation() {
    let view = fixtures::page_view(&fixtures::signed_out());
    for node in view.tree().nodes() {
        assert_ne!(node.role, Role::Navigation);
    }
}
#[test]
fn an_engine_not_serving_jigs_version_says_temper_is_out_of_date() {
    let view = fixtures::page_view(&fixtures::out_of_date());
    let nodes = view.tree().nodes();
    let mut main = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Main {
            let start = index.checked_add(1).expect("heading follows main");
            let end = index.checked_add(usize::try_from(node.size).expect("main size")).expect("main end");
            main = Some((start, end));
            break;
        }
    }
    let (start, end) = main.expect("main");
    let mut content = skein_lib::List::with_capacity(2);
    for (index, node) in nodes.iter().enumerate().take(end).skip(start) {
        if node.role != Role::Text {
            content
                .push((node.role, node.name.as_deref(), fixtures::node_text(view.tree(), index)))
                .expect("heading and paragraph");
        }
    }
    assert_eq!(
        content.as_slice(),
        [
            (Role::Heading { level: 1 }, Some(b"Out of date".as_slice()), Box::from(b"Out of date".as_slice())),
            (
                Role::Paragraph,
                Some(b"Run the temper the engine runs.".as_slice()),
                Box::from(b"Run the temper the engine runs.".as_slice())
            )
        ]
    );
}
#[test]
fn out_of_date_the_top_bar_says_the_links_state_alone() {
    let view = fixtures::page_view(&fixtures::out_of_date());
    assert_eq!(fixtures::frame_statuses(view.tree()).as_ref(), [Box::from(*b"Out of date")]);
}
#[test]
fn out_of_date_the_banner_offers_no_sign_out_button() {
    let view = fixtures::page_view(&fixtures::out_of_date());
    assert!(fixtures::find(view.tree(), Role::Button, b"Sign out").is_none());
}
#[test]
fn out_of_date_the_page_offers_no_navigation() {
    let view = fixtures::page_view(&fixtures::out_of_date());
    for node in view.tree().nodes() {
        assert_ne!(node.role, Role::Navigation);
    }
}
#[test]
fn each_link_shows_its_name_as_its_text() {
    let view = fixtures::page_view(&fixtures::no_project_root());
    let mut links = 0_u32;
    let nodes = view.tree().nodes();
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Link {
            assert_eq!(node.size, 2, "a link holds one text");
            let [run] = nodes[index + 1].text.as_slice() else { panic!("one run per link") };
            let expected = node.name.as_deref().unwrap();
            assert_eq!(run.text.as_ref(), expected);
            links += 1;
        }
    }
    assert_eq!(links, 1);
}

#[test]
fn the_navigation_marks_only_the_current_page() {
    use temper_client_domain::{Address, address::Filter};
    for (address, current) in [
        (Address::Inbox { filter: Filter::ALL }, Some(b"Needs you".as_slice())),
        (Address::Inbox { filter: Filter { goal: Some(9), ..Filter::ALL } }, Some(b"Needs you".as_slice())),
        (Address::Chats { filter: ChatsFilter::All }, Some(b"Work".as_slice())),
        (Address::Adopt { project: fixtures::PROJECT }, Some(b"How work runs".as_slice())),
        (Address::Accounts, Some(b"Deployment".as_slice())),
        (Address::Chat { task: 9 }, Some(b"Work".as_slice())),
        (Address::Task { task: 9 }, Some(b"Work".as_slice())),
    ] {
        let root = fixtures::root_at(address, &fixtures::domain_limits()).0;
        let view = fixtures::page_view(&root);
        let mut found = None;
        for node in view.tree().nodes() {
            if node.role == Role::Main {
                break;
            }
            if node.states.as_slice().contains(&skein_ui::State::Current) {
                assert_eq!(node.role, Role::Link);
                assert!(found.is_none(), "one current page");
                found = node.name.as_deref();
            }
        }
        assert_eq!(found, current);
    }
}

#[test]
fn the_signed_in_top_end_groups_link_person_and_sign_out_in_order() {
    let mut root = fixtures::no_project_root();
    let view = fixtures::page_view(&root);
    assert_top_end(&view, &[(Role::Region, None), (Role::Status, Some(b"Pat")), (Role::Button, Some(b"Sign out"))]);
    fixtures::root_event(&mut root, temper_client_domain::Event::Act { action: Action::Pages(pages::Act::SignOut) });
    fixtures::root_event(&mut root, temper_client_domain::Event::Client { event: jig_client::Event::SignOutLost });
    let view = fixtures::page_view(&root);
    assert_top_end(
        &view,
        &[
            (Role::Region, None),
            (Role::Status, Some(b"Pat")),
            (Role::Alert, Some(b"Sign out did not reach the engine.")),
            (Role::Button, Some(b"Sign out")),
        ],
    );
}
#[test]
fn the_unavailable_top_end_holds_only_the_link_region() {
    for root in fixtures::pair(fixtures::signed_out(), fixtures::out_of_date()) {
        let view = fixtures::page_view(&root);
        assert_top_end(&view, &[(Role::Region, None)]);
    }
}

pub(super) fn assert_top_end(view: &crate::View, children: &[(Role, Option<&[u8]>)]) {
    let nodes = view.tree().nodes();
    let mut corner = None;
    let mut banner_end = 0;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Banner {
            banner_end = index.checked_add(usize::try_from(node.size).expect("banner size")).expect("banner end");
        }
        if node.styles.as_slice().contains(&crate::look::TOP_END) {
            assert!(corner.is_none(), "one corner");
            assert_eq!(node.role, Role::Slot);
            assert!(index < banner_end, "corner is inside banner");
            corner = Some(index);
        }
    }
    let corner = corner.expect("corner");
    let end = corner.checked_add(usize::try_from(nodes[corner].size).expect("corner size")).expect("corner end");
    assert_eq!(end, banner_end, "corner finishes the banner");
    let mut index = corner.checked_add(1).expect("first child");
    for (role, name) in children {
        let node = &nodes[index];
        assert_eq!(node.role, *role);
        assert_eq!(node.name.as_deref(), *name);
        if *role == Role::Region {
            assert_eq!(node.region, Some(crate::frame::link_region()));
        }
        index = index.checked_add(usize::try_from(node.size).expect("child size")).expect("next child");
    }
    assert_eq!(index, end, "no loose corner contents");
}

#[test]
fn the_rail_holds_projects_navigation_and_foot_in_order() {
    let view = fixtures::signed_in_with_project();
    assert_rail(&view, b"Shop");
    assert_switcher(view.tree(), b"Shop", &[(b"Shop", true), (b"Second", false)]);
    let mut root = fixtures::root_at(
        temper_client_domain::Address::Team { project: fixtures::PROJECT },
        &fixtures::domain_limits(),
    )
    .0;
    super::rail::with_people(&mut root, 2);
    for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        assert_switcher(view.tree(), b"Shop", &[(b"Shop", true), (b"Second", false)]);
        let id = fixtures::find(view.tree(), Role::Link, b"Team for Shop").expect("Team in the switcher");
        assert_eq!(crate::href(&mut view, id).as_deref(), Some(b"/projects/1/team".as_slice()));
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(Action::Pages(pages::Act::Go {
                address: temper_client_domain::Address::Team { project: fixtures::PROJECT },
            }))
        );
    }
}

#[test]
fn home_and_needs_stay_across_projects_in_the_rail_and_phone_tabs() {
    use jig_client::contract as c;
    use skein_lib::List;
    use temper_client_domain::{Address, Event, Place, WorkFilter, address::Filter};
    let project = fixtures::SECOND_PROJECT;
    let (mut root, me_watch, _, inbox_watch) =
        fixtures::root_at_with_needs(Address::Work { project, filter: WorkFilter::All }, &fixtures::domain_limits());
    let me = root.pages().me(root.client()).unwrap().clone();
    assert_eq!(me.projects.len(), 2);
    fixtures::deliver_me(&mut root, me_watch, me);
    let mut rows = List::with_capacity(2);
    for project in [fixtures::PROJECT, fixtures::SECOND_PROJECT] {
        rows.push(temper_fixtures::client_documents::entry(
            c::ObjectKey::Question { task: u64::from(project), message: 1 },
            project,
            1,
            fixtures::NOW,
            None,
        ))
        .unwrap();
    }
    let inbox = temper_fixtures::client_documents::inbox(rows, 99, c::Count { value: 0, limited: false }, 0, false);
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: inbox_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(inbox))),
                },
            },
        },
    );
    for (address, across, home_current, needs_current) in [
        (Address::AllHome, true, true, false),
        (Address::Inbox { filter: Filter::ALL }, true, false, true),
        (Address::Work { project, filter: WorkFilter::All }, false, false, false),
        (Address::Home { project }, false, true, false),
        (Address::Inbox { filter: Filter { project: Some(project), ..Filter::ALL } }, false, false, true),
    ] {
        let work_current = address == Address::Work { project, filter: WorkFilter::All };
        fixtures::root_event(&mut root, Event::Went { place: Place::Address(address) });
        for browser in [false, true] {
            let mut view = if browser { fixtures::browser_page_view(&root) } else { fixtures::page_view(&root) };
            let home = if across { Address::AllHome } else { Address::Home { project } };
            let needs =
                Address::Inbox { filter: Filter { project: if across { None } else { Some(project) }, ..Filter::ALL } };
            let rail_name = if across { b"Needs you 2".as_slice() } else { b"Needs you 1".as_slice() };
            let tab_name = if across { b"Needs 2".as_slice() } else { b"Needs 1".as_slice() };
            let links = [
                (b"temper home".as_slice(), home.clone(), false, 1),
                (b"Home".as_slice(), home, home_current, if browser { 2 } else { 1 }),
                (rail_name, needs.clone(), needs_current, 1),
                (tab_name, needs, needs_current, u32::from(browser)),
                (
                    b"Work".as_slice(),
                    Address::Work { project, filter: WorkFilter::All },
                    work_current,
                    if browser { 2 } else { 1 },
                ),
                (b"How work runs".as_slice(), Address::Settings { project }, false, 1),
            ];
            for (name, target, current, expected) in links {
                let mut found = List::with_capacity(2);
                for node in view.tree().nodes() {
                    if node.role == Role::Link && node.name.as_deref() == Some(name) {
                        found.push((node.id, node.states.as_slice().contains(&skein_ui::State::Current))).unwrap();
                    }
                }
                assert_eq!(found.len(), expected, "{name:?}");
                for (id, is_current) in &found {
                    assert_eq!(*is_current, current, "{name:?}");
                    assert_eq!(
                        crate::act(&mut view, *id, Act::Press),
                        Some(Action::Pages(pages::Act::Go { address: target.clone() })),
                        "{name:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_switcher_puts_the_open_project_first_and_binds_the_other_project() {
    let root = fixtures::root_in_project(None);
    let mut view = fixtures::page_view(&root);
    let button = fixtures::find(view.tree(), Role::Button, b"Second").expect("other project");
    let action = crate::act(&mut view, button, Act::Press).expect("project binding");
    assert_eq!(action, Action::Pages(pages::Act::Project { project: fixtures::SECOND_PROJECT }));
    let mut root = root;
    fixtures::root_event(&mut root, temper_client_domain::Event::Act { action });
    fixtures::build(&mut view, &root);
    assert_switcher(view.tree(), b"All projects", &[(b"Second", false), (b"Shop", false)]);
}

fn assert_switcher(tree: &skein_ui::Tree, name: &[u8], projects: &[(&[u8], bool)]) {
    let nodes = tree.nodes();
    let mut index = None;
    for (at, node) in nodes.iter().enumerate() {
        if node.role == Role::Disclosure && node.name.as_deref() == Some(name) {
            index = Some(at);
            break;
        }
    }
    let index = index.expect("named switcher");
    let switcher = &nodes[index];
    assert_eq!(switcher.styles.as_slice(), [crate::look::PROJECTS]);
    assert!(!switcher.states.as_slice().contains(&skein_ui::State::Expanded));
    let end = index.checked_add(usize::try_from(switcher.size).unwrap()).unwrap();
    let mut at = index.checked_add(1).unwrap();
    let panel = nodes[at].styles.as_slice().contains(&crate::look::PROJECTS_MENU);
    if panel {
        at = at.checked_add(1).unwrap();
    }
    for (row_name, pressed) in
        [(b"All projects".as_slice(), name == b"All projects")].into_iter().chain(projects.iter().copied())
    {
        let row_end = if nodes[at].styles.as_slice().contains(&crate::look::PROJECTS_ROW) {
            let end = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
            at = at.checked_add(1).unwrap();
            Some(end)
        } else {
            None
        };
        let button = &nodes[at];
        assert_eq!(button.role, Role::Button);
        assert_eq!(button.name.as_deref(), Some(row_name));
        assert_eq!(button.states.as_slice().contains(&skein_ui::State::Pressed), pressed);
        assert!(button.binding.is_some());
        at = match row_end {
            Some(end) => end,
            None => at.checked_add(usize::try_from(button.size).unwrap()).unwrap(),
        };
        if nodes[at].role == Role::Button && nodes[at].name.as_deref() == Some(b"Rename") {
            assert!(pressed, "rename belongs to the open project");
            at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
        }
    }
    if panel {
        assert_eq!(nodes[at].styles.as_slice(), [crate::look::PROJECTS_ACTS]);
        at = at.checked_add(1).unwrap();
        if nodes[at].role == Role::Button {
            at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
        }
    }
    assert_eq!(nodes[at].role, Role::Form);
    assert_eq!(nodes[at].name.as_deref(), Some(b"New project".as_slice()));
    at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    if panel {
        if let Some(name) = nodes[at].name.as_deref()
            && name.starts_with(b"Team for ")
        {
            assert_eq!(nodes[at].role, Role::Link);
            assert_eq!(nodes[at].name.as_deref(), Some(b"Team for Shop".as_slice()));
            assert!(nodes[at].states.as_slice().contains(&skein_ui::State::Current));
            at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
        }
        assert_eq!(nodes[at].role, Role::Link);
        assert_eq!(
            nodes[at].name.as_deref().unwrap().strip_prefix(b"How work runs for "),
            Some(projects.first().unwrap().0)
        );
        at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    }
    assert_eq!(nodes[at].name.as_deref(), Some(b"Deployment".as_slice()));
    at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    assert_eq!(at, end, "the disclosure ends with its acts");
}

#[test]
fn the_corner_leads_with_set_up_temper_while_the_playground_runs() {
    use temper_client_domain::{Address, Event, forge};
    let mut root = fixtures::playground_root(Address::Forges, fixtures::PROJECT, true);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        fixtures::build(&mut view, &root);
        let nodes = view.tree().nodes();
        let mut corner = None;
        for (at, node) in nodes.iter().enumerate() {
            if node.styles.as_slice().contains(&crate::look::RAIL_FOOT) {
                corner = Some(at);
                break;
            }
        }
        let corner = corner.expect("banner corner");
        let mut setup = None;
        for (at, node) in nodes.iter().enumerate().skip(corner) {
            if node.styles.as_slice().contains(&crate::look::LEAVE_PLAYGROUND) {
                setup = Some(at);
                break;
            }
        }
        let at = setup.expect("playground setup in the corner");
        let link = &nodes[at];
        assert_eq!(link.role, Role::Link);
        assert_eq!(link.name.as_deref(), Some(b"Set up temper".as_slice()));
        assert_eq!(link.styles.as_slice(), [crate::look::LEAVE_PLAYGROUND]);
        let text = &nodes[at.checked_add(1).unwrap()];
        assert_eq!(text.role, Role::Text);
        assert_eq!(text.text.get(0).unwrap().text.as_ref(), b"Set up temper");
        let region = at.checked_add(usize::try_from(link.size).unwrap()).unwrap();
        assert_eq!(nodes[region].region, Some(crate::frame::link_region()));
        let id = link.id;
        assert_eq!(crate::act(&mut view, id, Act::Press), Some(Action::Forge(forge::Act::EndPlayground)));
    }
    fixtures::root_event(&mut root, Event::Act { action: Action::Forge(forge::Act::EndPlayground) });
    assert!(root.forge().leaving());
    assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Link, b"Set up temper").is_none());
}

#[test]
fn the_corner_has_no_set_up_link_outside_the_playground_or_on_welcome() {
    use temper_client_domain::{Address, Event, Step};
    let mut root = fixtures::playground_root(Address::Forges, fixtures::PROJECT, true);
    fixtures::root_event(
        &mut root,
        Event::Went { place: temper_client_domain::Place::Address(Address::Welcome { step: Step::Welcome }) },
    );
    for root in fixtures::pair(root, fixtures::root_in_project(None)) {
        let view = fixtures::page_view(&root);
        let nodes = view.tree().nodes();
        let mut banner = None;
        for (at, node) in nodes.iter().enumerate() {
            if node.role == Role::Banner || node.styles.as_slice().contains(&crate::look::RAIL) {
                banner = Some(at);
                break;
            }
        }
        let banner = banner.expect("banner");
        let end = banner.checked_add(usize::try_from(nodes[banner].size).unwrap()).unwrap();
        for node in &nodes[banner..end] {
            assert!(node.role != Role::Link || node.name.as_deref() != Some(b"Set up temper"));
        }
    }
}

#[test]
fn projects_me_leaves_unnamed_take_their_numbers_in_the_rail() {
    let (mut root, watch) = fixtures::root_with_watch(Some(fixtures::PROJECT));
    let mut me = root.pages().me(root.client()).expect("me").clone();
    for index in 0..me.projects.len() {
        let row = me.projects.get_mut(index).expect("project");
        row.name = jig_client::contract::DocumentText::complete(
            jig_client::contract::Text::new(
                fixtures::words()
                    .say(crate::words::Word::Project, &[skein_ui::phrases::Hole::Number(u64::from(row.project))]),
                64,
            )
            .expect("project name"),
        );
    }
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        me,
                    ))),
                },
            },
        },
    );
    let view = fixtures::page_view(&root);
    assert_rail(&view, b"Project 1");
    for name in [b"Project 1", b"Project 2"] {
        assert!(fixtures::person_find(view.tree(), Role::Button, name).is_some(), "a person finds {name:?}");
    }
}

#[test]
fn the_offline_rail_keeps_its_choices() {
    let (mut root, watch) = fixtures::root_with_watch(None);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
        },
    );
    assert_rail(&fixtures::page_view(&root), b"Shop");
}

#[test]
fn the_qa_controls_keep_their_accessible_names_with_the_menu_closed() {
    for (root, controls) in [
        (
            Box::new(fixtures::playground_root(
                temper_client_domain::Address::NewChat { draft: false },
                fixtures::PROJECT,
                true,
            )),
            &[(Role::Link, b"Chats".as_slice()), (Role::TextArea, b"What do you want done?"), (Role::Button, b"Send")]
                [..],
        ),
        (
            Box::new(fixtures::chat::messaging_root()),
            &[
                (Role::Link, b"Chats".as_slice()),
                (Role::TextArea, b"Message"),
                (Role::Button, b"Send"),
                (Role::Log, b"Conversation"),
            ][..],
        ),
    ] {
        let view = fixtures::page_view(&root);
        for (role, name) in controls {
            assert!(fixtures::person_find(view.tree(), *role, name).is_some(), "{role:?}: {name:?}");
        }
    }
}

// A second source lets the deciding-source test keep a later needing source.
const NAVIGATION_SOURCES: u32 = 2;
const NAVIGATION_PROVIDER: u32 = 18;
const NAVIGATION_PARTY: u64 = 73; // Pat in fixture_me.

fn navigation_accounts(states: &[jig_client::contract::AccountState]) -> jig_client::contract::Accounts {
    use jig_client::contract as c;
    let mut accounts = skein_lib::List::with_capacity(u32::try_from(states.len()).unwrap());
    for (index, state) in states.iter().enumerate() {
        let number = u32::try_from(index.checked_add(1).unwrap()).unwrap();
        let mut sources = skein_lib::List::with_capacity(NAVIGATION_SOURCES);
        sources
            .push(c::AccountSource {
                name: None,
                place: 1,
                number,
                kind: c::AccountKind::Key,
                state: *state,
                usable: *state == c::AccountState::Granted,
                expiring: false,
                attention: false,
                spent_until: None,
            })
            .unwrap();
        accounts
            .push(c::AccountLine {
                list: None,
                facts: skein_lib::List::with_capacity(0),
                number,
                place: number,
                uses: None,
                plan: false,
                provider: Some(NAVIGATION_PROVIDER),
                provider_name: None,
                manage_usage: None,
                display: c::DocumentText::complete(c::Text::new(Box::from(b"Pat".as_slice()), 32).unwrap()),
                holder: c::AccountHolder::Party(NAVIGATION_PARTY),
                sources,
                cut: false,
            })
            .unwrap();
    }
    c::Accounts {
        program_hosts: skein_lib::List::with_capacity(0),
        readiness: skein_lib::List::with_capacity(0),
        readiness_cut: false,
        accounts,
        signing: skein_lib::List::with_capacity(0),
        last_end: None,
        usable: false,
        settled: true,
        cut: false,
    }
}

fn deliver_navigation_accounts(
    root: &mut temper_client_domain::Domain,
    watch: skein_lib::Token,
    doc: jig_client::contract::Accounts,
) {
    fixtures::root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Accounts(
                        Box::new(doc),
                    )),
                },
            },
        },
    );
}

fn assert_account_mark(root: &temper_client_domain::Domain, expected: Option<(crate::accounts::words::Mark, &[u8])>) {
    let view = fixtures::page_view(root);
    let tree = view.tree();
    let mut in_foot = false;
    let mut link = None;
    for node in tree.nodes() {
        in_foot |= node.styles.as_slice().contains(&crate::look::RAIL_FOOT);
        if in_foot && node.role == Role::Link && node.name.as_deref() == Some(b"Deployment") {
            link = Some(node.id);
            break;
        }
    }
    let link = link.expect("Deployment in the rail's foot");
    let nodes = tree.nodes();
    let mut at = 0;
    for (index, node) in nodes.iter().enumerate() {
        if node.id == link {
            at = index;
        }
    }
    let end = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    let children = &nodes[at.checked_add(1).unwrap()..end];
    assert_eq!(children[0].text.get(0).unwrap().text.as_ref(), b"Deployment");
    let mut marks = 0_u32;
    let mut found = None;
    for node in children {
        for style in node.styles.as_slice() {
            if *style == crate::look::NAV_NEEDS || *style == crate::look::NAV_LIMITED {
                marks = marks.checked_add(1).unwrap();
                found = Some(node);
            }
        }
    }
    match expected {
        None => assert_eq!(marks, 0),
        Some((mark, words)) => {
            assert_eq!(marks, 1, "one navigation dot");
            let node = found.unwrap();
            assert_eq!(fixtures::person_find(tree, Role::Text, words), Some(node.id));
            assert_eq!(node.text.get(0).unwrap().text.as_ref(), b" \xc2\xb7 ");
            assert_eq!(node.text.get(1).unwrap().text.as_ref(), words);
            assert_eq!(
                node.styles.as_slice(),
                [match mark {
                    crate::accounts::words::Mark::Needs => crate::look::NAV_NEEDS,
                    crate::accounts::words::Mark::Limited => crate::look::NAV_LIMITED,
                }]
            );
            assert_eq!(children.len(), 2, "the state has its own text node after the name");
        }
    }
}

#[test]
fn accounts_navigation_has_no_dot_without_accounts_or_with_usable_accounts() {
    use jig_client::contract::AccountState as S;
    let (mut root, watch) = fixtures::account_root();
    assert_account_mark(&root, None);
    for states in [&[][..], &[S::Granted, S::Granted][..]] {
        deliver_navigation_accounts(&mut root, watch, navigation_accounts(states));
        assert_account_mark(&root, None);
    }
}

#[test]
fn a_moving_account_has_no_navigation_mark() {
    let (mut root, watch) = fixtures::account_root();
    deliver_navigation_accounts(&mut root, watch, navigation_accounts(&[jig_client::contract::AccountState::Moving]));
    assert_eq!(crate::accounts::words::mark(&root), None);
    assert_account_mark(&root, None);
}

#[test]
fn accounts_navigation_is_red_for_each_state_needing_the_person() {
    use c::AccountState as S;
    use jig_client::contract as c;
    let (mut root, watch) = fixtures::account_root();
    for state in [
        S::Unloaded { at: c::AccountPlace::File, why: c::AccountUnread::Refused },
        S::SignInAgain,
        S::Misconfigured,
        S::Unloaded { at: c::AccountPlace::Directory, why: c::AccountUnread::Unreadable },
    ] {
        deliver_navigation_accounts(&mut root, watch, navigation_accounts(&[state]));
        assert_account_mark(&root, Some((crate::accounts::words::Mark::Needs, b"an account needs you")));
    }
}

#[test]
fn accounts_navigation_is_amber_for_limited_and_red_takes_precedence() {
    use crate::accounts::words::Mark;
    use jig_client::contract::AccountState as S;
    let (mut root, watch) = fixtures::account_root();
    for (states, mark, words) in [
        (&[S::Later { until: None }][..], Mark::Limited, b"an account is limited".as_slice()),
        (&[S::Later { until: None }, S::SignInAgain][..], Mark::Needs, b"an account needs you".as_slice()),
        (&[S::SignInAgain, S::Later { until: None }][..], Mark::Needs, b"an account needs you".as_slice()),
    ] {
        deliver_navigation_accounts(&mut root, watch, navigation_accounts(states));
        assert_account_mark(&root, Some((mark, words)));
    }
}

#[test]
fn accounts_navigation_uses_only_the_first_supplied_source_even_when_cut() {
    use jig_client::contract::AccountState as S;
    let (mut root, watch) = fixtures::account_root();
    let mut doc = navigation_accounts(&[S::Granted, S::SignInAgain]);
    let later = doc.accounts.get(1).unwrap().sources.get(0).unwrap().clone();
    doc.accounts.truncate(1);
    let row = doc.accounts.get_mut(0).unwrap();
    row.sources.push(later).unwrap();
    row.cut = true;
    doc.cut = true;
    deliver_navigation_accounts(&mut root, watch, doc);
    assert_account_mark(&root, None);
    let mut doc = navigation_accounts(&[S::SignInAgain]);
    doc.cut = true;
    doc.accounts.get_mut(0).unwrap().cut = true;
    deliver_navigation_accounts(&mut root, watch, doc);
    assert_account_mark(&root, Some((crate::accounts::words::Mark::Needs, b"an account needs you")));
}

#[test]
fn accounts_navigation_ignores_the_playground_provider_and_other_holders() {
    use jig_client::contract as c;
    let (mut root, _, _, _, watch) = fixtures::root_at_with_playground(
        temper_client_domain::Address::Accounts,
        &fixtures::domain_limits(),
        b"Shop",
        Some((fixtures::PROJECT, true)),
    );
    let provider = root.forge().playground().unwrap().provider;
    let mut doc = navigation_accounts(&[c::AccountState::SignInAgain; 3]);
    doc.accounts.get_mut(0).unwrap().provider = Some(provider);
    doc.accounts.get_mut(1).unwrap().holder = c::AccountHolder::Deployment;
    doc.accounts.get_mut(2).unwrap().holder = c::AccountHolder::Party(fixtures::OTHER_PARTY);
    for cut in [false, true] {
        doc.cut = cut;
        deliver_navigation_accounts(&mut root, watch.unwrap(), doc.clone());
        assert_account_mark(&root, None);
    }
}

#[test]
fn accounts_navigation_reads_attention_as_needing_and_spent_usage_as_limited() {
    use crate::accounts::words::Mark;
    use jig_client::contract::AccountState as S;
    let (mut root, watch) = fixtures::account_root();
    let mut attention = navigation_accounts(&[S::Granted]);
    attention.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().attention = true;
    deliver_navigation_accounts(&mut root, watch, attention);
    assert_account_mark(&root, Some((Mark::Needs, b"an account needs you")));
    let mut spent = navigation_accounts(&[S::Granted]);
    spent.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().spent_until =
        Some(skein_lib::Wall::from_nanos(60_000_000_000));
    deliver_navigation_accounts(&mut root, watch, spent);
    assert_account_mark(&root, Some((Mark::Limited, b"an account is limited")));
}

#[test]
fn known_sign_in_ways_ignore_unknown_words_and_duplicates() {
    for (content, google) in
        [("", false), ("unknown", false), ("google", true), ("future google google", true), ("Google", false)]
    {
        assert_eq!(
            crate::sign_in::Ways::parse(content, b"https://temper.example.com"),
            crate::sign_in::Ways { google, origin: Some(Box::from(b"https://temper.example.com".as_slice())) },
            "{content}"
        );
    }
    for origin in [b"".as_slice(), b"null", b"file://", b"chrome-extension://x"] {
        assert_eq!(crate::sign_in::Ways::parse("google", origin), crate::sign_in::Ways::NONE);
    }
}
#[test]
fn browser_google_sign_in_has_the_mark_heading_and_return_link() {
    use crate::{sign_in::Ways, words::Shell};
    use temper_client_domain::{Address, Event};
    for (address, expected) in [
        // Signed out, the page stays at its address; its query is percent-encoded into `then`.
        (
            Address::Chats { filter: ChatsFilter::All },
            b"https://temper.example.com/sign-in/google?then=/%3Fproject%3D1".as_slice(),
        ),
    ] {
        let mut root = fixtures::signed_out();
        fixtures::root_event(&mut root, Event::Act { action: Action::Pages(pages::Act::Go { address }) });
        let mut view = fixtures::sign_in_view(
            &root,
            Shell::Browser,
            Ways { google: true, origin: Some(Box::from(b"https://temper.example.com".as_slice())) },
        );
        assert!(fixtures::find(view.tree(), Role::Heading { level: 1 }, b"Sign in").is_some());
        assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([crate::look::OUTSIDE])]);
        assert_eq!(fixtures::styles(view.tree(), Role::Paragraph).as_ref(), [Box::from([crate::look::OUTSIDE_QUIET])]);
        for styles in &fixtures::styles(view.tree(), Role::Slot) {
            assert!(!styles.contains(&crate::look::CARD));
        }
        let link = fixtures::find(view.tree(), Role::Link, b"Continue with Google").unwrap();
        assert_eq!(view.tree().find(link).unwrap().styles.as_slice(), [crate::look::OUTSIDE_PRIMARY]);
        assert_eq!(crate::href(&mut view, link).as_deref(), Some(expected));
        let public = fixtures::find(view.tree(), Role::Link, b"Read public projects").unwrap();
        assert_eq!(crate::href(&mut view, public).as_deref(), Some(b"https://temper.example.com/public".as_slice()));
    }
}
#[test]
fn terminal_with_google_keeps_its_signed_out_main() {
    let view = fixtures::sign_in_view(
        &fixtures::signed_out(),
        crate::words::Shell::Terminal,
        crate::sign_in::Ways { google: true, origin: Some(Box::from(b"https://temper.example.com".as_slice())) },
    );
    let nodes = view.tree().nodes();
    let mut main = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Main {
            let start = index.checked_add(1).expect("heading follows main");
            let end = index.checked_add(usize::try_from(node.size).expect("main size")).expect("main end");
            main = Some((start, end));
            break;
        }
    }
    let (start, end) = main.expect("main");
    let mut content = skein_lib::List::with_capacity(2);
    for (index, node) in nodes.iter().enumerate().take(end).skip(start) {
        if node.role != Role::Text {
            content
                .push((node.role, node.name.as_deref(), fixtures::node_text(view.tree(), index)))
                .expect("heading and paragraph");
        }
    }
    assert_eq!(
        content.as_slice(),
        [(Role::Heading { level: 1 }, Some(b"Signed out".as_slice()), Box::from(b"Signed out".as_slice()))]
    );
}

#[test]
fn signing_out_explains_each_forges_independent_sign_in() {
    let mut root = fixtures::signed_out();
    fixtures::root_event(&mut root, temper_client_domain::Event::Act { action: Action::Pages(pages::Act::SignOut) });
    let domain = fixtures::domain_limits();
    let limits = crate::Limits { tree: fixtures::tree_limits() };
    let mut view = crate::View::with_sign_in(
        &limits,
        &domain,
        fixtures::browser_words(),
        fixtures::FORGE,
        crate::sign_in::Ways::parse("google", b"https://temper.example"),
    );
    let mut facts = crate::forge_sign_in::Forges::new();
    facts.read("1 forgejo https://forge.example other");
    facts.read("2 owned http://127.0.0.1:3000 owned");
    view.forge_sign_in(facts);
    fixtures::build(&mut view, &root);
    let text = fixtures::main_text(view.tree());
    assert!(
        skein_lib::bytes::find(
            &text,
            b"You signed out. Forgejo at forge.example keeps its own sign-in: sign out there too."
        )
        .is_some()
    );
    assert!(
        skein_lib::bytes::find(
            &text,
            b"You signed out. Forgejo on this machine keeps its own sign-in: sign out there too."
        )
        .is_some()
    );
}

#[test]
fn the_brand_links_to_inbox_with_its_accessible_label() {
    let root = fixtures::no_project_root();
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        fixtures::build(&mut view, &root);
        let brand = fixtures::find(view.tree(), Role::Link, b"temper").expect("brand link");
        let mut at = None;
        for (index, node) in view.tree().nodes().iter().enumerate() {
            if node.id == brand {
                at = Some(index);
                break;
            }
        }
        let at = at.expect("brand position");
        assert_eq!(fixtures::node_text(view.tree(), at).as_ref(), b"temper");
        assert!(view.tree().nodes()[at].styles.as_slice().contains(&crate::look::BRAND));
        assert_eq!(crate::href(&mut view, brand).as_deref(), Some(b"/".as_slice()));
        assert_eq!(
            crate::act(&mut view, brand, Act::Press),
            Some(Action::Pages(pages::Act::Go {
                address: temper_client_domain::Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
            }))
        );
    }
}

#[test]
fn the_banner_is_one_line_with_its_set_up_link() {
    use super::fixtures as f;
    use temper_client_domain::{Address, Step};
    for address in [
        Address::Accounts,
        Address::Forges,
        Address::Welcome { step: Step::Welcome },
        Address::Chats { filter: ChatsFilter::All },
    ] {
        let chats = address == Address::Chats { filter: ChatsFilter::All };
        let mut root = f::playground_root(Address::Chats { filter: ChatsFilter::All }, f::PROJECT, true);
        f::root_event(
            &mut root,
            temper_client_domain::Event::Went { place: temper_client_domain::Place::Address(address) },
        );
        let view = f::page_view(&root);
        let mut region = None;
        for (index, node) in view.tree().nodes().iter().enumerate() {
            if node.role == Role::Region && node.name.as_deref() == Some(b"Playground") {
                region = Some(index);
            }
        }
        // While it runs the line ends the chats pane alone (client/view.md, 1).
        if !chats {
            assert!(region.is_none());
            continue;
        }
        let region = region.expect("playground banner");
        let end = region.checked_add(usize::try_from(view.tree().nodes().get(region).unwrap().size).unwrap()).unwrap();
        let mut paragraphs = 0_u32;
        let mut links = 0_u32;
        for node in view.tree().nodes().get(region..end).unwrap() {
            if node.role == Role::Paragraph {
                paragraphs = paragraphs.checked_add(1).unwrap();
            }
            if node.role == Role::Link {
                links = links.checked_add(1).unwrap();
            }
        }
        assert_eq!(paragraphs, 1);
        assert_eq!(links, 1);
        let text = f::node_text(view.tree(), region);
        assert_eq!(
            text.as_ref(),
            b"This is the playground: a scripted model and in-memory forge. Use the wizard to set up temper"
        );
    }
}
#[test]
fn the_link_sends_end_playground_and_shows_leaving() {
    use super::fixtures as f;
    use temper_client_domain::{Address, Event, forge};
    let mut root = f::playground_root(Address::Chats { filter: ChatsFilter::All }, f::PROJECT, true);
    let mut view = f::page_view(&root);
    let link = f::find(view.tree(), Role::Link, b"set up temper").unwrap();
    let action = crate::act(&mut view, link, Act::Press).unwrap();
    assert_eq!(action, Action::Forge(forge::Act::EndPlayground));
    f::root_event(&mut root, Event::Act { action });
    let view = f::page_view(&root);
    assert!(root.forge().leaving());
    assert!(skein_lib::bytes::find(&f::text(view.tree()), "Leaving the playground…".as_bytes()).is_some());
    assert!(f::find(view.tree(), Role::Link, b"set up temper").is_none());
    // While it is leaving, the line shows on every page (client/view.md, 1).
    f::root_event(&mut root, Event::Went { place: temper_client_domain::Place::Address(Address::Forges) });
    let view = f::page_view(&root);
    assert!(skein_lib::bytes::find(&f::text(view.tree()), "Leaving the playground…".as_bytes()).is_some());
}

fn switcher_act(root: &mut temper_client_domain::Domain, act: pages::switcher::Act) {
    fixtures::root_event(root, temper_client_domain::Event::Act { action: Action::Pages(pages::Act::Switcher(act)) });
}

fn switcher_owner(start: bool) -> temper_client_domain::Domain {
    use jig_client::contract as c;
    let (mut root, watch, _) = fixtures::root_at(
        if start {
            temper_client_domain::Address::NewChat { draft: false }
        } else {
            temper_client_domain::Address::Chats { filter: ChatsFilter::All }
        },
        &fixtures::domain_limits(),
    );
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().role = c::HeldRole::Owner;
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        },
    );
    root
}

fn switcher_node<'a>(view: &'a crate::View, role: Role, name: &[u8]) -> &'a skein_ui::Node {
    let id = fixtures::person_find(view.tree(), role, name).expect("switcher control");
    for node in view.tree().nodes() {
        if node.id == id {
            return node;
        }
    }
    panic!("found node belongs to tree");
}

#[test]
fn an_owner_is_offered_rename_on_the_open_project() {
    for words in [fixtures::words(), fixtures::browser_words()] {
        let root = switcher_owner(false);
        let mut view = fixtures::view_of(words);
        fixtures::build(&mut view, &root);
        assert_rail(&view, b"Shop");
        let rename = switcher_node(&view, Role::Button, b"Rename Shop").id;
        assert_eq!(
            crate::act(&mut view, rename, Act::Press),
            Some(Action::Pages(pages::Act::Switcher(pages::switcher::Act::Rename)))
        );
    }
}

#[test]
fn a_member_is_offered_no_rename() {
    let mut root = switcher_owner(false);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: Action::Pages(pages::Act::Project { project: fixtures::SECOND_PROJECT }),
        },
    );
    let view = fixtures::page_view(&root);
    assert!(fixtures::person_find(view.tree(), Role::Button, b"Rename Second").is_none());
    assert!(fixtures::person_find(view.tree(), Role::Form, b"New project").is_some());
}

#[test]
fn the_rename_form_binds_its_field_save_and_cancel() {
    use pages::switcher::Act as S;
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut root = switcher_owner(false);
        let mut view = fixtures::view_of(words);
        fixtures::build(&mut view, &root);
        let rename = switcher_node(&view, Role::Button, b"Rename Shop").id;
        let action = crate::act(&mut view, rename, Act::Press).unwrap();
        fixtures::root_event(&mut root, temper_client_domain::Event::Act { action });
        fixtures::build(&mut view, &root);
        assert_rail(&view, b"Shop");
        assert_eq!(skein_ui::outline::check(view.tree()), None);
        let field = switcher_node(&view, Role::TextField, b"Rename");
        assert_eq!(field.value.as_ref().unwrap().text.as_ref(), b"Shop");
        assert_eq!(field.focus, Some(0));
        let field = field.id;
        let action = crate::act(&mut view, field, Act::Input { value: b"New shop".as_slice().into() }).unwrap();
        assert_eq!(
            action,
            Action::Pages(pages::Act::Switcher(S::RenameWords { words: b"New shop".as_slice().into() }))
        );
        fixtures::root_event(&mut root, temper_client_domain::Event::Act { action });
        let mut patches = skein_lib::Queue::uncapped();
        crate::build(&mut view, &root, fixtures::NOW, &mut patches);
        assert_eq!(switcher_node(&view, Role::TextField, b"Rename").id, field);
        assert_eq!(switcher_node(&view, Role::TextField, b"Rename").value.as_ref().unwrap().text.as_ref(), b"New shop");
        for patch in &patches {
            if let skein_ui::Patch::Attribute { node, attribute } = patch
                && *node == field
            {
                assert_eq!(attribute.focus, Some(0), "editing does not advance the focus request");
            }
        }
        let save = switcher_node(&view, Role::Button, b"Save").id;
        let cancel = switcher_node(&view, Role::Button, b"Cancel");
        assert_eq!(cancel.shortcut, Some(skein_ui::Shortcut::ESCAPE));
        let cancel = cancel.id;
        for (node, act, expected) in
            [(field, Act::Submit, S::Save), (save, Act::Press, S::Save), (cancel, Act::Press, S::Cancel)]
        {
            assert_eq!(crate::act(&mut view, node, act), Some(Action::Pages(pages::Act::Switcher(expected))));
        }
        let action = crate::act(&mut view, cancel, Act::Press).unwrap();
        fixtures::root_event(&mut root, temper_client_domain::Event::Act { action });
        fixtures::build(&mut view, &root);
        assert!(fixtures::person_find(view.tree(), Role::Form, b"Rename Shop").is_none());
        assert!(fixtures::person_find(view.tree(), Role::Button, b"Rename Shop").is_some());
    }
}

#[test]
fn the_new_project_form_ends_the_list() {
    use pages::switcher::Act as S;
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut root = fixtures::root_in_project(None);
        switcher_act(&mut root, S::NewWords { words: b"Third".as_slice().into() });
        let mut view = fixtures::view_of(words);
        fixtures::build(&mut view, &root);
        assert_rail(&view, b"Shop");
        assert_eq!(skein_ui::outline::check(view.tree()), None);
        let field = switcher_node(&view, Role::TextField, b"New project");
        assert_eq!(field.placeholder.as_deref(), Some(b"+ New project".as_slice()));
        assert_eq!(field.value.as_ref().unwrap().text.as_ref(), b"Third");
        assert_eq!(field.focus, None);
        let field = field.id;
        assert_eq!(
            crate::act(&mut view, field, Act::Input { value: b"Fourth".as_slice().into() }),
            Some(Action::Pages(pages::Act::Switcher(S::NewWords { words: b"Fourth".as_slice().into() })))
        );
        let create = switcher_node(&view, Role::Button, b"Create").id;
        for (node, act) in [(field, Act::Submit), (create, Act::Press)] {
            assert_eq!(crate::act(&mut view, node, act), Some(Action::Pages(pages::Act::Switcher(S::Create))));
        }
    }
}

fn send_switcher(root: &mut temper_client_domain::Domain, act: pages::switcher::Act) -> skein_lib::Token {
    use temper_client_domain::{Event, Request};
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: fixtures::domain_limits() };
    let mut out = skein_lib::Queue::uncapped();
    temper_client_domain::step(root, &env, Event::Act { action: Action::Pages(pages::Act::Switcher(act)) }, &mut out);
    temper_client_domain::step(root, &env, Event::Client { event: jig_client::Event::Stored }, &mut out);
    let mut sent = None;
    for request in &out {
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            assert!(sent.replace(*attempt).is_none(), "one switcher request");
        }
    }
    sent.expect("switcher sends")
}

#[test]
fn a_pending_rename_disables_save() {
    use pages::switcher::Act as S;
    let mut root = switcher_owner(false);
    switcher_act(&mut root, S::Rename);
    switcher_act(&mut root, S::RenameWords { words: b"Renamed".as_slice().into() });
    send_switcher(&mut root, S::Save);
    assert_eq!(root.pages().switcher().pending(), Some(pages::switcher::Form::Rename));
    let mut view = fixtures::page_view(&root);
    for (role, name, act) in [
        (Role::Button, b"Save".as_slice(), Act::Press),
        (Role::Button, b"Cancel", Act::Press),
        (Role::TextField, b"Rename", Act::Submit),
        (Role::Button, b"Create", Act::Press),
    ] {
        let node = switcher_node(&view, role, name);
        assert!(node.states.as_slice().contains(&skein_ui::State::Disabled));
        let id = node.id;
        assert_eq!(crate::act(&mut view, id, act), None);
    }
}

#[test]
fn a_pending_new_project_disables_its_field_and_create() {
    use pages::switcher::Act as S;
    for (mut root, name) in [
        (Box::new(fixtures::root_in_project(None)), b"New project".as_slice()),
        (Box::new(fixtures::no_project_root()), b"Project name"),
    ] {
        switcher_act(&mut root, S::NewWords { words: b"Third".as_slice().into() });
        send_switcher(&mut root, S::Create);
        for words in [fixtures::words(), fixtures::browser_words()] {
            let mut view = fixtures::view_of(words);
            fixtures::build(&mut view, &root);
            for (role, name, act) in [(Role::TextField, name, Act::Submit), (Role::Button, b"Create", Act::Press)] {
                let node = switcher_node(&view, role, name);
                assert!(node.states.as_slice().contains(&skein_ui::State::Disabled));
                let id = node.id;
                assert_eq!(crate::act(&mut view, id, act), None);
            }
        }
    }
}

fn refuse_switcher(root: &mut temper_client_domain::Domain, attempt: skein_lib::Token) {
    fixtures::root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(jig_client::contract::Answer::Refused {
                        refusal: jig_client::contract::Refusal::NameTaken,
                        at: None,
                    })),
                    why: None,
                },
            },
        },
    );
}

#[test]
fn a_refused_new_project_shows_why() {
    use pages::switcher::Act as S;
    for (mut root, name) in [
        (Box::new(fixtures::root_in_project(None)), b"New project".as_slice()),
        (Box::new(fixtures::no_project_root()), b"Project name"),
    ] {
        switcher_act(&mut root, S::NewWords { words: b"Shop".as_slice().into() });
        let attempt = send_switcher(&mut root, S::Create);
        refuse_switcher(&mut root, attempt);
        for words in [fixtures::words(), fixtures::browser_words()] {
            let mut view = fixtures::view_of(words);
            fixtures::build(&mut view, &root);
            let field = switcher_node(&view, Role::TextField, name);
            assert_eq!(field.value.as_ref().unwrap().text.as_ref(), b"Shop");
            assert_refusal_under_field(&view, b"New project", name);
            assert!(
                !switcher_node(&view, Role::Button, b"Create").states.as_slice().contains(&skein_ui::State::Disabled)
            );
            if name == b"Project name" {
                let nodes = view.tree().nodes();
                let mut alert = None;
                let mut button = None;
                for (index, node) in nodes.iter().enumerate() {
                    if node.role == Role::Alert {
                        alert = Some(index);
                    }
                    if node.role == Role::Button && node.name.as_deref() == Some(b"Create") {
                        button = Some(index);
                    }
                }
                assert!(button.expect("create") < alert.expect("column refusal alert"));
            }
        }
    }
}

#[test]
fn a_refused_rename_keeps_the_words_and_shows_why() {
    use pages::switcher::Act as S;
    let mut root = switcher_owner(false);
    switcher_act(&mut root, S::Rename);
    switcher_act(&mut root, S::RenameWords { words: b"Second".as_slice().into() });
    let attempt = send_switcher(&mut root, S::Save);
    refuse_switcher(&mut root, attempt);
    let view = fixtures::page_view(&root);
    assert_eq!(switcher_node(&view, Role::TextField, b"Rename").value.as_ref().unwrap().text.as_ref(), b"Second");
    assert_refusal_under_field(&view, b"Rename Shop", b"Rename");
    assert!(!switcher_node(&view, Role::Button, b"Save").states.as_slice().contains(&skein_ui::State::Disabled));
}

fn assert_refusal_under_field(view: &crate::View, form_name: &[u8], field_name: &[u8]) {
    let form = switcher_node(view, Role::Form, form_name).id;
    let field = switcher_node(view, Role::TextField, field_name).id;
    let why =
        fixtures::words().say_word(jig_client::words::Word::refusal(&jig_client::contract::Refusal::NameTaken), &[]);
    let nodes = view.tree().nodes();
    let mut form_at = None;
    let mut field_at = None;
    let mut refusal = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.id == form {
            form_at = Some(index);
        }
        if node.id == field {
            field_at = Some(index);
        }
        if node.role == Role::Status && node.name.as_deref() == Some(why.as_ref()) {
            refusal = Some(index);
        }
    }
    let form = form_at.expect("form");
    let field = field_at.expect("field");
    let refusal = refusal.expect("refusal words");
    let end = form.checked_add(usize::try_from(nodes[form].size).unwrap()).unwrap();
    assert!(form < field && field < refusal && refusal < end, "refusal follows the field in its form");
}

#[test]
fn opening_rename_on_the_start_address_focuses_its_field_and_cancel_restores_the_composer() {
    let mut root = switcher_owner(true);
    let mut view = fixtures::page_view(&root);
    assert_eq!(switcher_node(&view, Role::TextArea, b"What do you want done?").focus, Some(0));
    let rename = switcher_node(&view, Role::Button, b"Rename Shop").id;
    let action = crate::act(&mut view, rename, Act::Press).unwrap();
    fixtures::root_event(&mut root, temper_client_domain::Event::Act { action });
    fixtures::build(&mut view, &root);
    assert_eq!(skein_ui::outline::check(view.tree()), None);
    let field = switcher_node(&view, Role::TextField, b"Rename").id;
    for node in view.tree().nodes() {
        assert_eq!(node.focus.is_some(), node.id == field);
    }
    let cancel = switcher_node(&view, Role::Button, b"Cancel").id;
    let action = crate::act(&mut view, cancel, Act::Press).unwrap();
    fixtures::root_event(&mut root, temper_client_domain::Event::Act { action });
    fixtures::build(&mut view, &root);
    assert_eq!(switcher_node(&view, Role::TextArea, b"What do you want done?").focus, Some(0));
    assert!(fixtures::person_find(view.tree(), Role::TextField, b"Rename").is_none());
}

fn assert_rail(view: &crate::View, project: &[u8]) {
    let nodes = view.tree().nodes();
    let mut rail = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.styles.as_slice().contains(&crate::look::RAIL) {
            rail = Some(index);
        }
    }
    let rail = rail.expect("rail");
    assert_eq!(nodes[rail].role, Role::Navigation);
    assert_eq!(nodes[rail].name.as_deref(), Some(project));
    let mut at = rail.checked_add(1).unwrap();
    for (role, style) in [
        (Role::Link, crate::look::BRAND),
        (Role::Disclosure, crate::look::PROJECTS),
        (Role::Slot, crate::look::RAIL_NAV),
        (Role::Slot, crate::look::RAIL_NEW),
        (Role::Slot, crate::look::RAIL_FOOT),
    ] {
        assert_eq!(nodes[at].role, role);
        assert_eq!(nodes[at].styles.as_slice(), [style]);
        if style == crate::look::PROJECTS {
            assert!(nodes[at].name.as_deref() == Some(project) || nodes[at].name.as_deref() == Some(b"All projects"));
        }
        at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
        if style == crate::look::PROJECTS && nodes[at].styles.as_slice().contains(&crate::look::AWAY_CHIP) {
            assert_eq!(nodes[at].role, Role::Button);
            at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
        }
    }
    if let Some(node) = nodes.get(at)
        && node.styles.as_slice().contains(&crate::look::NIGHT_SWITCH)
    {
        assert_eq!(node.role, Role::Button);
        at = at.checked_add(usize::try_from(node.size).unwrap()).unwrap();
    }
    assert_eq!(at, rail.checked_add(usize::try_from(nodes[rail].size).unwrap()).unwrap());
}

#[test]
#[expect(clippy::too_many_lines, reason = "one project switch checks its frame and Work's reads in both shells")]
fn goals_opens_in_its_projects_frame() {
    use temper_client_domain::{Address, Event, GoalsFilter, Place, Request};
    // Work's changes and notes, the rail's people and the project's ways and classes open without a goal template.
    const PROJECT_READS: u32 = 5;
    for filter in [GoalsFilter::All, GoalsFilter::Needs, GoalsFilter::Live, GoalsFilter::Ended] {
        for words in [fixtures::words(), fixtures::browser_words()] {
            let address = Address::Goals { project: fixtures::SECOND_PROJECT, filter };
            let (mut root, _, page) = fixtures::root_at(address.clone(), &fixtures::domain_limits());
            assert!(page.is_none(), "goals starts no page watch");
            assert_eq!(root.pages().project(), Some(fixtures::SECOND_PROJECT));
            assert_eq!(root.pages().address(), &address);
            let project_name = b"Shop";
            let mut view = fixtures::view_of(words);
            fixtures::build(&mut view, &root);
            assert_rail(&view, b"Second");
            assert!(fixtures::find(view.tree(), Role::Paragraph, "Second · Work".as_bytes()).is_some());
            assert!(fixtures::find(view.tree(), Role::Navigation, b"Work's filters").is_some());
            assert!(
                skein_lib::bytes::find(&fixtures::main_text(view.tree()), b"A goal turns a draft into landed work.")
                    .is_some()
            );
            assert!(fixtures::styles(view.tree(), Role::Heading { level: 1 }).is_empty());
            assert!(fixtures::find(view.tree(), Role::Link, b"Work").is_some(), "goals page keeps Work in the rail");
            assert_eq!(skein_ui::outline::check(view.tree()), None);
            for node in view.tree().nodes() {
                if node.role == Role::Main {
                    assert_eq!(node.styles.as_slice(), [crate::look::MAIN, crate::look::GOALS]);
                }
            }
            let project = fixtures::person_find(view.tree(), Role::Button, project_name).expect("other project");
            let action = crate::act(&mut view, project, Act::Press).expect("choose project");
            assert_eq!(action, Action::Pages(pages::Act::Project { project: fixtures::PROJECT }));
            let mut out = skein_lib::Queue::uncapped();
            let env =
                skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: fixtures::domain_limits() };
            temper_client_domain::step(&mut root, &env, Event::Act { action }, &mut out);
            let address = Address::Goals { project: fixtures::PROJECT, filter: GoalsFilter::All };
            let mut shown = 0_u32;
            for request in &out {
                if let Request::Show { address: next, replace } = request {
                    assert_eq!(next, &address);
                    assert!(!replace, "project choice is a move");
                    shown += 1;
                }
            }
            assert_eq!(shown, 1, "switch keeps Goals with filter All");
            out.clear();
            temper_client_domain::step(
                &mut root,
                &env,
                Event::Went { place: Place::Address(address.clone()) },
                &mut out,
            );
            assert_eq!(
                out.len(),
                PROJECT_READS,
                "changes, people, notes and the new project's ways and classes: {out:?}"
            );
            let Some(Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { subject, .. } },
            }) = out.pop()
            else {
                panic!("changes watch")
            };
            assert_eq!(
                Some(subject),
                temper_client_domain::forge::changes::watch(fixtures::PROJECT, &env.limits.forge)
            );
            let Some(Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Read {
                                what: jig_client::Readable::Jig(jig_client::contract::Read::Project { project }),
                                ..
                            },
                    },
            }) = out.pop()
            else {
                panic!("rail's project people")
            };
            assert_eq!(project, fixtures::PROJECT);
            let Some(Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Read {
                                what:
                                    jig_client::Readable::Jig(jig_client::contract::Read::Notes {
                                        scope,
                                        description,
                                        after,
                                    }),
                                ..
                            },
                    },
            }) = out.pop()
            else {
                panic!("chat notes")
            };
            assert_eq!(scope, jig_client::contract::NoteScope::Tasks { project: fixtures::PROJECT });
            assert_eq!((description, after), (None, None));
            let mut kinds = skein_lib::List::with_capacity(2);
            while let Some(request) = out.pop() {
                let Request::Unlinked {
                    request:
                        jig_client::UnlinkedRequest::Down {
                            down: jig_client::Down::Read { what: jig_client::Readable::Family { record, .. }, .. },
                        },
                } = request
                else {
                    panic!("project settings read")
                };
                let reading = temper_client_protocol_wire::decode_reading(
                    &record.bytes,
                    &temper_client_protocol_wire::v1::CEILINGS,
                )
                .unwrap();
                let temper_client_protocol_wire::v1::Read::Setting(setting) = reading.read() else {
                    panic!("project setting")
                };
                assert_eq!((setting.scope(), setting.number()), (fixtures::PROJECT, 1));
                kinds.push(setting.kind()).unwrap();
            }
            assert!(kinds.as_slice().contains(&7));
            assert!(kinds.as_slice().contains(&8));
            assert_eq!(root.pages().address(), &address);
            assert_eq!(root.pages().project(), Some(fixtures::PROJECT));
            fixtures::build(&mut view, &root);
            assert_rail(&view, b"Shop");
            assert!(fixtures::find(view.tree(), Role::Paragraph, "Shop · Work".as_bytes()).is_some());
            assert!(
                skein_lib::bytes::find(&fixtures::main_text(view.tree()), b"A goal turns a draft into landed work.")
                    .is_some()
            );
        }
    }
}

#[test]
fn renaming_replaces_the_open_project_row_in_the_rail() {
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut root = switcher_owner(false);
        fixtures::root_event(
            &mut root,
            temper_client_domain::Event::Act {
                action: Action::Pages(pages::Act::Switcher(pages::switcher::Act::Rename)),
            },
        );
        let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
        let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
        crate::frame::switcher(
            &mut builder,
            &root,
            &words,
            &mut bindings,
            fixtures::PROJECT,
            &crate::forge_sign_in::Forges::new(),
            pages::rail::parts(&pages::rail::facts(&root, fixtures::PROJECT, fixtures::NOW)),
        );
        let tree = builder.finish();
        assert!(fixtures::find(&tree, Role::Button, b"Shop").is_none());
        assert!(fixtures::find(&tree, Role::Form, b"Rename Shop").is_some());
    }
}

#[test]
fn opening_rename_takes_focus_from_a_chat_composer() {
    use temper_client_domain::{Address, Event};
    let source = fixtures::chat::messaging_root();
    let task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    let (mut root, me, watch) =
        fixtures::root_at(Address::Chat { task: fixtures::chat::TASK }, &fixtures::domain_limits());
    let mut person = root.pages().me(root.client()).unwrap().clone();
    person.projects.get_mut(0).unwrap().role = jig_client::contract::HeldRole::Owner;
    fixtures::deliver_me(&mut root, me, person);
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Task(Box::new(
                        task,
                    ))),
                },
            },
        },
    );
    fixtures::root_event(
        &mut root,
        Event::Act { action: Action::Pages(pages::Act::ChangeProposal { proposal: fixtures::chat::PROPOSAL }) },
    );
    for renaming in [false, true, false] {
        let act = if renaming { pages::switcher::Act::Rename } else { pages::switcher::Act::Cancel };
        fixtures::root_event(&mut root, Event::Act { action: Action::Pages(pages::Act::Switcher(act)) });
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let composer = switcher_node(&view, Role::TextArea, b"Message");
            assert_eq!(composer.focus, if renaming { None } else { Some(1) });
            if renaming {
                assert_eq!(switcher_node(&view, Role::TextField, b"Rename").focus, Some(0));
            }
        }
    }
}

#[test]
fn opening_rename_takes_focus_from_a_task_composer() {
    use temper_client_domain::{Address, Event};
    for priority in [None, Some(1)] {
        let mut task = fixtures::task::delegate();
        task.priority = priority;
        task.acts = skein_lib::List::uncapped();
        task.acts
            .push(jig_client::contract::Offered {
                act: jig_client::contract::ActKind::Message,
                admission: jig_client::contract::Admission::Allowed,
                effect: None,
            })
            .unwrap();
        let (mut root, me, watch) =
            fixtures::root_at(Address::Task { task: task.chip.task }, &fixtures::domain_limits());
        let mut person = root.pages().me(root.client()).unwrap().clone();
        person.projects.get_mut(0).unwrap().role = jig_client::contract::HeldRole::Owner;
        fixtures::deliver_me(&mut root, me, person);
        fixtures::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: watch.unwrap(),
                        event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Task(
                            Box::new(task),
                        )),
                    },
                },
            },
        );
        fixtures::root_event(&mut root, Event::Act { action: Action::Pages(pages::Act::Send) });
        for renaming in [false, true, false] {
            let act = if renaming { pages::switcher::Act::Rename } else { pages::switcher::Act::Cancel };
            fixtures::root_event(&mut root, Event::Act { action: Action::Pages(pages::Act::Switcher(act)) });
            for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
                let at = super::chat::named_position(view.tree().nodes(), Role::TextArea, None).expect("composer");
                let composer = &view.tree().nodes()[at];
                assert_eq!(composer.focus, if renaming { None } else { Some(1) });
                if renaming {
                    assert_eq!(switcher_node(&view, Role::TextField, b"Rename").focus, Some(0));
                }
            }
        }
    }
}

#[test]
fn opening_rename_takes_focus_from_the_limits_heading() {
    use temper_client_domain::{Address, Event, Place};
    let (mut root, watch) = fixtures::away_frame(None, None);
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().role = jig_client::contract::HeldRole::Owner;
    fixtures::deliver_me(&mut root, watch, me);
    fixtures::root_event(
        &mut root,
        Event::Act { action: Action::Pages(pages::Act::Away(pages::away::Act::Limits { project: fixtures::PROJECT })) },
    );
    fixtures::root_event(
        &mut root,
        Event::Went { place: Place::Address(Address::Settings { project: fixtures::PROJECT }) },
    );
    for renaming in [false, true, false] {
        let act = if renaming { pages::switcher::Act::Rename } else { pages::switcher::Act::Cancel };
        fixtures::root_event(&mut root, Event::Act { action: Action::Pages(pages::Act::Switcher(act)) });
        for words in [fixtures::words(), fixtures::browser_words()] {
            let limits = crate::Limits::of(&fixtures::domain_limits(), &words).unwrap();
            let mut view = crate::View::new(&limits, &fixtures::domain_limits(), words, fixtures::FORGE);
            fixtures::build(&mut view, &root);
            let heading = switcher_node(&view, Role::Heading { level: 2 }, "What reaches you".as_bytes());
            assert_eq!(heading.focus, if renaming { None } else { Some(1) });
            if renaming {
                assert_eq!(switcher_node(&view, Role::TextField, b"Rename").focus, Some(0));
            }
        }
    }
}

#[test]
fn opening_rename_takes_focus_from_a_shown_document_section() {
    let mut source = super::document::offered(Some(b"# Plan\n\nA draft.\n"), false);
    fixtures::root_event(
        &mut source,
        temper_client_domain::Event::Act { action: Action::Pages(pages::Act::ShowSection { line: 1 }) },
    );
    let chat = source.pages().chat().unwrap();
    let task = chat.task(source.client()).unwrap();
    let mut root = switcher_owner(false);
    for renaming in [false, true, false] {
        let act = if renaming { pages::switcher::Act::Rename } else { pages::switcher::Act::Cancel };
        fixtures::root_event(
            &mut root,
            temper_client_domain::Event::Act { action: Action::Pages(pages::Act::Switcher(act)) },
        );
        for words in [fixtures::words(), fixtures::browser_words()] {
            let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
            builder.open(Role::Slot);
            let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
            crate::frame::switcher(
                &mut builder,
                &root,
                &words,
                &mut bindings,
                fixtures::PROJECT,
                &crate::forge_sign_in::Forges::new(),
                pages::rail::parts(&pages::rail::facts(&root, fixtures::PROJECT, fixtures::NOW)),
            );
            crate::pages::document::pane(
                &mut builder,
                &root,
                chat,
                task,
                &words,
                &mut crate::words::Times::new(fixtures::NOW),
            );
            builder.close();
            let tree = builder.finish();
            let mut sections = 0_usize;
            for section in tree.nodes() {
                if section.styles.as_slice().contains(&crate::look::DOC_SEC) {
                    assert_eq!(section.focus, if renaming { None } else { Some(1) });
                    sections += 1;
                }
            }
            assert_eq!(sections, 1);
        }
    }
}

#[test]
#[expect(clippy::disallowed_types, clippy::disallowed_methods, reason = "ordinary tests inspect semantic menu rows")]
fn the_persons_menu_keeps_accounts_remembered_and_sign_out_ordered() {
    use temper_client_domain::Address;
    let root = fixtures::root_at(Address::Home { project: fixtures::PROJECT }, &fixtures::domain_limits()).0;
    let mut view = fixtures::browser_page_view(&root);
    let nodes = view.tree().nodes();
    let at = nodes.iter().position(|n| n.role == Role::Menu && n.name.as_deref() == Some(b"Pat")).unwrap();
    let menu = &nodes[at..at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap()];
    let items: Vec<_> =
        menu.iter().filter(|n| n.role == Role::MenuItem).map(|n| (n.id, n.name.clone().unwrap())).collect();
    assert_eq!(items.len(), 4);
    for ((id, name), (expected, act)) in items.into_iter().zip([
        (b"Accounts".as_slice(), pages::Act::Go { address: Address::Accounts }),
        (b"What temper remembers about you", pages::Act::Go { address: Address::Remembered }),
        (b"Telling you", pages::Act::Go { address: Address::Told }),
        (b"Sign out", pages::Act::SignOut),
    ]) {
        assert_eq!(name.as_ref(), expected);
        assert_eq!(crate::act(&mut view, id, Act::Press), Some(Action::Pages(act)));
    }
}
