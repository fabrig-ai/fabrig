use super::fixtures as f;
use crate::{
    Domain, Event, Request,
    forge::{
        self,
        dismissed::{self, PersonFact},
    },
};
use jig_client::{Ask, Down, Family, Link, Told, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

const WHAT: PersonFact = PersonFact::ToBegin { project: f::PROJECT };
const OTHER: PersonFact = PersonFact::ToBegin { project: 17 };

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn opening(out: &mut Queue<Request>) -> Option<Token> {
    let mut found = None;
    while let Some(request) = out.pop() {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } } } =
            request
            && subject == dismissed::watch(&f::limits().forge).unwrap()
        {
            assert!(found.is_none(), "one dismissal watch");
            found = Some(watch);
        }
    }
    found
}
fn opened() -> (Domain, Token, Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let frame = f::start(&mut root, List::with_capacity(0));
    let mut out = Queue::uncapped();
    f::confirm(&mut root, frame, f::PARTY, &mut out);
    let watch = opening(&mut out).expect("dismissal watch");
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    (root, watch, frame)
}
fn record(projects: &[u32]) -> Family {
    let mut dismissed = List::with_capacity(wire::DISMISSED);
    for project in projects {
        dismissed
            .push(w::PersonFact::ToBegin(w::ToBegin::new(&w::CEILINGS, w::ToBeginParts { project: *project }).unwrap()))
            .unwrap();
    }
    Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::dismissals(&w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).unwrap()),
    }
}
fn deliver(root: &mut Domain, watch: Token, projects: &[u32]) -> Queue<Request> {
    up(root, Up::FamilyDelivered { watch, record: record(projects) })
}
fn closes(out: &mut Queue<Request>, expected: Token) -> u32 {
    let mut count = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
            && watch == expected
        {
            count = count.checked_add(1).expect("bounded close count");
        }
    }
    count
}
fn link(root: &mut Domain, link: Link) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Link { link }, &mut out);
    out
}

#[test]
fn the_watch_opens_once_the_party_is_known() {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let frame = f::start(&mut root, List::with_capacity(0));
    let mut child = Queue::uncapped();
    dismissed::configure(&mut root.forge, f::limits().forge, &mut child);
    assert!(child.is_empty(), "no party yet");
    let mut out = Queue::uncapped();
    let Watchable::Family { scope, window, record } = dismissed::watch(&f::limits().forge).unwrap() else {
        panic!("family watch")
    };
    assert_eq!(scope, c::Scope::Own);
    assert_eq!(window, c::Window { rows: 0, after: None });
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    assert_eq!(wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch(), &w::Watch::Dismissed);
    assert_eq!(dismissed::handle().asker, f::FORGE_ASKER);
    assert_eq!(dismissed::handle().number, 2);
    f::confirm(&mut root, frame, f::PARTY, &mut out);
    assert!(opening(&mut out).is_some());
    assert!(root.client().document(dismissed::handle()).is_some());
    f::confirm(&mut root, frame, f::PARTY, &mut out);
    assert_eq!(opening(&mut out), None, "same party opens no second watch");
    assert_eq!(opening(&mut link(&mut root, Link::Offline)), None);
    assert_eq!(opening(&mut link(&mut root, Link::Live)), None);
}

#[test]
fn the_document_is_the_sides_facts() {
    let (mut root, watch, _) = opened();
    assert_eq!(root.forge().dismissed(WHAT), None);
    let shown = forge::shown(root.forge());
    deliver(&mut root, watch, &[f::PROJECT]);
    assert_eq!(root.forge().dismissed(WHAT), Some(true));
    assert_eq!(root.forge().dismissed(OTHER), Some(false));
    assert_ne!(forge::shown(root.forge()), shown);
}

#[test]
fn a_later_document_replaces_the_facts() {
    let (mut root, watch, _) = opened();
    deliver(&mut root, watch, &[f::PROJECT]);
    deliver(&mut root, watch, &[17]);
    assert_eq!(root.forge().dismissed(WHAT), Some(false));
    assert_eq!(root.forge().dismissed(OTHER), Some(true));
    deliver(&mut root, watch, &[]);
    assert_eq!(root.forge().dismissed(WHAT), Some(false));
    assert_eq!(root.forge().dismissed(OTHER), Some(false));
}

#[test]
fn an_undecodable_document_ends_the_facts() {
    for bad in [
        Family { bytes: b"bad".as_slice().into(), ..record(&[]) },
        Family { family: 99, ..record(&[]) },
        Family { version: wire::CLIENT_VERSION + 1, ..record(&[]) },
    ] {
        let (mut root, watch, frame) = opened();
        deliver(&mut root, watch, &[f::PROJECT]);
        let mut out = up(&mut root, Up::FamilyDelivered { watch, record: bad });
        assert_eq!(root.forge().dismissed(WHAT), None);
        assert_eq!(closes(&mut out, watch), 1);
        f::confirm(&mut root, frame, f::PARTY, &mut out);
        assert_eq!(opening(&mut out), None, "invalid documents stay ended");
        deliver(&mut root, watch, &[f::PROJECT]);
        assert_eq!(root.forge().dismissed(WHAT), None, "late documents stay ended");
    }
}

#[test]
fn a_refused_watch_keeps_no_rows_until_the_next_sign_in() {
    for end in [c::WatchEnd::NotAllowed, c::WatchEnd::SubjectEnded] {
        let (mut root, watch, frame) = opened();
        deliver(&mut root, watch, &[f::PROJECT]);
        let mut out = up(&mut root, Up::WatchEnded { watch, end });
        assert_eq!(root.forge().dismissed(WHAT), None, "{end:?}");
        assert_eq!(opening(&mut out), None);
        f::confirm(&mut root, frame, f::PARTY, &mut out);
        assert_eq!(opening(&mut out), None, "ended watch does not reopen");
        assert_eq!(opening(&mut link(&mut root, Link::Offline)), None);
        assert_eq!(opening(&mut link(&mut root, Link::Live)), None);
        up(&mut root, Up::SignedOut);
        assert_eq!(root.forge().dismissed(WHAT), None);
        let mut signed_in = up(&mut root, Up::SignedIn);
        let mut frame = None;
        let mut next = None;
        while let Some(request) = signed_in.pop() {
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } },
            } = request
            {
                if subject == Watchable::Jig(c::Subject::Me) {
                    frame = Some(watch);
                }
                if subject == dismissed::watch(&f::limits().forge).unwrap() {
                    next = Some(watch);
                }
            }
        }
        let frame = frame.expect("next sign-in opens me");
        f::confirm(&mut root, frame, f::PARTY, &mut out);
        let next = opening(&mut out).or(next).expect("next sign-in opens dismissals");
        assert_ne!(next, watch);
        deliver(&mut root, next, &[]);
        assert_eq!(root.forge().dismissed(WHAT), Some(false));
    }
}

#[test]
fn signing_out_closes_the_watch_and_clears_the_facts() {
    for ending in [Link::SignedOut, Link::OutOfDate] {
        let (mut root, watch, _) = opened();
        deliver(&mut root, watch, &[f::PROJECT]);
        let mut out = link(&mut root, ending);
        assert_eq!(root.forge().dismissed(WHAT), None);
        assert_eq!(closes(&mut out, watch), 1);
        assert_eq!(opening(&mut link(&mut root, ending)), None);
    }
}

#[test]
fn ask_is_of_the_projects_scope() {
    for project in [0, f::PROJECT, u32::MAX] {
        let Ask::Family { scope, record } =
            dismissed::ask(PersonFact::ToBegin { project }, &f::limits().forge).unwrap()
        else {
            panic!("family request")
        };
        assert_eq!(scope, c::Scope::Project(project));
        assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
        let w::Request::Dismiss(value) =
            wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
        else {
            panic!("dismiss request")
        };
        let w::PersonFact::ToBegin(value) = value.what() else { panic!("to begin") };
        assert_eq!(value.project(), project);
    }
}

#[test]
fn goals_from_reads_the_partys_rows() {
    const CHAT: u64 = 7;
    let (mut root, watch, _) = opened();
    assert!(root.forge().goals_from(f::PROJECT, CHAT).eq([]));
    let record = goal_record(&[
        (f::PROJECT, CHAT, 11, 90),
        (f::PROJECT, CHAT, 19, 10),
        (f::PROJECT, CHAT + 1, 23, 100),
        (f::PROJECT + 1, CHAT, 29, 100),
    ]);
    up(&mut root, Up::FamilyDelivered { watch, record });
    assert!(root.forge().goals_from(f::PROJECT, CHAT).eq([(11, 90), (19, 10)]));
    assert_eq!(
        root.forge().dismissed(PersonFact::GoalFrom { project: f::PROJECT, chat: CHAT, goal: 11, at: 90 }),
        Some(true)
    );
    up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded });
    assert!(root.forge().goals_from(f::PROJECT, CHAT).eq([]));
}

fn goal_record(facts: &[(u32, u64, u64, u64)]) -> Family {
    let mut dismissed = List::with_capacity(wire::DISMISSED);
    for (project, chat, goal, at) in facts {
        dismissed
            .push(w::PersonFact::GoalFrom(
                w::GoalFrom::new(
                    &w::CEILINGS,
                    w::GoalFromParts { project: *project, chat: *chat, goal: *goal, at: *at },
                )
                .unwrap(),
            ))
            .unwrap();
    }
    Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::dismissals(&w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).unwrap()),
    }
}

#[test]
fn the_newest_start_has_one_chip_watch_until_leaving() {
    use crate::pages;
    const CHAT: u64 = f::chat::TASK;
    const NEWEST: u64 = 11;
    const AT: u64 = 90;
    // One watch follows the newest goal for the page.
    const ONE_WATCH: u32 = 1;
    let (mut root, watch, _) = opened();
    event(&mut root, Event::Went { place: crate::Place::Address(crate::Address::Chat { task: CHAT }) });
    let handle = root.pages().chat().unwrap().document;
    crate::route::pages(
        &mut root,
        &f::env(&f::limits()),
        pages::Event::Told {
            told: Told::Page {
                handle,
                page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(f::chat::task())))),
            },
        },
        &mut Queue::uncapped(),
    );
    let record = goal_record(&[(f::PROJECT, CHAT, NEWEST, AT), (f::PROJECT, CHAT, 19, 10)]);
    let mut out = up(&mut root, Up::FamilyDelivered { watch, record: record.clone() });
    let mut chip_watches = List::uncapped();
    while let Some(request) = out.pop() {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { watch, subject: Watchable::Jig(c::Subject::Chips { tasks }) },
                },
        } = request
        {
            assert_eq!(tasks.as_slice(), [NEWEST]);
            chip_watches.push(watch).unwrap();
        }
    }
    assert_eq!(chip_watches.len(), ONE_WATCH);
    let chip_watch = *chip_watches.get(0).unwrap();
    assert_eq!(root.pages().chat().unwrap().goal_from(&root), Some((NEWEST, AT)));
    let mut chip = f::chat::task().chip;
    chip.task = NEWEST;
    chip.title = f::chat::document_text(b"Started from its draft");
    up(
        &mut root,
        Up::Watched {
            watch: chip_watch,
            event: c::WatchEvent::Delivered(c::Delivery::Chips(List::from_boxed(Box::new([chip.clone()])))),
        },
    );
    assert_eq!(root.pages().chat().unwrap().goal_chip(&root), Some(&chip));
    let again = up(&mut root, Up::FamilyDelivered { watch, record });
    for request in &again {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { subject: Watchable::Jig(c::Subject::Chips { .. }), .. },
                },
        } = request
        {
            panic!("the unchanged goal opened another watch");
        }
    }
    let leaving = event(
        &mut root,
        Event::Went { place: crate::Place::Address(crate::Address::Chats { filter: crate::ChatsFilter::All }) },
    );
    let mut closed = false;
    for request in &leaving {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
            && *watch == chip_watch
        {
            closed = true;
        }
    }
    assert!(closed, "leaving closes the goal chip watch");
}
