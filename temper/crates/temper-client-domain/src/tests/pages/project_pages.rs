use super::fixtures;
use crate::{
    Address, ChatsFilter, GoalsFilter, WorkFilter,
    pages::{self, Act, Event, Pages, Request, switcher},
};
use jig_client::{Handle, Readable, Told, Watchable, contract as c};
use skein_lib::{List, Queue, Token};

const OTHER: u32 = 2;
const UNKNOWN: u32 = 3;
// The owner's project and OTHER are the fixture's two membership rows.
const PROJECT_ROWS: u32 = 2;

struct Fixture {
    root: crate::Domain,
    pages: Pages,
    watch: Token,
    out: Queue<Request>,
}

fn fixture() -> Fixture {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let watch = fixtures::start(&mut root, List::with_capacity(0));
    Fixture { root, pages: Pages::new(), watch, out: Queue::uncapped() }
}

fn event(f: &mut Fixture, event: Event) {
    pages::step(&mut f.pages, &fixtures::env(&fixtures::limits()), f.root.client(), event, &mut f.out);
}

fn deliver(f: &mut Fixture, me: c::Me) {
    crate::step(
        &mut f.root,
        &fixtures::env(&fixtures::limits()),
        crate::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: f.watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut Queue::uncapped(),
    );
    event(f, Event::Told { told: Told::Changed { handle: pages::me_handle() } });
}

fn two_projects() -> c::Me {
    let mut me = fixtures::chats::me();
    let owner = me.projects.get(0).expect("first project").clone();
    let mut other = owner.clone();
    other.project = OTHER;
    me.projects = List::with_capacity(PROJECT_ROWS);
    me.projects.push(owner).expect("first");
    me.projects.push(other).expect("second");
    me
}

#[test]
fn project_addresses_wait_until_me_lists_their_project() {
    for address in [
        Address::Home { project: OTHER },
        Address::Settings { project: OTHER },
        Address::Repositories { project: OTHER },
        Address::Ask { project: OTHER },
    ] {
        let mut f = fixture();
        event(&mut f, Event::Went { address });
        assert_eq!(f.pages.project(), None);
        deliver(&mut f, fixtures::chats::me());
        assert_eq!(f.pages.project(), Some(fixtures::PROJECT));
        deliver(&mut f, two_projects());
        assert_eq!(f.pages.project(), Some(OTHER));
    }
}

#[test]
fn going_to_project_pages_opens_the_named_project() {
    for address in [
        Address::Home { project: OTHER },
        Address::Settings { project: OTHER },
        Address::Repositories { project: OTHER },
        Address::Ask { project: OTHER },
    ] {
        let mut f = fixture();
        deliver(&mut f, two_projects());
        event(&mut f, Event::Went { address });
        assert_eq!(f.pages.project(), Some(OTHER));
    }
}

#[test]
fn unlisted_project_addresses_keep_the_open_project() {
    for address in [
        Address::Home { project: UNKNOWN },
        Address::Settings { project: UNKNOWN },
        Address::Repositories { project: UNKNOWN },
        Address::Ask { project: UNKNOWN },
    ] {
        let mut f = fixture();
        deliver(&mut f, two_projects());
        event(&mut f, Event::Act { act: Act::Project { project: OTHER } });
        event(&mut f, Event::Went { address });
        deliver(&mut f, two_projects());
        assert_eq!(f.pages.project(), Some(OTHER));
    }
}

#[test]
fn switching_projects_keeps_home_settings_ask_or_chats() {
    for (address, shown) in [
        (
            Address::Work { project: fixtures::PROJECT, filter: WorkFilter::All },
            Some(Address::Work { project: OTHER, filter: WorkFilter::All }),
        ),
        (
            Address::Work { project: fixtures::PROJECT, filter: WorkFilter::Drafts },
            Some(Address::Work { project: OTHER, filter: WorkFilter::Drafts }),
        ),
        (Address::Home { project: fixtures::PROJECT }, Some(Address::Home { project: OTHER })),
        (Address::Changes { project: fixtures::PROJECT }, Some(Address::Changes { project: OTHER })),
        (Address::Settings { project: fixtures::PROJECT }, Some(Address::Settings { project: OTHER })),
        (Address::Ask { project: fixtures::PROJECT }, Some(Address::Ask { project: OTHER })),
        (Address::Role { project: fixtures::PROJECT, charter: 4 }, Some(Address::Settings { project: OTHER })),
        (Address::Rule { project: fixtures::PROJECT, kind: 2, number: 1 }, Some(Address::Settings { project: OTHER })),
        (Address::Chats { filter: ChatsFilter::All }, None),
        (Address::NewChat { draft: false }, None),
    ] {
        let mut f = fixture();
        deliver(&mut f, two_projects());
        event(&mut f, Event::Went { address: address.clone() });
        while f.out.pop().is_some() {}
        event(&mut f, Event::Act { act: Act::Project { project: OTHER } });
        assert_eq!(f.pages.project(), Some(OTHER));
        assert_eq!(f.pages.address(), &address);
        let mut requested = None;
        while let Some(request) = f.out.pop() {
            match request {
                Request::Show { address } => requested = Some(address),
                Request::Screen { .. }
                | Request::KeepLook { .. }
                | Request::Client(_)
                | Request::Replace { .. }
                | Request::OpenTab { .. } => {}
            }
        }
        assert_eq!(requested, shown);
    }
}

#[test]
fn project_pages_keep_the_switchers_rename_form() {
    for address in [
        Address::Home { project: fixtures::PROJECT },
        Address::Changes { project: fixtures::PROJECT },
        Address::Settings { project: fixtures::PROJECT },
        Address::Repositories { project: fixtures::PROJECT },
        Address::Ask { project: fixtures::PROJECT },
    ] {
        let mut f = fixture();
        deliver(&mut f, two_projects());
        event(&mut f, Event::Act { act: Act::Switcher(switcher::Act::Rename) });
        event(
            &mut f,
            Event::Act { act: Act::Switcher(switcher::Act::RenameWords { words: b"Changed".as_slice().into() }) },
        );
        event(&mut f, Event::Went { address });
        assert_eq!(f.pages.switcher().renaming(), Some(fixtures::PROJECT));
        assert_eq!(f.pages.switcher().rename_words(), b"Changed");
    }
}

#[test]
fn choosing_a_project_keeps_the_repositories_tab() {
    let mut f = fixture();
    deliver(&mut f, two_projects());
    event(&mut f, Event::Went { address: Address::Repositories { project: fixtures::PROJECT } });
    while f.out.pop().is_some() {}
    event(&mut f, Event::Act { act: Act::Project { project: OTHER } });
    assert_eq!(f.pages.project(), Some(OTHER));
    assert_eq!(f.out.pop(), Some(Request::Show { address: Address::Repositories { project: OTHER } }));
}

#[test]
fn choosing_a_project_stays_on_forges_and_accounts() {
    for address in [Address::Forges, Address::Accounts] {
        let mut f = fixture();
        deliver(&mut f, two_projects());
        event(&mut f, Event::Went { address: address.clone() });
        while f.out.pop().is_some() {}
        event(&mut f, Event::Act { act: Act::Project { project: OTHER } });
        assert_eq!(f.pages.project(), Some(OTHER));
        assert_eq!(f.pages.address(), &address);
        assert!(f.out.is_empty());
    }
}

#[test]
fn choosing_a_project_keeps_the_builders_tab_or_team() {
    const OPEN: u32 = 3;
    const CHOSEN: u32 = 4;
    let mut me = two_projects();
    me.projects.get_mut(0).expect("open project").project = OPEN;
    me.projects.get_mut(1).expect("chosen project").project = CHOSEN;
    for (address, expected) in [
        (Address::Builders { project: OPEN }, Address::Builders { project: CHOSEN }),
        (Address::Team { project: OPEN, role: None }, Address::Team { project: CHOSEN, role: None }),
    ] {
        let mut f = fixture();
        deliver(&mut f, me.clone());
        event(&mut f, Event::Went { address });
        while f.out.pop().is_some() {}
        event(&mut f, Event::Act { act: Act::Project { project: CHOSEN } });
        assert_eq!(f.pages.project(), Some(CHOSEN));
        assert_eq!(f.out.pop(), Some(Request::Show { address: expected }));
    }
}

fn work_addresses(project: u32) -> [Address; 5] {
    [
        Address::Work { project, filter: WorkFilter::All },
        Address::Work { project, filter: WorkFilter::Drafts },
        Address::Goals { project, filter: GoalsFilter::All },
        Address::Goals { project, filter: GoalsFilter::Needs },
        Address::Changes { project },
    ]
}

fn work_me() -> c::Me {
    let mut me = two_projects();
    for project in me.projects.iter_mut() {
        let chat = project.templates.get(0).expect("chat template").clone();
        let mut goal = chat.clone();
        goal.number = jig_client::shapes::GOAL;
        goal.tracked = Some(1);
        project.templates = List::from_boxed(Box::from([chat, goal]));
    }
    me
}

fn requests(f: &mut Fixture) -> List<Request> {
    let mut requests = List::uncapped();
    while let Some(request) = f.out.pop() {
        requests.push(request).expect("uncapped requests");
    }
    requests
}

struct WorkReads {
    goal_handles: [Handle; 4],
    chat_handles: [Handle; 4],
    change: Handle,
    chat_query: c::TaskQuery,
}

fn work_reads(f: &mut Fixture, address: &Address) -> WorkReads {
    let project = fixtures::PROJECT;
    let goals = f.pages.goals().expect("goals on Work");
    let chats = f.pages.chats().expect("chats on Work");
    assert_eq!(f.pages.changes().expect("changes on Work").project, project);
    let goal_handles = [goals.live, goals.ended, goals.needs, goals.report];
    let chat_handles = [chats.live, chats.closed, chats.notes, chats.goals];
    let goal_query = c::TaskQuery {
        project: Some(project),
        requester: None,
        root: None,
        parent: None,
        label: Some(7),
        phase: None,
        executor: None,
        tracked: Some(true),
        order: c::QueryOrder::Priority,
    };
    let chat_query = c::TaskQuery {
        requester: Some(c::Requester::Person(fixtures::PARTY)),
        tracked: None,
        order: c::QueryOrder::Activity,
        ..goal_query
    };
    let mut expected = List::uncapped();
    for (handle, query) in [(goals.live, goal_query), (chats.live, chat_query)] {
        expected
            .push(Request::Client(jig_client::Event::Watch {
                handle,
                subject: Watchable::Jig(c::Subject::Tasks { query, window: fixtures::limits().client.domain.rows }),
            }))
            .expect("watch");
    }
    for (handle, query) in [(goals.ended, goal_query), (chats.closed, chat_query)] {
        expected
            .push(Request::Client(jig_client::Event::Read {
                handle,
                read: Readable::Jig(c::Read::Tasks {
                    query: c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query },
                    live: None,
                    ended: None,
                }),
            }))
            .expect("ended read");
    }
    expected
        .push(Request::Client(jig_client::Event::Read {
            handle: goals.needs,
            read: Readable::Jig(c::Read::Inbox {
                group: c::InboxReadGroup::Needs,
                project: Some(project),
                kind: None,
                goal: None,
                after: None,
            }),
        }))
        .expect("goal needs");
    expected
        .push(Request::Client(jig_client::Event::Read {
            handle: chats.notes,
            read: Readable::Jig(c::Read::Notes {
                scope: c::NoteScope::Tasks { project },
                description: None,
                after: None,
            }),
        }))
        .expect("chat documents");
    let opened = requests(f);
    for request in &expected {
        assert!(opened.as_slice().contains(request), "{address:?}: missing {request:?}, got {opened:?}");
    }
    let subject = crate::forge::changes::watch(project, &fixtures::limits().forge).expect("changes watch");
    let mut change = None;
    for request in &opened {
        if let Request::Client(jig_client::Event::Watch { handle, subject: watched }) = request
            && *watched == subject
        {
            assert!(change.replace(*handle).is_none(), "one changes watch");
        }
    }
    let change = change.expect("changes watch opened");
    let mut handles = alloc::collections::BTreeSet::new();
    for handle in goal_handles.into_iter().chain(chat_handles).chain([change]) {
        assert!(handles.insert(handle.number), "independent page handles");
    }
    WorkReads { goal_handles, chat_handles, change, chat_query }
}

#[test]
fn works_addresses_open_and_keep_the_three_reads() {
    let project = fixtures::PROJECT;
    for address in work_addresses(project) {
        let mut f = fixture();
        deliver(&mut f, work_me());
        requests(&mut f);
        event(&mut f, Event::Went { address: address.clone() });
        let WorkReads { goal_handles, chat_handles, change, chat_query } = work_reads(&mut f, &address);
        for destination in work_addresses(project) {
            event(&mut f, Event::Went { address: destination.clone() });
            assert!(f.out.is_empty(), "{address:?} to {destination:?}: {out:?}", out = f.out);
            assert_eq!(f.pages.goals().unwrap().live, goal_handles[0]);
            assert_eq!(f.pages.chats().unwrap().live, chat_handles[0]);
            assert_eq!(f.pages.changes().unwrap().project, project);
            if let Address::Goals { filter, .. } = destination {
                assert_eq!(f.pages.goals().unwrap().filter, filter);
            }
            event(&mut f, Event::Went { address: address.clone() });
            assert!(f.out.is_empty(), "return to {address:?} keeps reads");
        }
        event(&mut f, Event::Went { address: Address::Chats { filter: ChatsFilter::All } });
        assert!(f.pages.goals().is_none());
        assert!(f.pages.changes().is_none());
        assert_eq!(f.pages.chats().unwrap().filter, ChatsFilter::All);
        let left = requests(&mut f);
        for handle in goal_handles.into_iter().chain(chat_handles).chain([change]) {
            assert!(
                left.as_slice().contains(&Request::Client(jig_client::Event::Unwatch { handle })),
                "released {handle:?}"
            );
        }
        let chats = f.pages.chats().unwrap();
        assert!(
            left.as_slice().contains(&Request::Client(jig_client::Event::Watch {
                handle: chats.live,
                subject: Watchable::Jig(c::Subject::Tasks {
                    query: chat_query,
                    window: fixtures::limits().client.domain.rows
                }),
            })),
            "Chats reopens its own query"
        );
        event(&mut f, Event::Went { address });
        requests(&mut f);
        let goals = f.pages.goals().unwrap().live;
        event(&mut f, Event::Went { address: Address::Work { project: OTHER, filter: WorkFilter::Drafts } });
        assert_eq!(f.pages.changes().unwrap().project, OTHER);
        assert!(requests(&mut f).as_slice().contains(&Request::Client(jig_client::Event::Unwatch { handle: goals })));
    }
}

fn view_read(out: &mut Queue<Request>) -> Option<(Handle, u32, Box<[u8]>)> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut found = None;
    while let Some(request) = out.pop() {
        if let Request::Client(jig_client::Event::Read { handle, read: Readable::Family { scope, record, .. } }) =
            request
        {
            let reading = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap();
            if let w::Read::View(view) = reading.read() {
                assert_eq!(scope, c::Scope::Project(view.project()));
                assert!(found.is_none(), "only one view read");
                found = Some((handle, view.project(), view.address().into()));
            }
        }
    }
    found
}

#[test]
#[expect(clippy::too_many_lines, reason = "one view-read table covers opening, revisions, refusal and leaving")]
fn role_pages_read_the_named_view_once_and_keep_the_answer() {
    use crate::pages::roles::Read;
    use temper_client_protocol_wire::{self as wire, v1 as w};
    for (address, wanted) in [
        (Address::Settings { project: 3 }, b"roles".as_slice()),
        (Address::Role { project: 3, charter: 4 }, b"charter 4".as_slice()),
        (Address::Rule { project: 3, kind: 2, number: 1 }, b"rule 2/1".as_slice()),
    ] {
        let mut f = fixture();
        let mut watching = Queue::uncapped();
        f.root.client.step_unlinked(
            skein_lib::Time::ZERO,
            skein_lib::Wall::EPOCH,
            jig_client::Event::Watch {
                handle: crate::forge::ways::handle(),
                subject: Watchable::Jig(c::Subject::Settings),
            },
            &mut watching,
        );
        let mut settings = None;
        for request in &watching {
            if let jig_client::UnlinkedRequest::Down {
                down: jig_client::Down::Open { watch, subject: Watchable::Jig(c::Subject::Settings) },
            } = request
            {
                settings = Some(*watch);
            }
        }
        let settings = settings.expect("settings watch");
        event(&mut f, Event::Went { address: address.clone() });
        assert!(match f.pages.roles().unwrap().read() {
            Read::Reading => true,
            Read::Drawn(_) | Read::Unreadable => false,
        });
        let (mut handle, project, text) = view_read(&mut f.out).expect("view read");
        assert_eq!(project, 3);
        assert_eq!(text.as_ref(), wanted);
        event(&mut f, Event::Went { address: address.clone() });
        assert_eq!(view_read(&mut f.out), None);
        let card = w::Card::new(
            &w::CEILINGS,
            w::CardParts {
                name: b"the reviewer".as_slice().into(),
                who: b"an agent".as_slice().into(),
                facts: List::with_capacity(0),
            },
        )
        .unwrap();
        let view = match address {
            Address::Settings { .. } => w::View::Roles(
                w::RolesView::new(
                    &w::CEILINGS,
                    w::RolesViewParts {
                        roles: List::from_boxed(Box::new([w::CharterView::new(
                            &w::CEILINGS,
                            w::CharterViewParts { charter: 4, card },
                        )
                        .unwrap()])),
                    },
                )
                .unwrap(),
            ),
            Address::Role { .. } => {
                w::View::Charter(w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter: 4, card }).unwrap())
            }
            Address::Rule { .. } => {
                w::View::Rule(w::RuleView::new(&w::CEILINGS, w::RuleViewParts { kind: 2, number: 1, card }).unwrap())
            }
            Address::AllHome
            | Address::Inbox { .. }
            | Address::Chats { .. }
            | Address::NewChat { .. }
            | Address::Work { .. }
            | Address::Remembered
            | Address::Members
            | Address::Limits
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Welcome { .. }
            | Address::Chat { .. }
            | Address::Task { .. }
            | Address::Home { .. }
            | Address::Goals { .. }
            | Address::Changes { .. }
            | Address::Team { .. }
            | Address::Repositories { .. }
            | Address::Models { .. }
            | Address::Builders { .. }
            | Address::Ask { .. }
            | Address::Adopt { .. } => unreachable!("role or rule row"),
        };
        let answer = w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view: view.clone() }).unwrap();
        event(
            &mut f,
            Event::Told {
                told: Told::Page {
                    handle,
                    page: jig_client::Readout::Family { family: wire::FORGE, record: Some(wire::view_answer(&answer)) },
                },
            },
        );
        let Read::Drawn(drawn) = f.pages.roles().unwrap().read() else { panic!("drawn view") };
        assert_eq!(drawn, &view);
        for (newest, refresh) in [(1, true), (1, false), (2, true)] {
            f.root.client.step_unlinked(
                skein_lib::Time::ZERO,
                skein_lib::Wall::EPOCH,
                jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: settings,
                        event: c::WatchEvent::Delivered(c::Delivery::Settings(Box::new(c::SettingsDocument {
                            newest,
                            settings: List::with_capacity(0),
                            held: List::with_capacity(0),
                            pending: List::with_capacity(0),
                        }))),
                    },
                },
                &mut Queue::uncapped(),
            );
            event(&mut f, Event::Told { told: Told::Changed { handle: crate::forge::ways::handle() } });
            let read = view_read(&mut f.out);
            assert_eq!(read.is_some(), refresh, "revision {newest}, {address:?}");
            if let Some((next, scope, words)) = read {
                assert_ne!(next, handle);
                assert_eq!(scope, project);
                assert_eq!(words.as_ref(), wanted);
                handle = next;
                event(
                    &mut f,
                    Event::Told {
                        told: Told::Page {
                            handle,
                            page: jig_client::Readout::Family {
                                family: wire::FORGE,
                                record: Some(wire::view_answer(&answer)),
                            },
                        },
                    },
                );
            }
        }
        event(&mut f, Event::Went { address: Address::Repositories { project: 3 } });
        assert!(f.pages.roles().is_none());
        let mut unwatched = false;
        for request in &f.out {
            unwatched |= *request == Request::Client(jig_client::Event::Unwatch { handle });
        }
        assert!(unwatched);
        event(&mut f, Event::Went { address });
        let (new, _, _) = view_read(&mut f.out).expect("read when reopened");
        assert_ne!(new, handle);
        event(
            &mut f,
            Event::Told {
                told: Told::Page {
                    handle: new,
                    page: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)),
                },
            },
        );
        assert!(match f.pages.roles().unwrap().read() {
            Read::Unreadable => true,
            Read::Drawn(_) | Read::Reading => false,
        });
        event(&mut f, Event::Left);
        assert!(f.pages.roles().is_none());
        assert_eq!(view_read(&mut f.out), None);
    }
}
