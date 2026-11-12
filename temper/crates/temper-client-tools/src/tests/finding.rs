//! Each finding act and the subject/read's printed forms.
use super::fixtures as f;
use crate::{finding, jig::{call::{Value, Values}, rows}};
use jig_client::{self as client, contract as c};
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};

fn root() -> f::Harness {
    let mut root = f::started(List::with_capacity(0));
    let mut me = f::me(f::PARTY);
    me.projects.get_mut(0).unwrap().project = 3;
    root.deliver(c::Delivery::Me(Box::new(me)));
    root
}
fn request(root: &f::Harness, call: u32) -> w::Request {
    let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family request") };
    assert_eq!(*scope, c::Scope::Project(3));
    wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request
}
fn held(task: u64) -> w::Holder { w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number: task }).unwrap()) }
fn row() -> w::FindingRow {
    w::FindingRow::new(&w::CEILINGS, w::FindingRowParts { number: 12, kind: w::FindingKind::Red, severity: w::Severity::Medium,
        area: Some(b"tests".as_slice().into()), state: w::FindingState::New, title: b"Flaky boot".as_slice().into(), holder: held(40), at: 77, why: Some(b"words, not code".as_slice().into()) }).unwrap()
}
fn document() -> w::Findings {
    let owner = w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number: 40 }).unwrap());
    let area = w::AreaRow::new(&w::CEILINGS, w::AreaRowParts { name: b"tests".as_slice().into(), words: b"Tests and their fixtures".as_slice().into(), owner, holder: Some(held(40)) }).unwrap();
    w::Findings::new(&w::CEILINGS, w::FindingsParts { project: 3, rows: List::from_boxed(Box::new([row()])), areas: List::from_boxed(Box::new([area])), triager: None, holder: held(41), cut: true }).unwrap()
}
fn deliver(root: &mut f::Harness, state: Option<w::FindingState>, mine: bool) {
    let watch = root.watch_token(&finding::subject(3, state, mine));
    root.client(client::Event::Up { up: client::Up::FamilyDelivered { watch, record: client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::findings(&document()) } } });
}
#[test]
fn finding_acts_send_their_fields_and_print_the_committed_holder() {
    let mut root = root();
    let mut call = f::call(&["finding", "file", "--project", "shop", "--kind", "red", "--area", "tests", "Flaky boot", "-", "--key", "file"]);
    *call.values.get_mut(5).unwrap() = Some(Values::One(Value::Text(b"Boot twice\n".as_slice().into())));
    let call = root.submit_call(call, false);
    let w::Request::FileFinding(file) = request(&root, call) else { panic!("FileFinding") };
    assert_eq!(file.into_parts(), w::FileFindingParts { project: 3, kind: w::FindingKind::Red, severity: w::Severity::Medium, area: Some(b"tests".as_slice().into()), title: b"Flaky boot".as_slice().into(), body: b"Boot twice\n".as_slice().into() });
    root.adoption_answer(call, w::Outcome::Filed(w::Filed::new(&w::CEILINGS, w::FiledParts { number: 12 }).unwrap()));
    assert!(root.answer(call).is_none(), "the durable outcome has no holder; read it");
    deliver(&mut root, None, false);
    assert_eq!(root.answer(call).unwrap().stdout.as_ref(), b"filed: 3/12, held by task 40\n");
    let hand = root.submit(&["finding", "hand", "3/12", "--area", "docs", "words, not code", "--key", "hand"]);
    let w::Request::HandFinding(handed) = request(&root, hand) else { panic!("HandFinding") };
    assert_eq!(handed.into_parts(), w::HandFindingParts { project: 3, number: 12, area: b"docs".as_slice().into(), why: b"words, not code".as_slice().into() });
    root.adoption_answer(hand, w::Outcome::Handed); deliver(&mut root, None, false);
    assert_eq!(root.answer(hand).unwrap().stdout.as_ref(), b"handed: 3/12, held by task 40\n");
    for (argv, act, output) in [
        (&["finding", "route", "3/12", "--goal", "90", "--key", "route"][..], w::Triage::Route(w::FindingRoute::new(&w::CEILINGS, w::FindingRouteParts { task: 90 }).unwrap()), "routed: 3/12\n"),
        (&["finding", "backlog", "3/12", "--key", "backlog"][..], w::Triage::Backlog, "backlog: 3/12\n"),
        (&["finding", "drop", "shop/12", "duplicate", "--key", "drop"][..], w::Triage::Drop(w::FindingDrop::new(&w::CEILINGS, w::FindingDropParts { reason: b"duplicate".as_slice().into() }).unwrap()), "dropped: 3/12\n"),
    ] {
        let call = root.submit(argv);
        let w::Request::TriageFinding(triage) = request(&root, call) else { panic!("TriageFinding") };
        assert_eq!(triage.into_parts(), w::TriageFindingParts { project: 3, number: 12, act });
        root.adoption_answer(call, w::Outcome::Triaged);
        assert_eq!(root.answer(call).unwrap().stdout.as_ref(), output.as_bytes());
    }
}
#[test]
fn finding_reads_print_areas_triager_and_the_whole_finding() {
    for json in [false, true] {
        let mut root = root();
        let list = root.submit_call(f::call(&["finding", "list", "--project", "shop", "--state", "new", "--mine"]), json);
        deliver(&mut root, Some(w::FindingState::New), true);
        let printed = root.answer(list).unwrap();
        assert_eq!(printed.status, 0);
        if json {
            f::unique_keys(&printed.stdout).unwrap();
            assert!(f::has(&printed, br#""areas":[{"name":"tests""#));
            assert!(f::has(&printed, br#""cut":true"#));
        } else {
            assert_eq!(printed.stdout.as_ref(), "tests owner task 40, held by task 40: Tests and their fixtures\ntriager owner role 0, held by task 41\n3/12 · new · red · medium · tests · task 40 · Flaky boot\n[cut]\n".as_bytes());
        }
        let show = root.submit_prepared(f::call(&["finding", "show", "shop/12"]), json, None);
        let client::Readable::Family { scope, record, .. } = &root.family_reads.get(&show).unwrap().1 else { panic!("family read") };
        assert_eq!(*scope, c::Scope::Project(3));
        let reading = w::Reading::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&record.bytes)).unwrap();
        assert_eq!(reading.read(), &w::Read::Finding(w::FindingRead::new(&w::CEILINGS, w::FindingReadParts { project: 3, number: 12 }).unwrap()));
        let detail = w::FindingDetail::new(&w::CEILINGS, w::FindingDetailParts { row: row(), body: b"Boot twice".as_slice().into(), filer: w::Filer::Party(w::HolderParty::new(&w::CEILINGS, w::HolderPartyParts { number: 8 }).unwrap()), goal: Some(90), reason: Some(b"duplicate".as_slice().into()) }).unwrap();
        let answer = w::FindingAnswer::new(&w::CEILINGS, w::FindingAnswerParts { result: w::FindingResult::Finding(detail) }).unwrap();
        root.serve_readout(show, client::Readout::Family { family: wire::FORGE, record: Some(wire::finding_answer(&answer)) });
        let printed = root.answer(show).unwrap();
        assert_eq!(printed.status, 0);
        if json { f::unique_keys(&printed.stdout).unwrap(); assert!(f::has(&printed, br#""body":"Boot twice""#)); }
        else { assert!(f::has(&printed, b"filer: party 8\nfiled: 77\nhanded on: words, not code\ngoal: task 90\ndropped: duplicate\n\nBoot twice\n")); }
        assert_eq!(root.watches.len(), 1, "reads close the findings watch");
    }
}
#[test]
fn finding_refuses_other_forms_and_names_the_areas_on_refusal() {
    for name in ["12", "shop", "/12", "3/", "3/0", "3/12/2", "4294967296/12", "3/no"] {
        assert!(f::parse(&["finding", "drop", name, "duplicate", "--key", "drop"]).is_err(), "{name}");
    }
    let mut root = root();
    let mut invalid = f::call(&["finding", "drop", "3/12", "duplicate", "--key", "drop"]);
    *invalid.values.get_mut(0).unwrap() = Some(Values::One(Value::Text(b"12".as_slice().into())));
    let invalid = root.submit_call(invalid, false);
    assert_eq!(root.answer(invalid).unwrap().status, 1); assert!(root.nothing_sent());
    let call = root.submit(&["finding", "hand", "3/12", "--area", "missing", "wrong area", "--key", "missing"]);
    let why = w::FindingRefusal::NoArea(w::FindingAreas::new(&w::CEILINGS, w::FindingAreasParts { areas: List::from_boxed(Box::new([b"tests".as_slice().into(), b"docs".as_slice().into()])) }).unwrap());
    root.adoption_answer(call, w::Outcome::FindingRefused(w::FindingRefused::new(&w::CEILINGS, w::FindingRefusedParts { reason: why }).unwrap()));
    let printed = root.answer(call).unwrap();
    assert_eq!(printed.status, 2);
    assert!(skein_lib::bytes::find(&printed.stderr, b"no area missing in project 3; its areas: tests, docs").is_some());
    assert_eq!(crate::table::TOOLS[usize::from(rows::FINDING_SHOW_AT)].name, b"finding show");
}
