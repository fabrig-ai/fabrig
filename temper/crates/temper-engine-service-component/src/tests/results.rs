//! A report is handed to the domain before kept is answered (domain/forge/reds.md, 5.6).
use super::{fixtures, reds};
use jig_domain_host as host;
use skein_lib::Token;
use temper_engine_domain as root;
use temper_worker_domain::wire as worker;

pub(super) fn report() -> worker::CheckResults {
    worker::CheckResults {
        tag: 1,
        check: 2,
        tree: Box::new([3; 20]),
        base: None,
        verdict: worker::Verdict::Passed,
        passed: Box::new([b"test fabrig-x::t a".as_slice().into()]),
        failed: Box::new([]),
        flakes: Box::new([worker::Flake {
            key: b"test fabrig-x::t b".as_slice().into(),
            log: b"boom".as_slice().into(),
        }]),
    }
}

#[test]
fn a_results_ask_queues_the_report_with_run_and_attempt_and_answers_kept_at_once() {
    let (mut service, run) = fixtures::claimed_chat();
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.events.is_empty());
    let bytes = temper_worker_service::translate::ask(worker::Ask::Results(report()));
    crate::route::request(
        engine,
        root::Request::Worker(root::WorkerRequest::Host(Box::new(root::HostRequest::Ask {
            channel: Token::new(7),
            run: Token::new(run.task),
            attempt: Token::new(run.attempt),
            number: 19,
            bytes,
        }))),
    );
    let event = engine.events.pop().expect("report first");
    let root::Event::Worker(root::WorkerInput::CheckResults { run: task, attempt, results }) = &event else {
        panic!("decoded report")
    };
    assert_eq!((task.raw(), attempt.raw()), (run.task, run.attempt));
    assert_eq!(
        results,
        &root::wire::CheckResults {
            tag: 1,
            check: 2,
            tree: Box::new([3; 20]),
            base: None,
            verdict: root::wire::Verdict::Passed,
            passed: Box::new([b"test fabrig-x::t a".as_slice().into()]),
            failed: Box::new([]),
            flakes: Box::new([root::wire::Flake {
                key: b"test fabrig-x::t b".as_slice().into(),
                log: b"boom".as_slice().into()
            }]),
        }
    );
    let Some(root::Event::Worker(root::WorkerInput::Asked {
        channel,
        run: task,
        attempt,
        number,
        answer: host::AskAnswer::Bytes(bytes),
    })) = engine.events.pop()
    else {
        panic!("encoded answer follows without a domain step")
    };
    assert_eq!((channel.raw(), task.raw(), attempt.raw(), number), (7, run.task, run.attempt, 19));
    assert_eq!(
        temper_worker_service::translate::answer(&bytes, &temper_protocol_worker_wire::v1::CEILINGS),
        Some(worker::Answer::Kept)
    );
    assert!(engine.events.is_empty());
    // The domain's report decision is an empty route in this change.
    let before = engine.domain.deployment().commits;
    engine.step(event);
    assert_eq!(engine.domain.deployment().commits, before);
}

#[test]
fn results_require_whole_asks_and_sha1_or_sha256_tree_and_base() {
    let (mut service, run) = fixtures::claimed_chat();
    for length in [0_usize, 19, 20, 21, 31, 32] {
        for base in [false, true] {
            let mut results = report();
            if base {
                results.base = Some(skein_lib::bytes::zeroed(length));
            } else {
                results.tree = skein_lib::bytes::zeroed(length);
            }
            // Exercise the other verdict and the failed list on the same path.
            results.verdict = worker::Verdict::Refused;
            results.failed = Box::new([b"test fabrig-x::t c".as_slice().into()]);
            let bytes = temper_worker_service::translate::ask(worker::Ask::Results(results));
            let answer = reds::answer(&mut service, &run, &bytes, 20);
            if length == 20 || length == 32 {
                assert_eq!(answer, host::AskAnswer::Bytes(Box::new([1])), "length {length}, base {base}");
            } else {
                assert_eq!(answer, host::AskAnswer::Refused, "length {length}, base {base}");
            }
        }
    }
    let bytes = temper_worker_service::translate::ask(worker::Ask::Results(report()));
    let trailing = skein_lib::bytes::concat(&[&bytes, &[0]]);
    for malformed in [bytes.get(..bytes.len() - 1).unwrap(), trailing.as_ref()] {
        assert_eq!(reds::answer(&mut service, &run, malformed, 21), host::AskAnswer::Refused);
    }
    // A tree or base past the schema ceiling is refused from its length prefix alone.
    for base in [false, true] {
        let mut oversized = skein_lib::Writer::new(if base { 46 } else { 21 });
        oversized.put(&[1]).unwrap();
        oversized.put(&1_u64.to_be_bytes()).unwrap();
        oversized.put(&2_u64.to_be_bytes()).unwrap();
        if base {
            oversized.put(&20_u32.to_be_bytes()).unwrap();
            oversized.put(&[3; 20]).unwrap();
            oversized.put(&[1]).unwrap();
        }
        oversized.put(&33_u32.to_be_bytes()).unwrap();
        assert_eq!(reds::answer(&mut service, &run, &oversized.finish(), 22), host::AskAnswer::Refused);
    }
}
