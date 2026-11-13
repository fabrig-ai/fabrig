//! Gone records use the shared landing-history reader and event renderers.
use super::{contains, end_panel, f, serve};
use jig_client::{self as client, contract as c};
use skein_lib::{Duration, List};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn missing(json: bool) -> (f::Harness, u32) {
    let mut root = f::signed_in(f::PARTY);
    let words = f::extended(&["change", "show", "2"], if json { &["--json"] } else { &[] });
    let call = root.submit(words.as_slice());
    serve(&mut root, call, true, true);
    root.pending_reads.push(call);
    end_panel(&mut root, call, c::WatchEnd::Unknown);
    serve(&mut root, call, true, true);
    assert!(root.answer(call).is_none(), "a gone record reads its history before answering");
    (root, call)
}
fn query(root: &f::Harness, call: u32) -> w::HistoryRead {
    let (_, client::Readable::Family { scope, record, .. }) = root.family_reads.get(&call).expect("history read")
    else {
        panic!("family read")
    };
    assert_eq!(*scope, c::Scope::Project(1));
    let w::Read::History(query) = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().into_parts().read else {
        panic!("history read")
    };
    assert_eq!(query.project(), 1);
    let w::HistoryOf::Change(task) = query.of() else { panic!("change filter") };
    assert_eq!(task.task(), 2);
    assert_eq!(*query.since(), None);
    assert_eq!(*query.until(), None);
    query
}
fn event(number: u64) -> w::HistoryEvent {
    w::HistoryEvent::new(
        &w::CEILINGS,
        w::HistoryEventParts {
            project: 1,
            number,
            at: 60_000_000_000,
            source_at: None,
            change: 2,
            repository: w::HistoryRepository::new(&w::CEILINGS, w::HistoryRepositoryParts { forge: 3, repository: 4 })
                .unwrap(),
            landing: None,
            head: None,
            base: None,
            operation: number,
            attempt: 0,
            what: w::HistoryWhat::Ready,
        },
    )
    .unwrap()
}
fn cursor(number: u64) -> [u8; 17] {
    let mut bytes = skein_lib::Writer::new(17);
    bytes.put(&[1]).unwrap();
    bytes.put(&20_u64.to_be_bytes()).unwrap();
    bytes.put(&number.to_be_bytes()).unwrap();
    bytes.finish().as_ref().try_into().unwrap()
}
fn reply(root: &mut f::Harness, call: u32, result: w::HistoryReply) {
    let answer = w::HistoryAnswer::new(&w::CEILINGS, w::HistoryAnswerParts { result }).unwrap();
    root.serve_readout(
        call,
        client::Readout::Family { family: wire::FORGE, record: Some(wire::history_answer(&answer)) },
    );
}
fn page(root: &mut f::Harness, call: u32, numbers: core::ops::Range<u64>, after: Option<[u8; 17]>) {
    let mut events = List::with_capacity(32);
    for number in numbers {
        events.push(event(number)).unwrap();
    }
    reply(
        root,
        call,
        w::HistoryReply::Page(
            w::HistoryPage::new(&w::CEILINGS, w::HistoryPageParts { events, after, upper: 20, whole_since: None })
                .unwrap(),
        ),
    );
}
fn eleven(json: bool) -> crate::jig::Printed {
    let (mut root, call) = missing(json);
    assert_eq!(*query(&root, call).after(), None);
    page(&mut root, call, 1..6, Some(cursor(5)));
    assert!(root.answer(call).is_none(), "read every page, not only the first");
    assert_eq!(*query(&root, call).after(), Some(cursor(5)));
    page(&mut root, call, 6..12, None);
    assert!(root.family_reads.is_empty());
    root.answer(call).expect("history ends the read")
}
#[test]
fn a_gone_record_prints_the_last_eight_of_eleven_events_and_the_full_read() {
    let answer = eleven(false);
    assert_eq!(answer.status, 0);
    assert!(answer.stderr.is_empty());
    assert_eq!(
        answer.stdout.as_ref(),
        concat!(
            "change 2: its record has gone; its last events:\n",
            "4 1970-01-01 00:01 UTC ready change 2\n",
            "5 1970-01-01 00:01 UTC ready change 2\n",
            "6 1970-01-01 00:01 UTC ready change 2\n",
            "7 1970-01-01 00:01 UTC ready change 2\n",
            "8 1970-01-01 00:01 UTC ready change 2\n",
            "9 1970-01-01 00:01 UTC ready change 2\n",
            "10 1970-01-01 00:01 UTC ready change 2\n",
            "11 1970-01-01 00:01 UTC ready change 2\n",
            "all of them: temper ctl landing history --project 1 --change 2\n",
        )
        .as_bytes()
    );
}
#[test]
fn a_gone_records_json_keeps_the_last_eight_whole_events() {
    let answer = eleven(true);
    assert_eq!(answer.status, 0);
    assert!(answer.stderr.is_empty());
    let json: serde_json::Value = serde_json::from_slice(&answer.stdout).unwrap();
    assert_eq!(json.as_object().unwrap().len(), 4);
    assert_eq!(json["task"], 2_u64);
    assert_eq!(json["record"], "gone");
    assert_eq!(json["read"], "temper ctl landing history --project 1 --change 2");
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 8);
    for (event, number) in events.iter().zip(4_u64..=11) {
        assert_eq!(
            *event,
            serde_json::json!({
                "project":1_u32, "number":number, "at":60_000_000_000_u64, "source_at":null,
                "change":2_u64, "repository":{"forge":3_u32,"repository":4_u32}, "landing":null,
                "head":null, "base":null, "operation":number, "attempt":0_u32, "what":{"kind":"ready"}
            })
        );
    }
}
#[test]
fn a_gone_record_and_empty_history_answer_gone_in_both_modes() {
    for json in [false, true] {
        let (mut root, call) = missing(json);
        query(&root, call);
        page(&mut root, call, 0..0, None);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 2);
        assert!(answer.stderr.is_empty());
        if json {
            let value: serde_json::Value = serde_json::from_slice(&answer.stdout).unwrap();
            assert_eq!(value, serde_json::json!({"task":2_u64,"record":"gone","history":"gone"}));
        } else {
            assert_eq!(answer.stdout.as_ref(), b"change 2: its record and its history have gone\n");
        }
    }
}
#[test]
fn sixteen_pages_stop_the_read_and_name_the_remaining_history_before_the_command() {
    let (mut root, call) = missing(false);
    // An empty cut page is not the end of the history.
    page(&mut root, call, 0..0, Some(cursor(1)));
    assert!(root.answer(call).is_none());
    for number in 2..=16 {
        assert_eq!(*query(&root, call).after(), Some(cursor(number - 1)));
        page(&mut root, call, number..number + 1, Some(cursor(number)));
    }
    let answer = root.answer(call).expect("bounded at sixteen pages");
    assert_eq!(answer.status, 0);
    assert!(root.family_reads.is_empty(), "no seventeenth read");
    assert_eq!(
        answer.stdout.as_ref(),
        concat!(
            "change 2: its record has gone; its last events:\n",
            "9 1970-01-01 00:01 UTC ready change 2\n",
            "10 1970-01-01 00:01 UTC ready change 2\n",
            "11 1970-01-01 00:01 UTC ready change 2\n",
            "12 1970-01-01 00:01 UTC ready change 2\n",
            "13 1970-01-01 00:01 UTC ready change 2\n",
            "14 1970-01-01 00:01 UTC ready change 2\n",
            "15 1970-01-01 00:01 UTC ready change 2\n",
            "16 1970-01-01 00:01 UTC ready change 2\n",
            "more events remain\nall of them: temper ctl landing history --project 1 --change 2\n",
        )
        .as_bytes()
    );
}
#[test]
fn an_explicit_project_can_read_history_when_the_task_is_gone_too() {
    for panel_first in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["change", "show", "2", "--project", "1"]);
        if panel_first {
            end_panel(&mut root, call, c::WatchEnd::Unknown);
        }
        root.serve(call, c::ReadResult::None(c::ReadNone::Gone));
        if !panel_first {
            root.pending_reads.push(call);
            end_panel(&mut root, call, c::WatchEnd::Unknown);
        }
        assert!(root.answer(call).is_none());
        query(&root, call);
        page(&mut root, call, 1..2, None);
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        contains(&answer.stdout, b"its record has gone; its last events:");
    }
}
#[test]
fn a_gone_records_history_retries_busy_without_advancing_the_cursor() {
    let (mut root, call) = missing(false);
    let before = query(&root, call);
    reply(&mut root, call, w::HistoryReply::Busy);
    assert!(root.answer(call).is_none());
    root.pending_reads.push(call);
    root.now = root.now.saturating_add(Duration::from_secs(1));
    root.fire();
    assert_eq!(query(&root, call), before);
    page(&mut root, call, 1..2, None);
    assert_eq!(root.answer(call).unwrap().status, 0);
}
