//! Team's plain facts and parts (temper/docs/design/client/pages/README.md, Team).
mod areas;
mod sessions;
use crate::pages::team::{self, Facts, Parts, Person, Waits};
use jig_client::contract as c;
use skein_lib::List;

// Names and logins in this fixture fit within this text bound.
const NAME_BYTES: u32 = 64;
const PROJECT: u32 = 3;
const YOU: u64 = 7;
const SAM: u64 = 8;
const UNLISTED: u64 = 9;
const YOURS: Waits = Waits { questions: 2, reviews: 1, proposals: 3, held: 4, person_tasks: 5, cut: false };
const SAMS: Waits = Waits { questions: 1, reviews: 0, proposals: 0, held: 0, person_tasks: 0, cut: true };

fn text(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(bytes.into(), NAME_BYTES).unwrap())
}
pub(super) fn member(party: u64, kind: c::PartyKind, name: &[u8], role: c::HeldRole) -> c::ProjectMember {
    c::ProjectMember { party, kind, role, login: text(b"login"), name: text(name) }
}
fn person(party: u64, you: bool, name: &[u8], role: c::HeldRole, waits: Option<Waits>) -> Person {
    Person { party, you, name: name.into(), role, waits, signed: None }
}
pub(super) fn project(members: Box<[c::ProjectMember]>, cut: bool) -> c::Project {
    c::Project {
        acts: List::with_capacity(0),
        number: PROJECT,
        name: text(b"Project"),
        resources: List::with_capacity(0),
        policy: c::ProjectPolicy {
            templates: List::with_capacity(0),
            ceiling: c::AuthorityDocument::complete(c::Authority {
                models: None,
                tools: 0,
                grants: List::with_capacity(0),
                delegation: c::Delegation { kinds: List::with_capacity(0), tasks: 0, depth: 0 },
                spend: 0,
                deadline: None,
                notes: 0,
                note_resources: List::with_capacity(0),
            }),
            period_spend: 0,
            escalation_role: None,
            projections: List::with_capacity(0),
            roles: List::with_capacity(0),
            requirements: List::with_capacity(0),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        members: List::from_boxed(members),
        current_period: None,
        pools: List::with_capacity(0),
        configured_period: 0,
        configured_pools: List::with_capacity(0),
        configured_pools_cut: false,
        spend: List::with_capacity(0),
        recurring: List::with_capacity(0),
        cut,
    }
}

#[test]
fn gather_lists_each_person_once_the_reader_first() {
    use c::{
        HeldRole::{Member, Observer, Owner},
        PartyKind::{Person as Human, Service},
    };
    let you = member(YOU, Human, b"Reader", Owner);
    let sam = member(SAM, Human, b"Sam", Member);
    let reader = person(YOU, true, b"Reader", Owner, None);
    let teammate = person(SAM, false, b"Sam", Member, None);
    for (name, members, yours, others, cut, expected) in [
        (
            "reader first",
            Box::<[_]>::from([sam.clone(), you.clone()]),
            None,
            Box::<[_]>::from([]),
            false,
            Box::<[_]>::from([reader.clone(), teammate.clone()]),
        ),
        (
            "service left out",
            Box::<[_]>::from([member(SAM, Service, b"Service", Owner)]),
            None,
            Box::<[_]>::from([]),
            false,
            Box::<[_]>::from([]),
        ),
        (
            "first duplicate kept",
            Box::<[_]>::from([
                sam.clone(),
                member(SAM, Human, b"Later", Observer),
                you.clone(),
                member(YOU, Human, b"Later reader", Member),
            ]),
            None,
            Box::<[_]>::from([]),
            false,
            Box::<[_]>::from([reader.clone(), teammate.clone()]),
        ),
        (
            "empty name uses login",
            Box::<[_]>::from([member(SAM, Human, b"", Observer)]),
            None,
            Box::<[_]>::from([]),
            false,
            Box::<[_]>::from([person(SAM, false, b"login", Observer, None)]),
        ),
        (
            "reader and other waits, first pair kept",
            Box::<[_]>::from([sam.clone(), you.clone()]),
            Some(YOURS),
            Box::<[_]>::from([(SAM, SAMS), (SAM, YOURS), (YOU, SAMS)]),
            false,
            Box::<[_]>::from([
                person(YOU, true, b"Reader", Owner, Some(YOURS)),
                person(SAM, false, b"Sam", Member, Some(SAMS)),
            ]),
        ),
        (
            "unlisted pair dropped",
            Box::<[_]>::from([you.clone()]),
            None,
            Box::<[_]>::from([(UNLISTED, SAMS)]),
            false,
            Box::<[_]>::from([reader.clone()]),
        ),
        (
            "reader is no member",
            Box::<[_]>::from([sam.clone()]),
            Some(YOURS),
            Box::<[_]>::from([]),
            false,
            Box::<[_]>::from([teammate.clone()]),
        ),
        ("cut copied", Box::<[_]>::from([you]), None, Box::<[_]>::from([]), true, Box::<[_]>::from([reader])),
    ] {
        assert_eq!(
            team::gather(&project(members, cut), YOU, yours, &others, None, Wall::EPOCH),
            Facts {
                people: List::from_boxed(expected),
                cut,
                areas: None,
                roles: List::with_capacity(0),
                roles_cut: false,
                selected: None,
                card: None
            },
            "{name}",
        );
    }
}

#[test]
fn parts_show_what_the_facts_hold() {
    use c::HeldRole::{Maintainer, Member, Observer, Owner, Policy};
    for (roles, cut, shared) in [
        (Box::<[_]>::from([]), false, None),
        (Box::<[_]>::from([Owner]), false, None),
        (Box::<[_]>::from([Owner, Owner]), false, Some(Owner)),
        (Box::<[_]>::from([Maintainer, Maintainer]), false, Some(Maintainer)),
        (Box::<[_]>::from([Member, Member]), false, Some(Member)),
        (Box::<[_]>::from([Observer, Observer, Observer]), false, Some(Observer)),
        (Box::<[_]>::from([Owner, Member]), false, None),
        (Box::<[_]>::from([Owner, Owner, Member]), false, None),
        (Box::<[_]>::from([Owner, Owner]), true, None),
        (Box::<[_]>::from([Policy { role: 12 }, Policy { role: 12 }]), false, None),
    ] {
        let mut people = List::with_capacity(u32::try_from(roles.len()).unwrap());
        for (at, role) in roles.iter().enumerate() {
            people.push(person(u64::try_from(at).unwrap(), false, b"Person", *role, None)).unwrap();
        }
        let facts = Facts {
            people,
            cut,
            areas: None,
            roles: List::with_capacity(0),
            roles_cut: false,
            selected: None,
            card: None,
        };
        assert_eq!(
            team::parts(&facts),
            Parts { people: !roles.is_empty(), shared, ..Parts::default() },
            "{roles:?} {cut}"
        );
    }
}

use super::fixtures;
use crate::{
    Address,
    pages::{self, Event, Request, needs},
};
use jig_client::{Handle, Readable, Readout, Told, Watchable};
use skein_lib::{Queue, Set, Token};

const OTHER_PROJECT: u32 = 4;
// The document below names the reader and two neighbours.
const THREE_PEOPLE: u32 = 3;
// Nine neighbours exercise the eight-watch bound's far side.
const NINE_OTHERS: u64 = 9;
// The reader and the nine neighbours in the bound fixture.
const TEN_PEOPLE: u32 = 10;
// The ordinary fixture owns a project read and two task watches.
const TWO_OTHERS: u32 = 2;

struct Reads {
    root: crate::Domain,
    me: Token,
    inbox: Token,
    approvals: Token,
    settings: Token,
    read: Option<Handle>,
    others: List<(u64, Handle, Token)>,
    handles: Set<Handle>,
    roles: Option<(Handle, Token)>,
    nows: List<(u64, Handle)>,
    card: Option<(u64, Handle, Token)>,
    activity: List<(Handle, c::Read)>,
    env: skein_lib::Env<crate::Limits>,
    charters: List<(Handle, Box<[u8]>)>,
    closed: List<Handle>,
    findings: List<(Handle, Token, temper_client_protocol_wire::v1::FindingsSubject)>,
}
#[derive(Debug, PartialEq, Eq)]
struct Effects {
    reads: List<u32>,
    watches: List<u64>,
    closed: u32,
}
impl Default for Effects {
    fn default() -> Self {
        Self { reads: List::with_capacity(1), watches: List::with_capacity(team::OTHERS), closed: 0 }
    }
}
fn neighbour(f: &Reads, party: u64) -> (Handle, Token) {
    for (whom, handle, watch) in &f.others {
        if *whom == party {
            return (*handle, *watch);
        }
    }
    panic!("neighbour watch");
}
fn reader() -> c::Me {
    let mut me = fixtures::chats::me();
    me.party.number = YOU;
    me.projects.get_mut(0).unwrap().project = PROJECT;
    me
}
fn reads_fixture() -> Reads {
    let mut limits = fixtures::limits();
    // The shared fixture admits the session; Team adds at most eight neighbours.
    limits.client.domain.handles = limits.client.domain.handles.checked_add(team::OTHERS).unwrap();
    limits.client.domain.watches = limits.client.domain.watches.checked_add(team::OTHERS).unwrap();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let me = fixtures::start(&mut root, List::with_capacity(0));
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        crate::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(reader()))),
                },
            },
        },
        &mut out,
    );
    let approvals = crate::tests::approvals::opening(&out).expect("session approvals");
    let mut inbox = None;
    let mut settings = None;
    for request in &out {
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: Watchable::Jig(c::Subject::Settings) },
                },
        } = request
        {
            settings = Some(*watch);
        }
        if let crate::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: Watchable::Jig(c::Subject::Inbox { .. }) },
                },
        } = request
        {
            inbox = Some(*watch);
        }
    }
    let inbox = inbox.expect("standing inbox");
    Reads {
        root,
        me,
        inbox,
        approvals,
        settings: settings.expect("settings watch"),
        read: None,
        others: List::with_capacity(team::OTHERS),
        handles: Set::with_capacity(team::OTHERS.checked_add(1).unwrap()),
        roles: None,
        nows: List::with_capacity(64),
        card: None,
        activity: List::with_capacity(64),
        env: fixtures::env(&fixtures::limits()),
        charters: List::with_capacity(32),
        closed: List::with_capacity(128),
        findings: List::with_capacity(16),
    }
}
fn client(f: &mut Reads, event: jig_client::Event) -> Queue<jig_client::UnlinkedRequest> {
    let env = fixtures::env(&fixtures::limits());
    let mut out = Queue::uncapped();
    f.root.client.step_unlinked(env.now, env.wall, event, &mut out);
    out
}
fn opened_token(opened: &Queue<jig_client::UnlinkedRequest>) -> Token {
    for request in opened {
        if let jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, .. } } = request {
            return *watch;
        }
    }
    panic!("watch opened");
}
fn close(f: &mut Reads, handle: Handle) {
    let closing = client(f, jig_client::Event::Unwatch { handle });
    for request in &closing {
        if let jig_client::UnlinkedRequest::Down { down: jig_client::Down::Close { watch } } = request {
            client(
                f,
                jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch: *watch, end: c::WatchEnd::Closed } },
            );
        }
    }
}
#[expect(clippy::too_many_lines, reason = "fixture dispatch records Team reads alongside its watches")]
fn page(f: &mut Reads, event: Event) -> Effects {
    let mut out = Queue::uncapped();
    pages::step(&mut f.root.pages, &f.env, &f.root.client, event, &mut out);
    let mut effects = Effects::default();
    while let Some(request) = out.pop() {
        match request {
            Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(c::Read::Project { project }) }) => {
                f.read = Some(handle);
                f.handles.insert(handle).unwrap();
                effects.reads.push(project).unwrap();
            }
            Request::Client(jig_client::Event::Watch { handle, subject: subject @ Watchable::Family { .. } }) => {
                let Watchable::Family { record, .. } = &subject else { unreachable!() };
                let watching = temper_client_protocol_wire::decode_watching(
                    &record.bytes,
                    &temper_client_protocol_wire::v1::CEILINGS,
                )
                .unwrap();
                if let temper_client_protocol_wire::v1::Watch::Findings(findings) = watching.watch() {
                    assert!(!findings.mine());
                    assert_eq!(*findings.state(), Some(temper_client_protocol_wire::v1::FindingState::New));
                    let findings = findings.clone();
                    let opened = client(f, jig_client::Event::Watch { handle, subject });
                    f.findings.push((handle, opened_token(&opened), findings)).unwrap();
                }
            }
            Request::Client(jig_client::Event::Watch {
                handle,
                subject: Watchable::Jig(c::Subject::Tasks { query, window }),
            }) if query.label.is_none() => {
                assert_eq!(window, fixtures::limits().client.domain.rows);
                let Some(c::Requester::Person(party)) = query.requester else { panic!("person requester") };
                assert_eq!(
                    query,
                    c::TaskQuery {
                        project: Some(PROJECT),
                        requester: Some(c::Requester::Person(party)),
                        root: None,
                        parent: None,
                        label: None,
                        phase: None,
                        executor: None,
                        tracked: None,
                        order: c::QueryOrder::Activity,
                    }
                );
                let opened = client(
                    f,
                    jig_client::Event::Watch { handle, subject: Watchable::Jig(c::Subject::Tasks { query, window }) },
                );
                let mut token = None;
                for request in &opened {
                    if let jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, .. } } = request {
                        token = Some(*watch);
                    }
                }
                let token = token.expect("tasks watch");
                f.handles.insert(handle).unwrap();
                f.others.push((party, handle, token)).unwrap();
                effects.watches.push(party).unwrap();
            }
            Request::Client(jig_client::Event::Watch {
                handle,
                subject: Watchable::Jig(c::Subject::Tasks { query, window }),
            }) if query.label == Some(3) && window == team::ROLES => {
                assert_eq!(window, team::ROLES);
                assert!(query.project == Some(PROJECT) || query.project == Some(OTHER_PROJECT));
                assert_eq!(query.order, c::QueryOrder::Number);
                assert_eq!(query.requester, None);
                let opened = client(
                    f,
                    jig_client::Event::Watch { handle, subject: Watchable::Jig(c::Subject::Tasks { query, window }) },
                );
                let token = opened_token(&opened);
                f.roles = Some((handle, token));
            }
            Request::Client(jig_client::Event::Watch {
                handle,
                subject: Watchable::Jig(c::Subject::Task { task }),
            }) => {
                let opened =
                    client(f, jig_client::Event::Watch { handle, subject: Watchable::Jig(c::Subject::Task { task }) });
                let token = opened_token(&opened);
                f.card = Some((task, handle, token));
                f.handles.insert(handle).unwrap();
            }
            Request::Client(jig_client::Event::Read {
                handle,
                read: Readable::Jig(c::Read::Notes { scope: c::NoteScope::Task { project, task }, description, after }),
            }) => {
                assert_eq!(project, PROJECT);
                assert_eq!(description.unwrap().bytes(), b"now");
                assert_eq!(after, None);
                f.nows.push((task, handle)).unwrap();
            }
            Request::Client(jig_client::Event::Read {
                handle,
                read: Readable::Jig(read @ c::Read::Activity { .. }),
            }) => {
                f.activity.push((handle, read)).unwrap();
            }
            Request::Client(jig_client::Event::Unwatch { handle }) if f.handles.remove(&handle) => {
                effects.closed = effects.closed.checked_add(1).unwrap();
                close(f, handle);
                for index in 0..f.others.len() {
                    if f.others.get(index).unwrap().1 == handle {
                        f.others.swap_remove(index).unwrap();
                        break;
                    }
                }
            }
            Request::Client(jig_client::Event::Read { handle, read: Readable::Family { record, .. } }) => {
                let reading = temper_client_protocol_wire::decode_reading(
                    &record.bytes,
                    &temper_client_protocol_wire::v1::CEILINGS,
                )
                .unwrap();
                if let temper_client_protocol_wire::v1::Read::View(view) = reading.read() {
                    f.charters.push((handle, view.address().into())).unwrap();
                }
            }
            Request::Client(jig_client::Event::Unwatch { handle }) => {
                f.closed.push(handle).unwrap();
                close(f, handle);
            }
            Request::Client(_)
            | Request::Push(_)
            | Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::OpenTab { .. }
            | Request::Show { .. }
            | Request::Replace { .. } => {}
        }
    }
    effects
}
fn delivered(f: &mut Reads, handle: Handle, watch: Token, delivery: c::Delivery) -> Effects {
    client(
        f,
        jig_client::Event::Up { up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) } },
    );
    page(f, Event::Told { told: Told::Changed { handle } })
}
fn read_people(members: &[u64]) -> c::Project {
    let mut rows = List::with_capacity(u32::try_from(members.len()).unwrap());
    for party in members {
        rows.push(member(*party, c::PartyKind::Person, b"Person", c::HeldRole::Member)).unwrap();
    }
    project(rows.into_boxed(), false)
}
fn read_shown(members: &[u64], waits: &[(u64, Waits)]) -> team::Shown {
    let mut people = List::with_capacity(u32::try_from(members.len()).unwrap());
    for party in members {
        let mut count = None;
        for (whom, waits) in waits {
            if *whom == *party {
                count = Some(*waits);
                break;
            }
        }
        people.push(person(*party, *party == YOU, b"Person", c::HeldRole::Member, count)).unwrap();
    }
    team::Shown::Read(Facts {
        people,
        cut: false,
        areas: None,
        roles: List::with_capacity(0),
        roles_cut: false,
        selected: None,
        card: None,
    })
}
#[derive(Debug)]
enum Feed {
    Open,
    Me,
    Project(Box<c::Project>),
    Refused,
    WrongValue,
    ReadEnded,
    Tasks { party: u64, lines: Box<[c::TaskLine]>, more: bool },
    TasksEnded(u64),
    Inbox { rows: Box<[c::InboxEntry]>, cut: bool },
    InboxEnded,
    Approvals { projects: Box<[u32]>, more: u32 },
    Answer,
    Leave(Event),
}
fn feed(f: &mut Reads, feed: Feed) -> Effects {
    match feed {
        Feed::Open => page(f, Event::Went { address: Address::Team { project: PROJECT, role: None } }),
        Feed::Me => delivered(f, pages::me_handle(), f.me, c::Delivery::Me(Box::new(reader()))),
        Feed::Project(document) => page(
            f,
            Event::Told {
                told: Told::Page {
                    handle: f.read.unwrap(),
                    page: Readout::Jig(c::ReadResult::Value(c::ReadValue::Project(document))),
                },
            },
        ),
        Feed::Refused => page(
            f,
            Event::Told {
                told: Told::Page {
                    handle: f.read.unwrap(),
                    page: Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)),
                },
            },
        ),
        Feed::WrongValue => page(
            f,
            Event::Told {
                told: Told::Page {
                    handle: f.read.unwrap(),
                    page: Readout::Jig(c::ReadResult::Value(fixtures::acts::question())),
                },
            },
        ),
        Feed::ReadEnded => {
            page(f, Event::Told { told: Told::Ended { handle: f.read.unwrap(), why: jig_client::Why::Unknown } })
        }
        Feed::Tasks { party, lines, more } => {
            let (handle, watch) = neighbour(f, party);
            delivered(f, handle, watch, c::Delivery::Tasks(Box::new(c::Tasks { rows: List::from_boxed(lines), more })))
        }
        Feed::TasksEnded(party) => {
            let (handle, watch) = neighbour(f, party);
            client(f, jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::NotAllowed } });
            page(f, Event::Told { told: Told::Ended { handle, why: jig_client::Why::Unknown } })
        }
        Feed::Inbox { rows, cut } => {
            let mut inbox = crate::tests::documents::inbox(
                List::from_boxed(rows),
                0,
                c::Count { value: 0, limited: false },
                0,
                false,
            );
            inbox.party = YOU;
            if cut {
                inbox.next = Some(c::InboxCursor::Number { number: 1 });
            }
            delivered(f, needs::handle(), f.inbox, c::Delivery::Inbox(Box::new(inbox)))
        }
        Feed::InboxEnded => {
            client(
                f,
                jig_client::Event::Up {
                    up: jig_client::Up::WatchEnded { watch: f.inbox, end: c::WatchEnd::NotAllowed },
                },
            );
            page(f, Event::Told { told: Told::Ended { handle: needs::handle(), why: jig_client::Why::Unknown } })
        }
        Feed::Approvals { projects, more } => {
            let mut rows = List::with_capacity(u32::try_from(projects.len()).unwrap());
            for (i, project) in projects.iter().enumerate() {
                rows.push(crate::tests::approvals::row(u64::try_from(i).unwrap(), *project, None)).unwrap();
            }
            crate::tests::approvals::deliver(&mut f.root, f.approvals, rows.as_slice(), more);
            Effects::default()
        }
        Feed::Answer => {
            let mut answer = fixtures::acts::pages(fixtures::acts::QUESTION, fixtures::acts::question());
            answer.inbox.as_mut().unwrap().cards.get_mut(0).unwrap().project = Some(PROJECT);
            f.root.pages.inbox = answer.inbox.take();
            let effects = page(
                f,
                Event::Act {
                    act: pages::Act::Needs(needs::Act::Choose {
                        object: fixtures::acts::QUESTION,
                        words: b"Yes".as_slice().into(),
                    }),
                },
            );
            f.root.pages.inbox = None;
            effects
        }
        Feed::Leave(event) => page(f, event),
    }
}
fn waiting(whom: c::Whom, kind: c::WaitedFor, ended: bool) -> c::TaskLine {
    let mut line = super::chats::chat_line();
    line.project = PROJECT;
    line.requester = c::Requester::Person(SAM);
    line.waits_on = c::WaitsOn::Party { whom, entries: 2, task: line.chip.task, kind };
    if ended {
        line.ended_at = Some(Wall::EPOCH);
    }
    line
}
fn need(project: u32, object: c::ObjectKey) -> c::InboxEntry {
    fixtures::inbox_entry(object, c::InboxGroup::Questions, project, 1)
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table follows Team’s reads and their lifecycle through deliveries")]
fn team_reads_the_project_and_what_waits_on_each() {
    let mut f = reads_fixture();
    let members = [YOU, SAM, UNLISTED];
    let read = read_shown(&members, &[]);
    let questions = Waits { questions: 1, ..Waits::default() };
    let yours = Waits { questions: 1, reviews: 1, ..Waits::default() };
    let mut nine = List::with_capacity(u32::try_from(NINE_OTHERS).unwrap());
    let mut many = List::with_capacity(TEN_PEOPLE);
    many.push(YOU).unwrap();
    for party in 10..10_u64.checked_add(NINE_OTHERS).unwrap() {
        nine.push(party).unwrap();
        many.push(party).unwrap();
    }
    let mut watched = List::with_capacity(team::OTHERS);
    for party in &nine {
        if watched.len() < team::OTHERS {
            watched.push(*party).unwrap();
        }
    }
    let mut scoped = need(OTHER_PROJECT, c::ObjectKey::Proposal(fixtures::acts::PROPOSAL));
    scoped.projects = List::from_boxed(Box::new([PROJECT]));
    let inbox_rows = Box::<[_]>::from([
        need(PROJECT, fixtures::acts::QUESTION),
        need(OTHER_PROJECT, c::ObjectKey::Question { task: 10, message: 1 }),
    ]);
    let cases = Box::<[_]>::from([
        ("opening", Feed::Open, team::Shown::Reading, None, Box::<[_]>::from([PROJECT]), Box::<[_]>::from([]), 0),
        ("opening again", Feed::Open, team::Shown::Reading, None, Box::<[_]>::from([]), Box::<[_]>::from([]), 0),
        ("me while pending", Feed::Me, team::Shown::Reading, None, Box::<[_]>::from([]), Box::<[_]>::from([]), 0),
        (
            "answer opens others",
            Feed::Project(Box::new(read_people(&members))),
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([SAM, UNLISTED]),
            0,
        ),
        (
            "me after answer",
            Feed::Me,
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([PROJECT]),
            Box::<[_]>::from([]),
            1,
        ),
        (
            "unchanged members keep watches",
            Feed::Project(Box::new(read_people(&members))),
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "another person's question",
            Feed::Tasks {
                party: SAM,
                lines: Box::<[_]>::from([waiting(c::Whom::Person(SAM), c::WaitedFor::Question, false)]),
                more: false,
            },
            read_shown(&members, &[(SAM, questions)]),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "role and other person and ended lines do not count",
            Feed::Tasks {
                party: SAM,
                lines: Box::<[_]>::from([
                    waiting(c::Whom::Role { project: PROJECT, role: 1 }, c::WaitedFor::Question, false),
                    waiting(c::Whom::Person(UNLISTED), c::WaitedFor::Question, false),
                    waiting(c::Whom::Person(SAM), c::WaitedFor::Question, true),
                ]),
                more: true,
            },
            read_shown(&members, &[(SAM, Waits { cut: true, ..Waits::default() })]),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "every other wait kind",
            Feed::Tasks {
                party: SAM,
                lines: Box::<[_]>::from([
                    waiting(c::Whom::Person(SAM), c::WaitedFor::Proposal, false),
                    waiting(c::Whom::Person(SAM), c::WaitedFor::Escalation, false),
                    waiting(c::Whom::Person(SAM), c::WaitedFor::PersonTask, false),
                ]),
                more: false,
            },
            read_shown(&members, &[(SAM, Waits { proposals: 1, held: 1, person_tasks: 1, ..Waits::default() })]),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "ended other's watch",
            Feed::TasksEnded(SAM),
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "reader's inbox scoped",
            Feed::Inbox { rows: inbox_rows.clone(), cut: false },
            read_shown(&members, &[(YOU, questions)]),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "reader's approvals scoped",
            Feed::Approvals { projects: Box::<[_]>::from([PROJECT, OTHER_PROJECT]), more: 0 },
            read_shown(&members, &[(YOU, yours)]),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "answered here removed",
            Feed::Answer,
            read_shown(&members, &[(YOU, Waits { reviews: 1, ..Waits::default() })]),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "all inbox groups and shows_in and cut",
            Feed::Inbox {
                rows: Box::<[_]>::from([
                    need(PROJECT, fixtures::acts::QUESTION),
                    scoped,
                    need(PROJECT, c::ObjectKey::Escalation(11)),
                    need(PROJECT, c::ObjectKey::PersonTask(12)),
                ]),
                cut: true,
            },
            read_shown(
                &members,
                &[(YOU, Waits { reviews: 1, proposals: 1, held: 1, person_tasks: 1, cut: true, ..Waits::default() })],
            ),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "approval cut",
            Feed::Inbox { rows: Box::<[_]>::from([]), cut: false },
            read_shown(&members, &[(YOU, Waits { reviews: 1, ..Waits::default() })]),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "approval cut retained",
            Feed::Approvals { projects: Box::<[_]>::from([PROJECT]), more: 1 },
            read_shown(&members, &[(YOU, Waits { reviews: 1, cut: true, ..Waits::default() })]),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "ended inbox hides reader waits",
            Feed::InboxEnded,
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "refresh before membership change",
            Feed::Me,
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([PROJECT]),
            Box::<[_]>::from([]),
            1,
        ),
        (
            "nine others open eight",
            Feed::Project(Box::new(read_people(many.as_slice()))),
            read_shown(many.as_slice(), &[]),
            Some(TEN_PEOPLE),
            Box::<[_]>::from([]),
            watched.into_boxed(),
            TWO_OTHERS,
        ),
        (
            "refresh before shrinking membership",
            Feed::Me,
            read_shown(many.as_slice(), &[]),
            Some(TEN_PEOPLE),
            Box::<[_]>::from([PROJECT]),
            Box::<[_]>::from([]),
            1,
        ),
        (
            "gone people close",
            Feed::Project(Box::new(read_people(&members))),
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([SAM, UNLISTED]),
            team::OTHERS,
        ),
        (
            "home keeps the document and closes the others",
            Feed::Leave(Event::Went { address: Address::Home { project: PROJECT } }),
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            TWO_OTHERS,
        ),
        (
            "me refreshes the document on Home",
            Feed::Me,
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([PROJECT]),
            Box::<[_]>::from([]),
            1,
        ),
        (
            "Home reads without watching others",
            Feed::Project(Box::new(read_people(&members))),
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "Settings keeps the same project document",
            Feed::Leave(Event::Went { address: Address::Settings { project: PROJECT } }),
            read.clone(),
            Some(THREE_PEOPLE),
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "another project drops the document",
            Feed::Leave(Event::Went { address: Address::Home { project: OTHER_PROJECT } }),
            team::Shown::Reading,
            None,
            Box::<[_]>::from([OTHER_PROJECT]),
            Box::<[_]>::from([]),
            1,
        ),
        ("reopen", Feed::Open, team::Shown::Reading, None, Box::<[_]>::from([PROJECT]), Box::<[_]>::from([]), 1),
        ("refused", Feed::Refused, team::Shown::Unreadable, None, Box::<[_]>::from([]), Box::<[_]>::from([]), 0),
        (
            "retry after refusal",
            Feed::Me,
            team::Shown::Reading,
            None,
            Box::<[_]>::from([PROJECT]),
            Box::<[_]>::from([]),
            1,
        ),
        ("wrong value", Feed::WrongValue, team::Shown::Unreadable, None, Box::<[_]>::from([]), Box::<[_]>::from([]), 0),
        (
            "retry after wrong value",
            Feed::Me,
            team::Shown::Reading,
            None,
            Box::<[_]>::from([PROJECT]),
            Box::<[_]>::from([]),
            1,
        ),
        (
            "ended project read",
            Feed::ReadEnded,
            team::Shown::Unreadable,
            None,
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            0,
        ),
        (
            "left forgets refusal",
            Feed::Leave(Event::Left),
            team::Shown::Reading,
            None,
            Box::<[_]>::from([]),
            Box::<[_]>::from([]),
            1,
        ),
    ]);
    for (name, input, shown, people, reads, watches, closed) in cases {
        assert_eq!(
            feed(&mut f, input),
            Effects { reads: List::from_boxed(reads), watches: List::from_boxed(watches), closed },
            "{name}: effects"
        );
        assert_eq!(team::shown_at(&f.root, PROJECT, fixtures::NOW), shown, "{name}: shown");
        assert_eq!(team::people(&f.root, PROJECT), people, "{name}: people");
        assert_eq!(
            team::shown_at(&f.root, OTHER_PROJECT, fixtures::NOW),
            team::Shown::Reading,
            "{name}: other project"
        );
        assert_eq!(team::people(&f.root, OTHER_PROJECT), None, "{name}: other project's people");
    }
}

fn role(task: u64) -> team::Role {
    team::Role { task, name: b"Prover".as_slice().into(), now: None, spent: 21, budget: 40 }
}
fn card(task: u64) -> team::Card {
    team::Card {
        task,
        name: b"Prover".as_slice().into(),
        made: Some(Wall::EPOCH),
        requester: Some(team::StartedBy::You),
        hold: None,
        paused: false,
        pause: true,
        resume: true,
        talk: false,
        now: None,
        sessions: List::with_capacity(0),
        rules: None,
        wakes: List::with_capacity(0),
        wakes_cut: false,
    }
}
#[test]
fn standing_parts_select_a_listed_card_and_only_read_parts() {
    let mut facts = team::gather(&project(Box::new([]), false), YOU, None, &[], None, Wall::EPOCH);
    facts.roles_cut = true;
    facts.selected = Some(52);
    facts.card = Some(Box::new(card(52)));
    assert_eq!(
        team::parts(&facts),
        Parts { standing_roles: true, roles_cut: true, ..Parts::default() },
        "a cut watch keeps its notice, never an unlisted card"
    );
    facts.roles = List::from_boxed(Box::new([role(41), role(52)]));
    assert_eq!(
        team::parts(&facts),
        Parts {
            standing_roles: true,
            standing: List::from_boxed(Box::new([41, 52])),
            roles_cut: true,
            card: Some(52),
            pause: true,
            ..Parts::default()
        }
    );
    facts.selected = Some(99);
    assert_eq!(team::parts(&facts).card, Some(41), "unlisted selection falls back to the first");
    assert!(!team::parts(&facts).pause, "old card's acts are not the fallback's");
    facts.selected = None;
    facts.card = Some(Box::new(card(41)));
    let c::ReadValue::Escalation(held) = fixtures::acts::held(c::ActKind::Release) else { panic!("held fixture") };
    let detail = facts.card.as_mut().unwrap();
    detail.hold = held.task.hold;
    detail.now = Some(team::Now { body: b"- A finding.\nA paragraph.".as_slice().into(), changed: None, cut: true });
    assert_eq!(
        team::parts(&facts),
        Parts {
            standing_roles: true,
            standing: List::from_boxed(Box::new([41, 52])),
            roles_cut: true,
            card: Some(41),
            now: true,
            ..Parts::default()
        },
        "held for another cause has neither act"
    );
    facts.card.as_mut().unwrap().paused = true;
    assert!(team::parts(&facts).resume);
    facts.card.as_mut().unwrap().resume = false;
    assert!(!team::parts(&facts).resume, "Release must be allowed");
    facts.card.as_mut().unwrap().now.as_mut().unwrap().body = Box::default();
    assert!(!team::parts(&facts).now, "no empty Now part");
    facts.card.as_mut().unwrap().hold = None;
    facts.card.as_mut().unwrap().pause = false;
    assert!(!team::parts(&facts).pause, "Stop must be allowed");
}
fn note(number: u64, description: &[u8], body: &[u8]) -> c::Note {
    c::Note {
        number,
        revision: c::Revision { value: 1 },
        scope: c::NoteScope::Task { project: PROJECT, task: 52 },
        description: text(description),
        body: text(body),
        writer: c::NoteAuthor::Task { task: 52, attempt: 1 },
        tasks: List::with_capacity(0),
        changed: Some(Wall::EPOCH),
        acts: List::with_capacity(0),
        cut: false,
    }
}
#[test]
fn standing_reads_each_now_again_on_line_delivery() {
    let mut f = reads_fixture();
    feed(&mut f, Feed::Open);
    let (handle, watch) = f.roles.unwrap();
    let mut first = super::chats::chat_line();
    first.chip.task = 41;
    first.project = PROJECT;
    first.label = 3;
    first.ended_at = None;
    let mut second = first.clone();
    second.chip.task = 52;
    let rows = List::from_boxed(Box::new([first.clone(), second.clone(), first.clone()]));
    delivered(&mut f, handle, watch, c::Delivery::Tasks(Box::new(c::Tasks { rows: rows.clone(), more: true })));
    let team::Shown::PeopleUnread { facts, failed } = team::shown(&f.root, PROJECT) else {
        panic!("roles do not wait for the people read")
    };
    assert!(!failed);
    assert_eq!(facts.roles.len(), 2);
    feed(&mut f, Feed::Refused);
    let team::Shown::PeopleUnread { facts, failed } = team::shown(&f.root, PROJECT) else {
        panic!("roles survive an unreadable project")
    };
    assert!(failed);
    assert_eq!(facts.roles.len(), 2);
    feed(&mut f, Feed::Me);
    feed(&mut f, Feed::Project(Box::new(read_people(&[YOU]))));
    assert_eq!(f.nows.len(), 1, "Now reads share one in-flight slot");
    let first_handle = f.nows.get(0).unwrap().1;
    page(
        &mut f,
        Event::Told {
            told: Told::Page {
                handle: first_handle,
                page: Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage {
                    rows: List::with_capacity(0),
                    cut: false,
                    before: None,
                    after: None,
                })))),
            },
        },
    );
    assert_eq!(f.nows.len(), 2, "one read per distinct listed role");
    assert_eq!(f.nows.get(0).unwrap().0, 41);
    assert_eq!(f.nows.get(1).unwrap().0, 52);
    assert_eq!(f.card.unwrap().0, 41);
    page(&mut f, Event::Went { address: Address::Team { project: PROJECT, role: Some(52) } });
    assert_eq!(f.card.unwrap().0, 52, "address selects the card's task watch");
    let now_handle = f.nows.get(1).unwrap().1;
    let rows = List::from_boxed(Box::new([
        note(8, b"now", b"Later"),
        note(1, b"now-ish", b"Wrong"),
        note(3, b"now", b"First\n- Next"),
    ]));
    page(
        &mut f,
        Event::Told {
            told: Told::Page {
                handle: now_handle,
                page: Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage {
                    rows,
                    cut: false,
                    before: None,
                    after: None,
                })))),
            },
        },
    );
    let team::Shown::Read(facts) = team::shown(&f.root, PROJECT) else { panic!("Team") };
    assert_eq!(facts.roles.len(), 2);
    assert_eq!(facts.roles.get(1).unwrap().now.as_deref(), Some(b"First".as_slice()));
    assert_eq!(
        facts.card.unwrap().now.unwrap().body.as_ref(),
        b"First\n- Next",
        "Now does not wait for the head document"
    );
    let rows = List::from_boxed(Box::new([first.clone(), second.clone()]));
    delivered(&mut f, handle, watch, c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })));
    assert_eq!(f.nows.len(), 3, "a delivered line reads Now even when last activity is unchanged");
    second.last_activity = Wall::from_nanos(second.last_activity.as_nanos().saturating_add(1));
    let rows = List::from_boxed(Box::new([second]));
    delivered(&mut f, handle, watch, c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })));
    assert_eq!(f.nows.len(), 4, "the removed role’s pending read gives way to the remaining role");
    assert_eq!(f.nows.get(3).unwrap().0, 52);
    let team::Shown::Read(facts) = team::shown(&f.root, PROJECT) else { panic!("Team") };
    assert_eq!(facts.roles.len(), 1);
    assert!(!facts.roles_cut);
    delivered(
        &mut f,
        handle,
        watch,
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::with_capacity(0), more: false })),
    );
    let team::Shown::Read(facts) = team::shown(&f.root, PROJECT) else { panic!("Team") };
    assert_eq!(team::parts(&facts).card, None);
}

fn charter_view(charter: u32, label: &[u8], value: &[u8]) -> temper_client_protocol_wire::v1::View {
    use temper_client_protocol_wire::v1 as w;
    let fact = w::Fact::new(&w::CEILINGS, w::FactParts { label: label.into(), value: value.into() }).unwrap();
    let card = w::Card::new(
        &w::CEILINGS,
        w::CardParts {
            name: b"Prover".as_slice().into(),
            who: b"an agent".as_slice().into(),
            facts: List::from_boxed(Box::new([fact])),
        },
    )
    .unwrap();
    w::View::Charter(w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter, card }).unwrap())
}

#[test]
fn rules_and_wakes_parts_need_the_selected_cards_reads() {
    use crate::pages::roles::Read;
    use skein_lib::{Duration, Wall};
    let mut facts = team::gather(&project(Box::new([]), false), YOU, None, &[], None, Wall::EPOCH);
    facts.roles = List::from_boxed(Box::new([role(52)]));
    facts.card = Some(Box::new(card(52)));
    let asked = charter_view(13, b"Asked to", b"Reproduce before routing.");
    let other = charter_view(13, b"Model", b"No rules");
    let empty = charter_view(13, b"Asked to", b"");
    for (read, expected) in [
        (Read::Reading, false),
        (Read::Unreadable, false),
        (Read::Drawn(&other), false),
        (Read::Drawn(&empty), false),
        (Read::Drawn(&asked), true),
    ] {
        facts.card.as_mut().unwrap().rules = team::asked(13, read);
        assert_eq!(team::parts(&facts).rules, expected);
    }
    assert!(team::asked(14, Read::Drawn(&asked)).is_none(), "a different charter is not this role's rules");
    facts.card.as_mut().unwrap().rules = Some(team::Rules { charter: 13, asked: Box::default() });
    assert!(!team::parts(&facts).rules, "an empty fact cannot draw an empty rules part");
    facts.card.as_mut().unwrap().rules = team::asked(13, Read::Drawn(&asked));
    facts.card.as_mut().unwrap().wakes_cut = true;
    assert!(!team::parts(&facts).wakes_cut, "cut cannot draw an empty part");
    let timer = team::wakes::Row {
        interest: team::wakes::Interest::Timer {
            at: Wall::EPOCH,
            period: Some(Duration::from_secs(3600)),
            wakes: true,
        },
        class: c::WakeRule::Never,
    };
    facts.card.as_mut().unwrap().wakes = List::from_boxed(Box::new([timer]));
    let parts = team::parts(&facts);
    assert!(parts.rules && parts.wakes && parts.wakes_cut && parts.right, "cut is shown even if all subscriptions fit");
    facts.card.as_mut().unwrap().task = 99;
    let parts = team::parts(&facts);
    assert!(!parts.rules && !parts.wakes && !parts.wakes_cut && !parts.right, "stale card parts are hidden");
    facts.roles = List::with_capacity(0);
    assert_eq!(team::parts(&facts).card, None);
}

#[test]
fn wake_rows_group_connectors_at_the_first_topic_and_keep_policy_classes() {
    use skein_lib::{Duration, Wall};
    use team::wakes::{Interest, Row};
    let mut task = fixtures::chat::task();
    task.wake.notices = c::WakeRule::Batch { count: 5, age: Duration::from_secs(7200) };
    task.wake.news = c::WakeRule::Paced { span: Duration::from_secs(1800) };
    task.wake.timers = false;
    let timer = c::SubscriptionKind::Timer { at: Wall::EPOCH, period: Some(Duration::from_secs(604_800)) };
    let kinds = [
        c::SubscriptionKind::Topic { connector: temper_client_protocol_wire::FORGE, topic: 9 },
        timer,
        c::SubscriptionKind::Topic { connector: 4, topic: 1 },
        c::SubscriptionKind::Topic { connector: temper_client_protocol_wire::FORGE, topic: 10 },
        c::SubscriptionKind::Task { target: 61, held: true, result: false },
        c::SubscriptionKind::Topic { connector: 4, topic: 2 },
        c::SubscriptionKind::Task { target: 62, held: false, result: true },
        c::SubscriptionKind::Task { target: 63, held: true, result: true },
        c::SubscriptionKind::Timer { at: Wall::from_nanos(1), period: None },
    ];
    task.subscriptions.entries = List::with_capacity(u32::try_from(kinds.len()).unwrap());
    for (n, kind) in kinds.into_iter().enumerate() {
        task.subscriptions.entries.push(c::Subscription { number: u64::try_from(n).unwrap(), kind }).unwrap();
    }
    assert_eq!(
        team::wakes::rows(&task.subscriptions, &task.wake).as_slice(),
        &[
            Row {
                interest: Interest::Topics { connector: temper_client_protocol_wire::FORGE, count: 2 },
                class: task.wake.news
            },
            Row {
                interest: Interest::Timer { at: Wall::EPOCH, period: Some(Duration::from_secs(604_800)), wakes: false },
                class: c::WakeRule::Never
            },
            Row { interest: Interest::Topics { connector: 4, count: 2 }, class: task.wake.news },
            Row { interest: Interest::Task { target: 61, held: true, result: false }, class: task.wake.notices },
            Row { interest: Interest::Task { target: 62, held: false, result: true }, class: task.wake.notices },
            Row { interest: Interest::Task { target: 63, held: true, result: true }, class: task.wake.notices },
            Row {
                interest: Interest::Timer { at: Wall::from_nanos(1), period: None, wakes: false },
                class: c::WakeRule::Never
            },
        ]
    );
}

#[test]
fn team_reads_the_selected_charter_and_closes_it_on_switch_or_leave() {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut f = reads_fixture();
    feed(&mut f, Feed::Open);
    let (handle, watch) = f.roles.unwrap();
    let mut line = super::chats::chat_line();
    line.chip.task = 52;
    line.project = PROJECT;
    line.label = 3;
    line.ended_at = None;
    delivered(
        &mut f,
        handle,
        watch,
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::from_boxed(Box::new([line.clone()])), more: false })),
    );
    assert!(f.charters.is_empty(), "no charter invented before the task document");
    let (_, handle, watch) = f.card.unwrap();
    let mut document = fixtures::chat::task();
    document.chip.task = 52;
    document.project = PROJECT;
    document.chip.executor = c::ChipExecutor::Agent { charter: 13 };
    delivered(&mut f, handle, watch, c::Delivery::Task(Box::new(document)));
    assert_eq!(f.charters.len(), 1);
    assert_eq!(f.charters.get(0).unwrap().1.as_ref(), b"charter 13");
    let charter_handle = f.charters.get(0).unwrap().0;
    let view = charter_view(13, b"Asked to", b"Reproduce before routing.");
    let answer = w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view }).unwrap();
    page(
        &mut f,
        Event::Told {
            told: Told::Page {
                handle: charter_handle,
                page: Readout::Family { family: wire::FORGE, record: Some(wire::view_answer(&answer)) },
            },
        },
    );
    let shown = team::shown(&f.root, PROJECT);
    assert_eq!(
        shown.facts().unwrap().card.as_ref().unwrap().rules.as_ref().unwrap().asked.as_ref(),
        b"Reproduce before routing."
    );
    let settings = f.settings;
    let mut charter_handle = charter_handle;
    for (newest, expected) in [(1, 2), (1, 2), (2, 3)] {
        delivered(
            &mut f,
            crate::forge::ways::handle(),
            settings,
            c::Delivery::Settings(Box::new(c::SettingsDocument {
                newest,
                settings: List::with_capacity(0),
                held: List::with_capacity(0),
                pending: List::with_capacity(0),
            })),
        );
        assert_eq!(f.charters.len(), expected, "only newer settings refresh");
        charter_handle = f.charters.get(expected - 1).unwrap().0;
        page(
            &mut f,
            Event::Told {
                told: Told::Page {
                    handle: charter_handle,
                    page: Readout::Family { family: wire::FORGE, record: Some(wire::view_answer(&answer)) },
                },
            },
        );
    }
    page(&mut f, Event::Told { told: Told::Ended { handle: charter_handle, why: jig_client::Why::Unknown } });
    assert!(!team::parts(team::shown(&f.root, PROJECT).facts().unwrap()).rules, "failed reads draw no rules");
    page(&mut f, Event::Went { address: Address::Home { project: PROJECT } });
    assert!(f.closed.as_slice().contains(&charter_handle), "leaving Team closes the charter read");
    assert!(!team::parts(team::shown(&f.root, PROJECT).facts().unwrap()).rules);
    page(&mut f, Event::Went { address: Address::Team { project: PROJECT, role: None } });
    let (_, handle, watch) = f.card.unwrap();
    let mut document = fixtures::chat::task();
    document.chip.task = 52;
    document.project = PROJECT;
    document.chip.executor = c::ChipExecutor::Agent { charter: 13 };
    delivered(&mut f, handle, watch, c::Delivery::Task(Box::new(document)));
    assert_eq!(f.charters.len(), 4, "return reads anew");
    let charter_handle = f.charters.get(3).unwrap().0;
    let (handle, watch) = f.roles.unwrap();
    line.chip.task = 53;
    delivered(
        &mut f,
        handle,
        watch,
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::from_boxed(Box::new([line])), more: false })),
    );
    assert!(f.closed.as_slice().contains(&charter_handle), "switching cards closes the old charter");
}

use skein_lib::Wall;
use temper_client_protocol_wire::{self as wire, v1 as w};
const DAY: u64 = 86_400_000_000_000;
const WALL: Wall = Wall::from_nanos(20 * DAY);

pub(super) fn landing(task: u64, at: u64, parties: &[u64]) -> w::ChangesLanded {
    let mut signed = List::with_capacity(u32::try_from(parties.len()).unwrap());
    for party in parties {
        signed
            .push(
                w::ChangesSigned::new(&w::CEILINGS, w::ChangesSignedParts { party: *party, head: [3; 32], at: 0 })
                    .unwrap(),
            )
            .unwrap();
    }
    w::ChangesLanded::new(
        &w::CEILINGS,
        w::ChangesLandedParts {
            task,
            at,
            signed,
            title: b"Change".as_slice().into(),
            commit: [4; 32],
            default_branch: true,
        },
    )
    .unwrap()
}
pub(super) fn changes(rows: &[w::ChangesLanded], more: u32) -> w::ChangesPanel {
    w::ChangesPanel::new(
        &w::CEILINGS,
        w::ChangesPanelParts {
            project: PROJECT,
            landing: b"main".as_slice().into(),
            queues: List::with_capacity(0),
            more: 0,
            landed: List::with_capacity(0),
            landed_more: 0,
            landed_default: List::from_boxed(rows.into()),
            landed_default_more: more,
        },
    )
    .unwrap()
}

#[test]
fn signoffs_count_distinct_changes_in_the_week_with_a_floor_only_when_cut_inside_it() {
    let document = read_people(&[YOU, SAM, UNLISTED]);
    let one = landing(1, WALL.as_nanos() - DAY, &[SAM]);
    let two = landing(2, WALL.as_nanos() - 3 * DAY, &[SAM, YOU]);
    let old = landing(3, WALL.as_nanos() - 8 * DAY, &[SAM]);
    for (rows, more, reader, sam) in [
        (
            Box::<[_]>::from([one.clone(), two.clone(), old.clone()]),
            0,
            "Signed one last week.",
            "Signed two last week.",
        ),
        (
            Box::<[_]>::from([one.clone(), two.clone(), two.clone()]),
            0,
            "Signed one last week.",
            "Signed two last week.",
        ),
        (Box::<[_]>::from([one.clone(), two.clone()]), 1, "Signed 1+ last week.", "Signed 2+ last week."),
        (Box::<[_]>::from([one, two, old]), 1, "Signed one last week.", "Signed two last week."),
        (
            Box::<[_]>::from([
                landing(4, WALL.as_nanos() - 7 * DAY, &[SAM]),
                landing(5, WALL.as_nanos(), &[YOU]),
                landing(6, WALL.as_nanos() + 1, &[SAM]),
            ]),
            0,
            "Signed one last week.",
            "Signed one last week.",
        ),
    ] {
        let panel = changes(&rows, more);
        let facts = team::gather(&document, YOU, None, &[], Some(&panel), WALL);
        let parts = team::parts(&facts);
        assert_eq!(parts.signed(YOU), Some(reader.as_bytes()));
        assert_eq!(parts.signed(SAM), Some(sam.as_bytes()));
        assert_eq!(parts.signed(UNLISTED), None, "zero is silent");
    }
    let facts = team::gather(&document, YOU, None, &[], None, WALL);
    assert!(team::parts(&facts).signed.is_empty(), "changes unread");
    let mut members = List::uncapped();
    for party in 1..=10 {
        members.push(party).unwrap();
    }
    let panel = changes(&[landing(7, WALL.as_nanos(), &[10])], 0);
    let facts = team::gather(&read_people(members.as_slice()), 1, None, &[], Some(&panel), WALL);
    assert_eq!(facts.people.get(9).unwrap().waits, None);
    assert_eq!(team::parts(&facts).signed(10), Some(b"Signed one last week.".as_slice()), "past eight others");
    for (count, floor, expected) in [
        (9, false, "Signed nine last week."),
        (10, false, "Signed 10 last week."),
        (1, true, "Signed 1+ last week."),
        (0, true, ""),
    ] {
        let mut facts = facts.clone();
        facts.people.get_mut(9).unwrap().signed = Some(team::Signed { count, floor });
        assert_eq!(team::parts(&facts).signed(10).unwrap_or(b""), expected.as_bytes());
    }
}

#[test]
fn team_watches_signoffs_delivers_counts_and_closes_the_watch_on_leaving() {
    let mut f = reads_fixture();
    let mut out = Queue::uncapped();
    pages::step(
        &mut f.root.pages,
        &fixtures::env(&fixtures::limits()),
        &f.root.client,
        Event::Went { address: Address::Team { project: PROJECT, role: None } },
        &mut out,
    );
    let mut changes_watch = None;
    for request in &out {
        if let Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(c::Read::Project { .. }) }) =
            request
        {
            f.read = Some(*handle);
        }
        if let Request::Client(jig_client::Event::Watch { handle, subject }) = request
            && Some(subject.clone()) == crate::forge::changes::watch(PROJECT, &fixtures::limits().forge)
        {
            assert!(changes_watch.is_none(), "one changes watch");
            changes_watch = Some((*handle, subject.clone()));
        }
    }
    let (handle, subject) = changes_watch.expect("Team watches changes");
    let opened = client(&mut f, jig_client::Event::Watch { handle, subject });
    let mut watch = None;
    for request in &opened {
        if let jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch: token, .. } } = request {
            watch = Some(*token);
        }
    }
    feed(&mut f, Feed::Project(Box::new(read_people(&[YOU, SAM]))));
    let panel = changes(&[landing(1, fixtures::NOW.as_nanos(), &[SAM])], 0);
    client(
        &mut f,
        jig_client::Event::Up {
            up: jig_client::Up::FamilyDelivered {
                watch: watch.unwrap(),
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::changes_panel(&panel),
                },
            },
        },
    );
    page(&mut f, Event::Told { told: Told::Changed { handle } });
    let team::Shown::Read(facts) = team::shown_at(&f.root, PROJECT, fixtures::NOW) else { panic!("people read") };
    assert_eq!(team::parts(&facts).signed(SAM), Some(b"Signed one last week.".as_slice()));
    page(&mut f, Event::Told { told: Told::Ended { handle, why: jig_client::Why::Unknown } });
    let team::Shown::Read(facts) = team::shown_at(&f.root, PROJECT, fixtures::NOW) else { panic!("people stay read") };
    assert!(team::parts(&facts).signed.is_empty(), "failed changes do not claim counts");
    out = Queue::uncapped();
    pages::step(
        &mut f.root.pages,
        &fixtures::env(&fixtures::limits()),
        &f.root.client,
        Event::Went { address: Address::Models { project: PROJECT } },
        &mut out,
    );
    let mut closed = 0_u32;
    for request in &out {
        if let Request::Client(jig_client::Event::Unwatch { handle: ended }) = request
            && *ended == handle
        {
            closed += 1;
        }
    }
    assert_eq!(closed, 1, "leaving Team closes its changes watch");
}
