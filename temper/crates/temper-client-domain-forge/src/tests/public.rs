//! Public reader steps (temper/docs/design/client/web/public.md, 4).
use crate::{
    address::Public,
    public::{Doc, Heard, PublicEvent, PublicLimits, PublicRead, Reader, Shown},
};
use jig_client_protocol_wire::public_v1 as jig;
use skein_lib::{Duration, List, Queue, Time, Writer};
use temper_client_protocol_wire::public_v1 as forge;
const PROJECT: u64 = 0x1234_5678_0000_0001;
const TASK: u64 = 0xabcd_ef01_0000_0002;
fn at(seconds: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_secs(seconds))
}
fn limits() -> PublicLimits {
    PublicLimits { jig: jig::CEILINGS, forge: forge::CEILINGS }
}
fn opened(page: Public) -> (Reader, Queue<PublicRead>) {
    let mut reader = Reader::new(&limits());
    let mut out = Queue::uncapped();
    reader.step(Time::ZERO, PublicEvent::Open { page }, &mut out);
    while out.pop().is_some() {}
    (reader, out)
}
fn projects() -> Box<[u8]> {
    let record = jig::PublicProjects::new(
        &jig::CEILINGS,
        jig::PublicProjectsParts { projects: List::with_capacity(0), max_age: 8 },
    )
    .expect("projects");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("small record"));
    record.encode(&mut writer).expect("record encodes");
    writer.finish()
}
fn answer(reader: &mut Reader, out: &mut Queue<PublicRead>, seconds: u64, doc: Doc, heard: Heard) {
    reader.step(at(seconds), PublicEvent::Heard { doc, heard }, out);
}
fn document(reader: &mut Reader, out: &mut Queue<PublicRead>, fresh: u64) {
    answer(reader, out, 0, Doc::JigProjects, Heard::Document { body: projects(), fresh: Duration::from_secs(fresh) });
}
#[test]
fn each_page_reads_its_documents() {
    let mut reader = Reader::new(&limits());
    let mut out = Queue::uncapped();
    for (page, expected) in [
        (Public::Projects, &[(Doc::JigProjects, 0, 1, [None, None])][..]),
        (
            Public::Board { project: PROJECT },
            &[(Doc::JigBoard, 0, 2, [Some(PROJECT), None]), (Doc::ForgeLandings, 2, 2, [Some(PROJECT), None])],
        ),
        (
            Public::Task { project: PROJECT, task: TASK },
            &[(Doc::JigCard, 0, 3, [Some(PROJECT), Some(TASK)]), (Doc::ForgeChange, 2, 1, [Some(PROJECT), Some(TASK)])],
        ),
        (Public::Landings { project: PROJECT }, &[(Doc::ForgeLandings, 2, 2, [Some(PROJECT), None])]),
        (Public::Landing { project: PROJECT, task: TASK }, &[(Doc::ForgeLanding, 2, 3, [Some(PROJECT), Some(TASK)])]),
    ] {
        reader.step(at(0), PublicEvent::Open { page }, &mut out);
        for &(doc, owner, kind, ids) in expected {
            assert_eq!(
                out.pop(),
                Some(PublicRead {
                    doc,
                    owner,
                    version: if owner == 2 { forge::PUBLIC_VERSION } else { jig_client_protocol_wire::PUBLIC_VERSION },
                    kind,
                    ids
                })
            );
        }
        assert!(out.pop().is_none());
    }
}
#[test]
fn it_reads_again_when_freshness_ends() {
    let (mut reader, mut out) = opened(Public::Projects);
    // The HTTP client subtracts Age 3 from max-age 8 before this boundary.
    document(&mut reader, &mut out, 5);
    reader.step(at(4), PublicEvent::Due, &mut out);
    assert!(out.pop().is_none());
    assert_eq!(reader.deadline(), Some(at(5)));
    reader.step(at(5), PublicEvent::Due, &mut out);
    assert_eq!(out.pop().expect("freshness read").doc, Doc::JigProjects);
    assert_eq!(reader.shown(), Shown::Unavailable);
}
#[test]
fn a_hidden_tab_reads_nothing_and_reads_at_once_when_shown() {
    let (mut reader, mut out) = opened(Public::Projects);
    document(&mut reader, &mut out, 5);
    reader.step(at(1), PublicEvent::Visible { visible: false }, &mut out);
    reader.step(at(8), PublicEvent::Due, &mut out);
    assert!(out.pop().is_none());
    assert_eq!(reader.deadline(), None);
    reader.step(at(8), PublicEvent::Visible { visible: true }, &mut out);
    assert_eq!(out.pop().expect("read on showing").doc, Doc::JigProjects);
    reader.step(at(9), PublicEvent::Due, &mut out);
    assert!(out.pop().is_none(), "one request in flight per document");
}
#[test]
fn busy_keeps_the_page_until_its_freshness_ends_then_drops_it() {
    let (mut reader, mut out) = opened(Public::Projects);
    document(&mut reader, &mut out, 5);
    answer(&mut reader, &mut out, 3, Doc::JigProjects, Heard::Busy { retry: Duration::from_secs(3) });
    let Shown::Projects { as_of, .. } = reader.shown() else { panic!("fresh projects retained") };
    assert_eq!(as_of, Some(Duration::from_secs(3)));
    assert_eq!(reader.deadline(), Some(at(5)), "expiry precedes retry");
    reader.step(at(5), PublicEvent::Due, &mut out);
    assert_eq!(reader.shown(), Shown::Unavailable);
    assert!(out.pop().is_none(), "retry is still backed off");
    assert_eq!(reader.deadline(), Some(at(6)));
}
#[test]
fn refusals_back_off_from_retry_after_doubling_to_a_minute() {
    let (mut reader, mut out) = opened(Public::Projects);
    let mut now = 0_u64;
    for wait in [3_u64, 6, 12, 24, 48, 60, 60] {
        answer(&mut reader, &mut out, now, Doc::JigProjects, Heard::Busy { retry: Duration::from_secs(3) });
        now = now.checked_add(wait).expect("small test time");
        assert_eq!(reader.deadline(), Some(at(now)));
        reader.step(at(now), PublicEvent::Due, &mut out);
        assert_eq!(out.pop().expect("retry").doc, Doc::JigProjects);
    }
    answer(
        &mut reader,
        &mut out,
        now,
        Doc::JigProjects,
        Heard::Document { body: projects(), fresh: Duration::from_secs(2) },
    );
    now = now.checked_add(2).expect("test time");
    reader.step(at(now), PublicEvent::Due, &mut out);
    answer(&mut reader, &mut out, now, Doc::JigProjects, Heard::Failed);
    assert_eq!(reader.deadline(), Some(at(now.checked_add(1).expect("test time"))));
}
#[test]
fn unknown_drops_the_page_and_shows_its_main() {
    let (mut reader, mut out) = opened(Public::Board { project: PROJECT });
    answer(&mut reader, &mut out, 0, Doc::JigBoard, Heard::Document { body: board(), fresh: Duration::from_secs(5) });
    answer(
        &mut reader,
        &mut out,
        0,
        Doc::ForgeLandings,
        Heard::Document { body: landings(), fresh: Duration::from_secs(20) },
    );
    reader.step(at(5), PublicEvent::Due, &mut out);
    let _read = out.pop().expect("main re-read");
    answer(&mut reader, &mut out, 5, Doc::JigBoard, Heard::Unknown { fresh: Duration::from_secs(5) });
    assert_eq!(reader.shown(), Shown::Unknown);
    assert!(out.pop().is_none());
    answer(
        &mut reader,
        &mut out,
        6,
        Doc::ForgeLandings,
        Heard::Document { body: landings(), fresh: Duration::from_secs(5) },
    );
    assert_eq!(reader.shown(), Shown::Unknown, "late companion answer cannot restore page");
    reader.step(at(10), PublicEvent::Due, &mut out);
    assert_eq!(out.pop().expect("main re-read").doc, Doc::JigBoard);
    assert!(out.pop().is_none(), "unknown page reads only its main");
}
fn board() -> Box<[u8]> {
    let record = jig::PublicBoard::new(
        &jig::CEILINGS,
        jig::PublicBoardParts {
            project: PROJECT,
            name: b"Public project".as_slice().into(),
            live: List::with_capacity(0),
            live_folded: 0,
            ended: List::with_capacity(0),
        },
    )
    .expect("board");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("small record"));
    record.encode(&mut writer).expect("record");
    writer.finish()
}
fn landings() -> Box<[u8]> {
    let record = forge::PublicLandings::new(
        &forge::CEILINGS,
        forge::PublicLandingsParts { turn: None, landings: List::with_capacity(0), queues: List::with_capacity(0) },
    )
    .expect("landings");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("small record"));
    record.encode(&mut writer).expect("record");
    let record = jig::PublicApplication::new(
        &jig::CEILINGS,
        jig::PublicApplicationParts { tasks: List::with_capacity(0), record: writer.finish() },
    )
    .expect("application");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("small record"));
    record.encode(&mut writer).expect("envelope");
    writer.finish()
}
#[test]
fn a_second_documents_unknown_shows_nothing_of_it() {
    let (mut reader, mut out) = opened(Public::Board { project: PROJECT });
    answer(&mut reader, &mut out, 0, Doc::JigBoard, Heard::Document { body: board(), fresh: Duration::from_secs(5) });
    answer(&mut reader, &mut out, 0, Doc::ForgeLandings, Heard::Unknown { fresh: Duration::from_secs(5) });
    let Shown::Board { landings: None, .. } = reader.shown() else { panic!("board without landings") };
}
#[test]
fn drops_happen_while_hidden() {
    let (mut reader, mut out) = opened(Public::Projects);
    document(&mut reader, &mut out, 5);
    reader.step(at(1), PublicEvent::Visible { visible: false }, &mut out);
    assert_eq!(reader.deadline(), Some(at(5)));
    reader.step(at(5), PublicEvent::Due, &mut out);
    assert_eq!(reader.shown(), Shown::Unavailable);
    assert!(out.pop().is_none());
    assert_eq!(reader.deadline(), None);
}
#[test]
fn invalid_records_and_zero_freshness_never_show() {
    for (body, fresh) in [(Box::from(&b"broken"[..]), 5), (projects(), 0)] {
        let (mut reader, mut out) = opened(Public::Projects);
        answer(&mut reader, &mut out, 0, Doc::JigProjects, Heard::Document { body, fresh: Duration::from_secs(fresh) });
        assert_eq!(reader.shown(), Shown::Unavailable);
        assert_eq!(reader.deadline(), Some(at(1)));
    }
}

#[test]
fn each_document_expires_at_its_own_freshness() {
    let (mut reader, mut out) = opened(Public::Board { project: PROJECT });
    answer(&mut reader, &mut out, 0, Doc::JigBoard, Heard::Document { body: board(), fresh: Duration::from_secs(10) });
    answer(
        &mut reader,
        &mut out,
        0,
        Doc::ForgeLandings,
        Heard::Document { body: landings(), fresh: Duration::from_secs(3) },
    );
    reader.step(at(3), PublicEvent::Due, &mut out);
    let Shown::Board { landings: None, unavailable, .. } = reader.shown() else {
        panic!("fresh board, expired companion")
    };
    assert!(unavailable);
    assert_eq!(out.pop().expect("only companion re-read").doc, Doc::ForgeLandings);
    assert!(out.pop().is_none());
    assert_eq!(reader.deadline(), Some(at(10)), "main expiry stays armed while companion is pending");
}
#[test]
fn a_page_opened_hidden_waits_until_it_is_shown() {
    let mut reader = Reader::new(&limits());
    let mut out = Queue::uncapped();
    reader.step(at(0), PublicEvent::Visible { visible: false }, &mut out);
    reader.step(at(0), PublicEvent::Open { page: Public::Board { project: PROJECT } }, &mut out);
    assert!(out.pop().is_none());
    assert_eq!(reader.deadline(), None);
    reader.step(at(8), PublicEvent::Visible { visible: true }, &mut out);
    assert_eq!(out.pop().expect("main read").doc, Doc::JigBoard);
    assert_eq!(out.pop().expect("companion read").doc, Doc::ForgeLandings);
    assert!(out.pop().is_none());
}
