//! Harness reports passed to the drive logic (client/drive.md, 5).
use super::*;
use crate::drive::io::page_event;
use skein_fake_person_browser::harness::session::Report;

#[test]
fn sent_report_preserves_every_field() {
    for document in [false, true] {
        let report = Report::Sent {
            id: b"request-7".as_slice().into(),
            load: b"load-3".as_slice().into(),
            document,
            method: b"POST".as_slice().into(),
            url: b"http://localhost:8367/jig/v1/batch?version=3".as_slice().into(),
        };
        let Some(Event::PageSent { id, load, document: mapped_document, method, url }) = page_event(report) else {
            panic!("sent report must become PageSent");
        };
        assert_eq!(id.as_ref(), b"request-7");
        assert_eq!(load.as_ref(), b"load-3");
        assert_eq!(mapped_document, document);
        assert_eq!(method.as_ref(), b"POST");
        assert_eq!(url.as_ref(), b"http://localhost:8367/jig/v1/batch?version=3");
    }
}

#[test]
fn answered_report_preserves_id() {
    let report = Report::Answered { id: b"request-7".as_slice().into() };
    let Some(Event::PageAnswered { id }) = page_event(report) else {
        panic!("answered report must become PageAnswered");
    };
    assert_eq!(id.as_ref(), b"request-7");
}

#[test]
fn page_report_preserves_trouble() {
    let report = Report::Page {
        trouble: Trouble::Log,
        text: b"resource refused".as_slice().into(),
        url: b"http://localhost:8367/missing".as_slice().into(),
    };
    let Some(Event::PageTrouble { trouble, text, url }) = page_event(report) else {
        panic!("page report must become PageTrouble");
    };
    assert_eq!(trouble, Trouble::Log);
    assert_eq!(text.as_ref(), b"resource refused");
    assert_eq!(url.as_ref(), b"http://localhost:8367/missing");
}

#[test]
fn recorded_reports_answer_web_requests_in_two_waves() {
    let mut machine = ready();
    let reports = [
        (
            0,
            Report::Sent {
                id: b"document".as_slice().into(),
                load: b"load".as_slice().into(),
                document: true,
                method: b"GET".as_slice().into(),
                url: b"http://127.0.0.1:9/projects/1/work".as_slice().into(),
            },
        ),
        (10, Report::Answered { id: b"document".as_slice().into() }),
        (
            20,
            Report::Sent {
                id: b"stream".as_slice().into(),
                load: b"load".as_slice().into(),
                document: false,
                method: b"GET".as_slice().into(),
                url: b"http://127.0.0.1:9/jig/v1/stream?version=3".as_slice().into(),
            },
        ),
        (30, Report::Answered { id: b"stream".as_slice().into() }),
        (
            40,
            Report::Sent {
                id: b"batch".as_slice().into(),
                load: b"load".as_slice().into(),
                document: false,
                method: b"POST".as_slice().into(),
                url: b"http://127.0.0.1:9/jig/v1/batch".as_slice().into(),
            },
        ),
        (50, Report::Answered { id: b"batch".as_slice().into() }),
    ];
    for (millis, report) in reports {
        let event = page_event(report).expect("page report");
        let now = Time::ZERO.saturating_add(Duration::from_millis(millis));
        assert!(machine.step(event, now).is_empty());
    }
    let actions = machine
        .step(Event::Line(b"web requests".as_slice().into()), Time::ZERO.saturating_add(Duration::from_millis(2050)));
    assert_eq!(
        answer(&actions).print(),
        "| GET /jig/v1/stream, wave 1\n| POST /jig/v1/batch, wave 2\n| 2 requests in 2 waves\nok\n"
    );
}
