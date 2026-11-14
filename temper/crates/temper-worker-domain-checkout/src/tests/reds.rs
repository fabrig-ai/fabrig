use super::*;
use crate::git::Reds;
#[test]
fn prepare_marks_only_writable_repositories_after_their_pins_and_ignores_file_failures() {
    let mut h = Harness::new(LIMITS);
    h.observe_reds = true;
    let (hold, mut op) = h.begin(1, two(b"reds"));
    let mut pinned = false;
    let mut marked = 0_u32;
    for _ in 0..32_u32 {
        let done = match &op {
            Op::Reds { at, action: Reds::Prepare { wait_ms, rerun_processes } } => {
                assert!(pinned);
                assert_eq!(at.repository.as_ref(), b"a");
                assert_eq!(*wait_ms, 6000);
                assert_eq!(*rerun_processes, 4);
                marked += 1;
                Done::Failed { fault: Fault::Broken }
            }
            Op::Point { into: Into::Landing, .. } => {
                pinned = true;
                Done::Succeeded
            }
            Op::Fetch { at, .. } => Done::Fetched { commit: start_of(at) },
            Op::Store { .. } | Op::Make { .. } | Op::Clone { .. } | Op::CheckOut { .. } => Done::Succeeded,
            other @ (Op::Reds { .. }
            | Op::Look { .. }
            | Op::Point { .. }
            | Op::Create { .. }
            | Op::IsAncestor { .. }
            | Op::Brings { .. }
            | Op::DiffPaths { .. }
            | Op::Merge { .. }
            | Op::Commit { .. }
            | Op::Push { .. }) => panic!("unexpected prepare: {other:?}"),
        };
        match h.one(Event::Done { owner: hold, done }) {
            Request::Io { op: next, .. } => op = next,
            Request::Prepared { prepared: Prepared::Ready { .. }, .. } => {
                assert_eq!(marked, 1);
                return;
            }
            other @ (Request::Results { .. }
            | Request::ResultsEnd { .. }
            | Request::Held { .. }
            | Request::Prepared { .. }
            | Request::Pushed { .. }
            | Request::Saved { .. }
            | Request::Released { .. }
            | Request::Cancel { .. }) => panic!("prepare survives optional IO failure: {other:?}"),
        }
    }
    panic!("prepare settles");
}
#[test]
fn an_ended_check_clears_an_inflight_write_before_the_delivery_can_start() {
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"reds"));
    let reds = Box::new([crate::git::Red { key: bytes(b"test a::t x"), head: Box::new(commit(1).raw()) }]);
    let Request::Io { op: Op::Reds { action: Reds::Write { .. }, .. }, .. } =
        h.one(Event::Reds { hold, action: Reds::Write { reds } })
    else {
        panic!("write")
    };
    assert_eq!(h.one(Event::Reds { hold, action: Reds::Clear }), Request::Cancel { owner: hold });
    h.none(Event::Push { hold, message: message() });
    let Op::Reds { action: Reds::Clear, .. } = h.next(hold, Done::Succeeded) else { panic!("clear") };
    let Op::Commit { .. } = h.next(hold, Done::Succeeded) else { panic!("delivery after clear") };
}
#[test]
fn a_new_check_discards_old_pending_reds_and_finishes_its_clear_before_writing() {
    let mut h = Harness::new(LIMITS);
    let hold = h.ready(1, one(b"reds"));
    let Request::Io { op: Op::Reds { action: Reds::Clear, .. }, .. } = h.one(Event::Reds { hold, action: Reds::Clear })
    else {
        panic!("clear")
    };
    for action in [
        Reds::Write { reds: Box::new([crate::git::Red { key: bytes(b"old"), head: Box::new(commit(1).raw()) }]) },
        Reds::Clear,
    ] {
        assert!(h.step(Event::Reds { hold, action }).len() <= 1);
    }
    h.none(Event::Reds {
        hold,
        action: Reds::Write {
            reds: Box::new([crate::git::Red { key: bytes(b"new"), head: Box::new(commit(2).raw()) }]),
        },
    });
    let Op::Reds { action: Reds::Clear, .. } = h.next(hold, Done::Succeeded) else { panic!("fresh clear") };
    let Op::Reds { action: Reds::Write { reds }, .. } = h.next(hold, Done::Succeeded) else { panic!("new write") };
    assert_eq!(reds[0].key.as_ref(), b"new");
    h.none(Event::Done { owner: hold, done: Done::Failed { fault: Fault::Broken } });
    let Request::Io { op: Op::Reds { action: Reds::Clear, .. }, .. } = h.one(Event::Reds { hold, action: Reds::Clear })
    else {
        panic!("failure does not stop next check")
    };
}

#[test]
fn results_reads_report_each_repository_before_the_queued_delivery() {
    let mut h = Harness::new(LIMITS);
    let mut spec = two(b"results");
    spec.repositories[1].push = Some(bytes(b"topic"));
    let hold = h.ready(1, spec);
    let Request::Io { op: Op::Reds { action: Reds::Write { .. }, .. }, .. } =
        h.one(Event::Reds { hold, action: Reds::Write { reds: Box::new([]) } })
    else {
        panic!("write")
    };
    assert_eq!(h.one(Event::Reds { hold, action: Reds::Read { check: 7 } }), Request::Cancel { owner: hold });
    h.none(Event::Push { hold, message: message() });
    let Op::Reds { at, action: Reds::Read { check: 7 } } = h.next(hold, Done::Succeeded) else {
        panic!("read before push")
    };
    assert_eq!(at.repository.as_ref(), b"a");
    let rows = h.step(Event::Done { owner: hold, done: Done::Results { bytes: Some(bytes(b"a results")) } });
    let [
        Request::Results { client, check: 7, repository: 0, bytes: Some(file) },
        Request::Io { op: Op::Reds { at, action: Reds::Read { check: 7 } }, .. },
    ] = rows.as_slice()
    else {
        panic!("first file and next read: {rows:?}")
    };
    assert_eq!(*client, Token::new(1));
    assert_eq!(file.as_ref(), b"a results");
    assert_eq!(at.repository.as_ref(), b"b");
    let rows = h.step(Event::Done { owner: hold, done: Done::Results { bytes: None } });
    let [
        Request::Results { check: 7, repository: 1, bytes: None, .. },
        Request::ResultsEnd { check: 7, .. },
        Request::Io { op: Op::Commit { .. }, .. },
    ] = rows.as_slice()
    else {
        panic!("read ends before delivery: {rows:?}")
    };
}
