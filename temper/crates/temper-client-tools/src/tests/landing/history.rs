//! History arguments, pages and refusals at the tools' client boundary.
use crate::{jig, landing::history, tests::fixtures as f};
use jig_client::{self as client, contract as c};
use skein_lib::{Duration, List, Time};
use temper_client_protocol_wire::{self as wire, v1 as w};
const CURSOR: &str = "0100000000000000280000000000000020";
fn cursor() -> [u8; 17] {
    [1, 0, 0, 0, 0, 0, 0, 0, 40, 0, 0, 0, 0, 0, 0, 0, 32]
}
fn event(number: u64, what: w::HistoryWhat) -> w::HistoryEvent {
    w::HistoryEvent::new(
        &w::CEILINGS,
        w::HistoryEventParts {
            project: 1,
            number,
            at: 60_000_000_000,
            source_at: Some(5),
            change: 51,
            repository: w::HistoryRepository::new(&w::CEILINGS, w::HistoryRepositoryParts { forge: 2, repository: 3 })
                .unwrap(),
            landing: if number == 2 {
                Some(
                    w::HistoryInLanding::new(&w::CEILINGS, w::HistoryInLandingParts { task: 61, round: 2, unit: 1 })
                        .unwrap(),
                )
            } else {
                None
            },
            head: Some([0x11; 32]),
            base: Some([0x22; 32]),
            operation: 99,
            attempt: 3,
            what,
        },
    )
    .unwrap()
}
fn formed() -> w::HistoryWhat {
    w::HistoryWhat::Formed(
        w::HistoryFormed::new(
            &w::CEILINGS,
            w::HistoryFormedParts {
                members: List::from_boxed(Box::new([w::HistoryMember::new(
                    &w::CEILINGS,
                    w::HistoryMemberParts { change: 51, unit: 1, head: [0x11; 32] },
                )
                .unwrap()])),
                place: 1,
            },
        )
        .unwrap(),
    )
}
fn page(events: List<w::HistoryEvent>, after: Option<[u8; 17]>) -> w::HistoryReply {
    w::HistoryReply::Page(
        w::HistoryPage::new(&w::CEILINGS, w::HistoryPageParts { events, after, upper: 40, whole_since: Some(0) })
            .unwrap(),
    )
}
fn serve(root: &mut f::Harness, call: u32, result: w::HistoryReply) {
    let answer = w::HistoryAnswer::new(&w::CEILINGS, w::HistoryAnswerParts { result }).unwrap();
    root.serve_readout(
        call,
        client::Readout::Family { family: wire::FORGE, record: Some(wire::history_answer(&answer)) },
    );
}
fn query(root: &f::Harness, call: u32) -> w::HistoryRead {
    let (_, client::Readable::Family { scope, record, .. }) = root.family_reads.get(&call).unwrap() else {
        panic!("family read")
    };
    assert_eq!(*scope, c::Scope::Project(1));
    let w::Read::History(query) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().into_parts().read else {
        panic!("history read")
    };
    query
}
#[test]
fn history_arguments_reach_the_forge_with_a_project_filter_range_and_cursor() {
    for flag in ["--change", "--landing"] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&[
            "landing",
            "history",
            "--project",
            "shop",
            flag,
            "51",
            "--since",
            "2026-10-24T17:00Z",
            "--until",
            "2026-10-24T18:00Z",
            "--after",
            CURSOR,
            "--json",
        ]);
        let query = query(&root, call);
        assert_eq!(query.project(), 1);
        match query.of() {
            w::HistoryOf::Change(task) => {
                assert_eq!(flag, "--change");
                assert_eq!(task.task(), 51);
            }
            w::HistoryOf::Landing(task) => {
                assert_eq!(flag, "--landing");
                assert_eq!(task.task(), 51);
            }
            w::HistoryOf::Project => panic!("filter"),
        }
        assert_eq!(*query.since(), Some(jig::argv::deadline(b"2026-10-24T17:00Z").unwrap().as_nanos()));
        assert_eq!(*query.until(), Some(jig::argv::deadline(b"2026-10-24T18:00Z").unwrap().as_nanos()));
        assert_eq!(*query.after(), Some(cursor()));
        serve(&mut root, call, page(List::with_capacity(0), None));
        assert_eq!(root.answer(call).unwrap().status, 0);
    }
}
#[test]
fn history_rejects_two_filters_bad_times_and_any_non_cursor_before_sending() {
    let cases: &[&[&str]] = &[
        &["landing", "history", "--project", "1", "--change", "51", "--landing", "61"],
        &["landing", "history", "--project", "1", "--since", "2026-02-30T17:00Z"],
        &["landing", "history", "--project", "1", "--until", "landed"],
        &["landing", "history", "--project", "1", "--after", "01"],
        &["landing", "history", "--project", "1", "--after", "010000000000000028000000000000002A"],
        &["landing", "history", "--project", "1", "--after", "010000000000000028000000000000002g"],
        &["landing", "history", "--project", "1", "--after", "0200000000000000280000000000000020"],
        &["landing", "history", "--project", "1", "--after", "010000000000000028000000000000002000"],
        &["landing", "history"],
    ];
    for words in cases {
        assert!(f::parse(words).is_err(), "{words:?}");
    }
    let mut root = f::signed_in(f::PARTY);
    let mut call = f::call(&["landing", "history", "--project", "1", "--change", "51"]);
    *call.values.get_mut(2).unwrap() = Some(jig::call::Values::One(jig::call::Value::Number(61)));
    let call = root.submit_prepared(call, true, None);
    assert_eq!(root.answer(call).unwrap().status, 1);
    assert!(root.family_reads.is_empty());
}
#[test]
fn history_text_prints_ready_formed_header_and_end() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["landing", "history", "--project", "1"]);
    serve(
        &mut root,
        call,
        page(List::from_boxed(Box::new([event(1, w::HistoryWhat::Ready), event(2, formed())])), None),
    );
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 0);
    assert_eq!(answer.stdout.as_ref(), b"project 1: events through 40, whole since 1970-01-01 00:00 UTC\n1 1970-01-01 00:01 UTC ready change 51 head 111111111111 base 222222222222\n2 1970-01-01 00:01 UTC formed change 51 landing 61 round 2 unit 1 head 111111111111 base 222222222222 members [{ change 51 unit 1 head 111111111111 }] place 1\nend\n");
}
#[test]
fn history_json_keeps_all_row_fields_and_named_kind_fields() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["landing", "history", "--project", "1", "--json"]);
    serve(
        &mut root,
        call,
        page(List::from_boxed(Box::new([event(1, w::HistoryWhat::Ready), event(2, formed())])), Some(cursor())),
    );
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.status, 0);
    let json: serde_json::Value = serde_json::from_slice(&answer.stdout).unwrap();
    assert_eq!(json["upper"], 40_u64);
    assert_eq!(json["after"], CURSOR);
    assert_eq!(json["whole_since"], 0_u64);
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], serde_json::from_slice::<serde_json::Value>(br#"{"project":1,"number":1,"at":60000000000,"source_at":5,"change":51,"repository":{"forge":2,"repository":3},"landing":null,"head":"1111111111111111111111111111111111111111111111111111111111111111","base":"2222222222222222222222222222222222222222222222222222222222222222","operation":99,"attempt":3,"what":{"kind":"ready"}}"#).unwrap());
    assert_eq!(
        events[1]["landing"],
        serde_json::from_slice::<serde_json::Value>(br#"{"task":61,"round":2,"unit":1}"#).unwrap()
    );
    assert_eq!(events[1]["what"], serde_json::from_slice::<serde_json::Value>(br#"{"kind":"formed","members":[{"change":51,"unit":1,"head":"1111111111111111111111111111111111111111111111111111111111111111"}],"place":1}"#).unwrap());
}
#[test]
fn a_cut_empty_history_page_prints_the_whole_continuation() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&[
        "landing",
        "history",
        "--project",
        "1",
        "--landing",
        "61",
        "--since",
        "2026-10-24T17:00Z",
        "--until",
        "2026-10-24T18:00Z",
    ]);
    serve(&mut root, call, page(List::with_capacity(0), Some(cursor())));
    let answer = root.answer(call).unwrap();
    assert_eq!(answer.stdout.as_ref(), b"project 1: events through 40, whole since 1970-01-01 00:00 UTC\ntemper ctl DIR landing history --project 1 --landing 61 --since 2026-10-24T17:00Z --until 2026-10-24T18:00Z --after 0100000000000000280000000000000020\n");
}
#[test]
fn a_gone_history_cursor_refuses_in_text_and_json_and_names_the_read_again() {
    for json in [false, true] {
        for first in [Some(35), None] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(
                f::call(&["landing", "history", "--project", "1", "--change", "51", "--after", CURSOR]),
                json,
                None,
            );
            serve(
                &mut root,
                call,
                w::HistoryReply::Gone(
                    w::HistoryGone::new(&w::CEILINGS, w::HistoryGoneParts { from: 32, first }).unwrap(),
                ),
            );
            let answer = root.answer(call).unwrap();
            assert_eq!(answer.status, 2);
            assert!(answer.stdout.is_empty());
            let expected: &[u8] = if first.is_some() {
                b"history from event 32 is gone; the first event that remains after it is 35\nread again: temper ctl DIR landing history --project 1 --change 51"
            } else {
                b"history from event 32 is gone; none remains after it\nread again: temper ctl DIR landing history --project 1 --change 51"
            };
            if json {
                let error: serde_json::Value = serde_json::from_slice(&answer.stderr).unwrap();
                assert_eq!(error["error"].as_str().unwrap().as_bytes(), expected);
            } else {
                assert_eq!(answer.stderr.strip_suffix(b"\n").unwrap(), expected);
            }
        }
    }
}
#[test]
fn history_busy_retries_only_within_the_tools_wait() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["landing", "history", "--project", "1"]);
    serve(&mut root, call, w::HistoryReply::Busy);
    assert!(root.answer(call).is_none());
    root.now = Time::ZERO.saturating_add(Duration::from_secs(1));
    root.pending_reads.push(call);
    root.fire();
    assert_eq!(query(&root, call).project(), 1);
    serve(&mut root, call, w::HistoryReply::Busy);
    root.now = Time::ZERO.saturating_add(root.limits.tools.wait);
    root.fire();
    assert!(root.family_reads.is_empty(), "no read after the wait");
    assert_eq!(root.answer(call).unwrap().status, 1);
}
#[test]
fn change_history_reader_follows_each_cursor_to_end_and_stops_at_sixteen_pages() {
    let (mut pager, client::Readable::Family { record, .. }) = history::read_change_history(1, 51) else {
        panic!("history read")
    };
    let w::Read::History(query) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().into_parts().read else {
        panic!("history query")
    };
    let w::HistoryOf::Change(task) = query.of() else { panic!("change filter") };
    assert_eq!(task.task(), 51);
    for number in 1..=16 {
        let w::HistoryReply::Page(page) =
            page(List::from_boxed(Box::new([event(number, w::HistoryWhat::Ready)])), Some(cursor()))
        else {
            unreachable!()
        };
        let next = pager.accept(page);
        if number < 16 {
            let Some(client::Readable::Family { record, .. }) = next else { panic!("next history read") };
            let w::Read::History(query) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().into_parts().read
            else {
                panic!("history query")
            };
            assert_eq!(*query.after(), Some(cursor()));
        } else {
            assert!(next.is_none());
        }
    }
    assert_eq!(pager.events().len(), 16);
    let (mut pager, _) = history::read_change_history(1, 51);
    let w::HistoryReply::Page(page) = page(List::with_capacity(0), None) else { unreachable!() };
    assert!(pager.accept(page).is_none());
}
#[test]
fn history_ended_kind_names_and_nullable_page_fields_are_stable() {
    let gate = w::HistoryWhat::GateEnded(
        w::HistoryGateEnded::new(
            &w::CEILINGS,
            w::HistoryGateEndedParts { gate: 4, key: [0x33; 32], verdict: w::HistoryGateVerdict::Superseded },
        )
        .unwrap(),
    );
    let round = w::HistoryWhat::RoundEnded(
        w::HistoryRoundEnded::new(&w::CEILINGS, w::HistoryRoundEndedParts { cause: w::HistoryRoundCause::Cut })
            .unwrap(),
    );
    let result = w::HistoryReply::Page(
        w::HistoryPage::new(
            &w::CEILINGS,
            w::HistoryPageParts {
                events: List::from_boxed(Box::new([event(1, gate), event(2, round)])),
                after: None,
                upper: 2,
                whole_since: None,
            },
        )
        .unwrap(),
    );
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["landing", "history", "--project", "1", "--json"]);
    serve(&mut root, call, result);
    let json: serde_json::Value = serde_json::from_slice(&root.answer(call).unwrap().stdout).unwrap();
    assert_eq!(json["after"], serde_json::Value::Null);
    assert_eq!(json["whole_since"], serde_json::Value::Null);
    assert_eq!(json["events"][0]["what"]["kind"], "gate-ended");
    assert_eq!(json["events"][0]["what"]["verdict"], "superseded");
    assert_eq!(json["events"][1]["what"]["kind"], "round-ended");
    assert_eq!(json["events"][1]["what"]["cause"], "cut");
}
