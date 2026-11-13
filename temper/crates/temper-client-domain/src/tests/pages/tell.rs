//! Tell's filing through the root and its fake engine.
use crate::{
    Action, Address, Domain, Event, Place, Request,
    pages::{
        self,
        tell::{About, Goes, State, Tell},
    },
    tests::fixtures as f,
};
use jig_client::{Down, Up, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn setup() -> Domain {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me = f::start(&mut root, List::uncapped());
    super::welcome::deliver(&mut root, me, c::Delivery::Me(Box::new(f::chats::me())));
    root
}
fn tell(root: &mut Domain, words: &[u8]) -> Queue<Request> {
    tell_about(root, words, None)
}
fn tell_about(root: &mut Domain, words: &[u8], about: Option<About>) -> Queue<Request> {
    super::welcome::event(
        root,
        Event::Act { action: Action::Pages(pages::Act::Tell { project: 1, words: words.into(), about }) },
    )
}
fn sent(out: &Queue<Request>) -> (Token, w::FileFinding) {
    let mut sent = List::uncapped();
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Send { attempt, request: jig_client::Ask::Family { scope, record }, .. },
                },
        } = request
        {
            assert_eq!(*scope, c::Scope::Project(1));
            assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
            let request = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap();
            let w::Request::FileFinding(filing) = request.request() else { panic!("FileFinding") };
            sent.push((*attempt, filing.clone())).unwrap();
        }
    }
    assert_eq!(sent.len(), 1, "one filing");
    sent.swap_remove(0).unwrap()
}
fn answer(root: &mut Domain, attempt: Token, outcome: Option<w::Outcome>) {
    let answer = match outcome {
        Some(outcome) => Some(jig_client::Answered::Family(jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
        })),
        None => None,
    };
    super::welcome::event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::Answer { attempt, envelope: skein_lib::RequestEnvelope::Final, answer, why: None },
            },
        },
    );
}

#[test]
fn tell_files_trimmed_feedback_once_and_keeps_each_outcome() {
    let mut root = setup();
    assert!(root.pages().tell().is_none());
    assert!(tell(&mut root, b" \n\t ").is_empty());
    assert!(root.pages().tell().is_none(), "empty words keep no state");
    let body = b"The export button is grey on dark.\nIt should show.";
    let (attempt, request) = sent(&tell(&mut root, b"  The export button is grey on dark.\nIt should show.  "));
    assert_eq!(request.project(), 1);
    assert_eq!(request.kind(), &w::FindingKind::Feedback);
    assert_eq!(request.severity(), &w::Severity::Medium);
    assert_eq!(request.area(), &None);
    assert_eq!(request.title(), b"The export button is grey on dark.");
    assert_eq!(request.body(), body);
    assert_eq!(
        root.pages().tell(),
        Some(&Tell { serial: 1, project: 1, words: body.as_slice().into(), goes: None, state: State::Sending })
    );
    assert!(tell(&mut root, b"Another").is_empty(), "one in flight");
    assert_eq!(root.pages().tell().unwrap().serial, 1);
    answer(
        &mut root,
        attempt,
        Some(w::Outcome::Filed(w::Filed::new(&w::CEILINGS, w::FiledParts { number: 12 }).unwrap())),
    );
    assert_eq!(root.pages().tell().unwrap().state, State::Filed { number: 12 });
    assert!(tell(&mut root, b" \n ").is_empty());
    assert_eq!(root.pages().tell().unwrap().serial, 1, "empty words keep last outcome");
    let (attempt, _) = sent(&tell(&mut root, b"Another"));
    assert_eq!(root.pages().tell().unwrap().serial, 2);
    let refusal =
        w::FindingRefusal::Full(w::FindingsFull::new(&w::CEILINGS, w::FindingsFullParts { count: 128 }).unwrap());
    answer(
        &mut root,
        attempt,
        Some(w::Outcome::FindingRefused(
            w::FindingRefused::new(&w::CEILINGS, w::FindingRefusedParts { reason: refusal.clone() }).unwrap(),
        )),
    );
    assert_eq!(root.pages().tell().unwrap().state, State::Refused(refusal));
}

#[test]
fn tell_loses_an_ended_or_unknown_request_and_can_send_again() {
    for outcome in [None, Some(w::Outcome::Unknown)] {
        let mut root = setup();
        let out = tell(&mut root, b"Something off");
        let (attempt, _) = sent(&out);
        match outcome {
            Some(outcome) => answer(&mut root, attempt, Some(outcome)),
            None => {
                let mut key = None;
                for request in &out {
                    if let Request::Unlinked {
                        request: jig_client::UnlinkedRequest::Down { down: Down::Save { record } },
                    } = request
                    {
                        key = Some(record.request.key);
                    }
                }
                crate::route::told(
                    &mut root,
                    &f::env(&f::limits()),
                    jig_client::Told::Answered {
                        key: key.expect("filing saved under its Pages key"),
                        outcome: jig_client::Outcome::Lost,
                    },
                    &mut Queue::uncapped(),
                );
            }
        }
        assert_eq!(root.pages().tell().unwrap().state, State::Lost);
        sent(&tell(&mut root, b"Try again"));
        assert_eq!(root.pages().tell().unwrap().serial, 2);
    }
}

#[test]
fn tell_bounds_the_body_and_cuts_its_title_at_a_character() {
    let mut root = setup();
    assert!(tell(&mut root, &[b'x'; 4097]).is_empty());
    let last = root.pages().tell().unwrap();
    assert_eq!(last.serial, 1);
    assert_eq!(last.words.len(), 4097);
    assert_eq!(last.state, State::Long { over: 1 });
    let (attempt, request) = sent(&tell(&mut root, &[b'x'; 4096]));
    assert_eq!(request.body().len(), 4096);
    assert_eq!(request.title().len(), 200);
    assert_eq!(root.pages().tell().unwrap().serial, 2);
    answer(&mut root, attempt, Some(w::Outcome::Unknown));
    for (prefix, length) in [(198, 200), (199, 199)] {
        let body = skein_lib::bytes::concat(&["x".repeat(prefix).as_bytes(), "éz\nNext".as_bytes()]);
        let (attempt, request) = sent(&tell(&mut root, body.as_ref()));
        assert_eq!(request.title().len(), length, "two-byte character at {prefix}");
        assert!(skein_lib::bytes::is_utf8(request.title()));
        assert_eq!(request.body(), body.as_ref());
        answer(&mut root, attempt, Some(w::Outcome::Unknown));
    }
}

fn task_holder(number: u64) -> w::Holder {
    w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number }).unwrap())
}
fn document() -> w::FindingsParts {
    let mut areas = List::with_capacity(3);
    for (name, number) in [(b"docs".as_slice(), 40), (b"tests", 52), (b"more-tests", 52)] {
        areas
            .push(
                w::AreaRow::new(
                    &w::CEILINGS,
                    w::AreaRowParts {
                        name: name.into(),
                        words: name.into(),
                        owner: w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number }).unwrap()),
                        holder: Some(task_holder(number)),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    w::FindingsParts {
        project: 1,
        rows: List::with_capacity(0),
        areas,
        triager: Some(w::AreaOwner::Charter(
            w::AreaCharter::new(&w::CEILINGS, w::AreaCharterParts { number: 14 }).unwrap(),
        )),
        holder: task_holder(60),
        cut: false,
    }
}
fn read_findings(root: &mut Domain, parts: w::FindingsParts) {
    let out = super::welcome::event(
        root,
        Event::Went { place: Place::Address(Address::Inbox { filter: crate::address::Filter::ALL }) },
    );
    let mut found = None;
    for request in &out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { watch, subject: jig_client::Watchable::Family { record, .. } },
                },
        } = request
            && let w::Watch::Findings(subject) = wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch()
            && subject.project() == 1
        {
            found = Some(*watch);
        }
    }
    let watch = found.expect("Needs you reads findings");
    super::welcome::event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::findings(&w::Findings::new(&w::CEILINGS, parts).unwrap()),
                    },
                },
            },
        },
    );
}
fn about(address: Address) -> About {
    About { address, title: Some(b"Prover".as_slice().into()) }
}
#[test]
fn tell_routes_the_pages_task_to_its_first_area() {
    use pages::{team::Holder, tell};
    for address in [
        Address::Talk { project: 1, task: 52 },
        Address::Task { task: 52 },
        Address::Chat { task: 52 },
        Address::Team { project: 1, role: Some(52) },
    ] {
        let mut root = setup();
        read_findings(&mut root, document());
        let about = about(address);
        let expected = Some(Goes::Area { name: b"tests".as_slice().into(), holder: Holder::Task(52) });
        assert_eq!(tell::goes(&root, 1, Some(&about)), expected);
        let (_, file) = sent(&tell_about(&mut root, b"The flake is back", Some(about)));
        assert_eq!(file.area().as_deref(), Some(b"tests".as_slice()));
        assert_eq!(root.pages().tell().unwrap().goes, expected, "destination is kept at the act");
    }
}
#[test]
fn tell_falls_back_to_a_standing_triager_then_owners() {
    use pages::{team::Holder, tell};
    for about in [Some(about(Address::Task { task: 99 })), Some(about(Address::Role { project: 1, charter: 14 })), None]
    {
        for fallback in [0_u8, 1, 2, 3] {
            let mut root = setup();
            let mut document = document();
            let expected = match fallback {
                0 => Goes::Triager { holder: Holder::Task(60), areas: true },
                1 => {
                    document.triager = Some(w::AreaOwner::Task(
                        w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number: 61 }).unwrap(),
                    ));
                    document.holder =
                        w::Holder::Role(w::HolderRole::new(&w::CEILINGS, w::HolderRoleParts { number: 0 }).unwrap());
                    Goes::Owners { areas: true }
                }
                2 => {
                    document.triager = None;
                    Goes::Owners { areas: true }
                }
                _ => {
                    document.triager = None;
                    document.areas.clear();
                    Goes::Owners { areas: false }
                }
            };
            read_findings(&mut root, document);
            assert_eq!(tell::goes(&root, 1, about.as_ref()), Some(expected.clone()));
            let (_, file) = sent(&tell_about(&mut root, b"The flake is back", about.clone()));
            assert_eq!(file.area(), &None);
            assert_eq!(root.pages().tell().unwrap().goes, Some(expected));
        }
    }
}
#[test]
fn tell_files_without_waiting_for_an_unread_document() {
    let mut root = setup();
    let about = about(Address::Talk { project: 1, task: 52 });
    assert_eq!(pages::tell::goes(&root, 1, Some(&about)), None);
    let (_, file) = sent(&tell_about(&mut root, b"The flake is back", Some(about)));
    assert_eq!(file.area(), &None);
    assert_eq!(root.pages().tell().unwrap().goes, None);
}
#[test]
fn tell_body_names_the_page_only_when_the_whole_line_fits() {
    for title in [Some(b"Prover".as_slice().into()), None] {
        let about = About { address: Address::Talk { project: 1, task: 52 }, title };
        let line = if about.title.is_some() {
            b"\n\nPage: /projects/1/talk/52 (Prover)".as_slice()
        } else {
            b"\n\nPage: /projects/1/talk/52".as_slice()
        };
        for length in [16, 4096 - line.len(), 4097 - line.len(), 4090, 4097] {
            let mut root = setup();
            let words = "x".repeat(length).into_bytes().into_boxed_slice();
            let out = tell_about(&mut root, &words, Some(about.clone()));
            if length > 4096 {
                assert!(out.is_empty());
                assert_eq!(root.pages().tell().unwrap().state, State::Long { over: 1 });
                continue;
            }
            let (_, file) = sent(&out);
            let expected =
                if length + line.len() <= 4096 { skein_lib::bytes::concat(&[&words, line]) } else { words.clone() };
            assert_eq!(file.body().len(), expected.len(), "length {length}");
            assert_eq!(file.body(), expected.as_ref(), "length {length}");
            assert_eq!(file.title(), &words[..length.min(200)]);
            assert_eq!(root.pages().tell().unwrap().words.as_ref(), words.as_ref());
        }
    }
    let mut root = setup();
    let (_, file) =
        sent(&tell_about(&mut root, b"The flake is back\nAgain", Some(about(Address::Talk { project: 1, task: 52 }))));
    assert_eq!(file.body(), b"The flake is back\nAgain\n\nPage: /projects/1/talk/52 (Prover)");
    assert_eq!(file.title(), b"The flake is back");
}
