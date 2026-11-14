//! Check notifications, the real hub's ask and serialized checkout operations.
use super::{fixtures, harness::Harness};
use crate::{Event, Request, agent, host, wire};
use alloc::boxed::Box;
use skein_lib::{Decimal, Duration, Intake, List, Queue, Token, Writer, stream};
use temper_worker_domain_checkout::git::{Commit, Done, Op, Reds};
pub(crate) fn prepared() -> Harness {
    let mut items = fixtures::items(&[b"app", b"reference"]);
    items.repositories[0].access = wire::Access::Writable {
        push: Box::from(b"topic".as_slice()),
        landing: Some(Box::from(b"goal".as_slice())),
        expected: None,
    };
    prepared_items(items)
}
fn prepared_items(items: wire::Workspace) -> Harness {
    let mut h = Harness::new();
    h.env.limits.host.application_ask_bytes = 4;
    h.env.limits.host.application_answer_bytes = 64;
    let rows = h.step(Event::Assign { assignment: fixtures::assignment(true, Box::new([])), items: Ok(items) });
    let mut pending = Queue::uncapped();
    for row in rows {
        pending.push(row);
    }
    for _ in 0..64_u32 {
        let row = pending.pop().expect("prepare continues");
        if let Request::Io { owner, op, .. } = row {
            let done = if let Op::Fetch { .. } = op {
                Done::Fetched { commit: Commit::new([1; 32]) }
            } else {
                Done::Succeeded
            };
            for row in h.step(Event::Done { owner, done }) {
                pending.push(row);
            }
        } else if let Request::Decode { agent, run, start } = row {
            h.owner = Some(agent);
            h.run(Event::Decoded { agent, start: Ok(Box::new(fixtures::start(run, &start))), endpoints: Box::new([]) });
            h.say(agent::Up::Admitted);
            return h;
        }
    }
    panic!("prepared agent starts");
}
pub(crate) fn notify(h: &mut Harness, operation: agent::Operation) -> Box<[Request]> {
    let owner = h.owner.unwrap();
    crate::agents::up(
        &mut h.domain,
        &h.env,
        agent::parent::Request::Long { client: owner, operation, span: Duration::from_secs(1) },
        &mut h.out,
    );
    crate::route::hand_off(&mut h.domain, &h.env, &mut h.out);
    let mut rows = List::uncapped();
    while let Some(row) = h.out.pop() {
        rows.push(row).expect("test rows");
    }
    rows.into_boxed()
}
fn start(h: &mut Harness) -> (u64, Token) {
    let rows = notify(h, agent::Operation::Checks);
    let mut ask = None;
    let mut hold = None;
    for row in rows {
        if let Request::RedsAsk { owner, check, ask: wire::Ask::Reds } = &row {
            assert!(ask.is_none());
            ask = Some((*owner, *check));
        }
        if let Request::Io { owner, op: Op::Reds { at, action: Reds::Clear }, .. } = row {
            assert_eq!(at.repository.as_ref(), b"app");
            hold = Some(owner);
        }
    }
    let (owner, check) = ask.expect("one fresh typed ask");
    let rows = h.step(Event::RedsAsk { owner, check, bytes: Box::from(b"reds".as_slice()) });
    let mut number = None;
    for row in rows {
        if let Request::Link(host::LinkUp::Ask { number: given, run, attempt, .. }) = row {
            assert_eq!((run, attempt), (fixtures::RUN, fixtures::ATTEMPT));
            assert!(number.is_none());
            number = Some(given);
        }
    }
    (number.expect("hub ask"), hold.expect("check clears writable repository"))
}
pub(crate) fn ended(h: &mut Harness) {
    crate::agents::up(&mut h.domain, &h.env, agent::parent::Request::LongDone { client: h.owner.unwrap() }, &mut h.out);
    crate::route::hand_off(&mut h.domain, &h.env, &mut h.out);
}
fn stale(h: &mut Harness, number: u64) {
    let rows = h.step(Event::Link(host::LinkDown::Asked {
        run: fixtures::RUN,
        attempt: fixtures::ATTEMPT,
        number,
        answer: host::AskAnswer::Bytes(Box::from(b"stale".as_slice())),
    }));
    for row in rows {
        if let Request::RedsAnswer { .. } = row {
            panic!("stale answer must not decode");
        }
    }
}
fn answer(h: &mut Harness, number: u64, key: Option<&[u8]>) -> Box<[Request]> {
    let reply = match key {
        Some(_) => host::AskAnswer::Bytes(Box::from(b"answer".as_slice())),
        None => host::AskAnswer::Refused,
    };
    let rows = h.step(Event::Link(host::LinkDown::Asked {
        run: fixtures::RUN,
        attempt: fixtures::ATTEMPT,
        number,
        answer: reply,
    }));
    let mut decoded = None;
    for row in rows {
        if let Request::RedsAnswer { owner, check, .. } = row {
            decoded = Some((owner, check));
        }
    }
    match decoded {
        Some((owner, check)) => {
            let answer = match key {
                Some(key) => Some(wire::Answer::Reds {
                    reds: Box::new([wire::Red { key: Box::from(key), head: Box::new([1; 20]) }]),
                }),
                None => None,
            };
            h.step(Event::RedsAnswer { owner, check, answer })
        }
        None => Box::new([]),
    }
}
#[test]
fn each_check_asks_afresh_writes_its_own_answer_and_clears_at_long_done() {
    let mut h = prepared();
    for key in [b"test a::t first".as_slice(), b"test b::t second"] {
        let (number, hold) = start(&mut h);
        assert!(answer(&mut h, number, Some(key)).is_empty());
        let rows = h.step(Event::Done { owner: hold, done: Done::Succeeded });
        let [Request::Io { op: Op::Reds { at, action: Reds::Write { reds } }, .. }] = rows.as_ref() else {
            panic!("current reds: {rows:?}")
        };
        assert_eq!(at.repository.as_ref(), b"app");
        assert_eq!(reds.len(), 1);
        assert_eq!(reds[0].key.as_ref(), key);
        h.step(Event::Done { owner: hold, done: Done::Succeeded });
        ended(&mut h);
        let Request::Io { owner, op: Op::Reds { action: Reds::Read { .. }, .. }, .. } = h.out.pop().expect("end reads")
        else {
            panic!("end remove")
        };
        assert_eq!(owner, hold);
        h.step(Event::Done { owner: hold, done: Done::Succeeded });
    }
}
#[test]
fn refused_malformed_kept_timed_out_and_superseded_answers_write_nothing() {
    for case in 0..6_u32 {
        let mut h = prepared();
        let (number, hold) = start(&mut h);
        h.step(Event::Done { owner: hold, done: Done::Succeeded });
        match case {
            0 => {
                assert!(answer(&mut h, number, None).is_empty());
            }
            1 | 5 => {
                let rows = h.step(Event::Link(host::LinkDown::Asked {
                    run: fixtures::RUN,
                    attempt: fixtures::ATTEMPT,
                    number,
                    answer: host::AskAnswer::Bytes(Box::new([])),
                }));
                let Request::RedsAnswer { owner, check, .. } = rows.into_iter().next().expect("decode") else {
                    panic!("decode")
                };
                let answer = if case == 5 { Some(wire::Answer::Kept) } else { None };
                assert!(h.step(Event::RedsAnswer { owner, check, answer }).is_empty());
            }
            2 => {
                h.env.now = h.env.now.saturating_add(Duration::from_millis(5001));
                crate::fire(&mut h.domain, &h.env, &mut h.out);
                stale(&mut h, number);
            }
            3 => {
                ended(&mut h);
                while h.out.pop().is_some() {}
                stale(&mut h, number);
            }
            4 => {
                let (next, _) = start(&mut h);
                assert_ne!(number, next);
                stale(&mut h, number);
            }
            _ => unreachable!(),
        }
        assert!(h.out.is_empty());
    }
}
#[test]
fn other_long_operations_do_not_start_a_reds_read() {
    let mut h = prepared();
    for row in notify(&mut h, agent::Operation::Command) {
        if let Request::RedsAsk { .. } | Request::Io { op: Op::Reds { .. }, .. } = row {
            panic!("only checks read reds");
        }
    }
}

const TREE: &[u8] = b"1111111111111111111111111111111111111111";
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).unwrap();
    }
    let mut out = Writer::new(size);
    for part in parts {
        out.put(part).unwrap();
    }
    out.finish()
}
fn prefix(verdict: &[u8]) -> Box<[u8]> {
    joined(&[b"temper-check 1\ncheck ", TREE, b" - ", verdict, b"\n"])
}
fn file(verdict: &str) -> Box<[u8]> {
    joined(&[&prefix(verdict.as_bytes()), b"passed test fabrig-x::t a\nflake test fabrig-x::t b\n| boom\n"])
}
fn taken(text: &mut Intake) -> Box<[u8]> {
    let size = text.len();
    text.meet(stream::Read::Fill(size)).unwrap()
}
fn checking(h: &mut Harness) -> Token {
    let (number, hold) = start(h);
    assert!(answer(h, number, None).is_empty());
    let mut rows = h.step(Event::Done { owner: hold, done: Done::Succeeded });
    for _ in 0..2_u32 {
        if rows.is_empty() {
            return hold;
        }
        let [Request::Io { owner, op: Op::Reds { action: Reds::Clear, .. }, .. }] = rows.as_ref() else {
            panic!("clear: {rows:?}")
        };
        rows = h.step(Event::Done { owner: *owner, done: Done::Succeeded });
    }
    assert!(rows.is_empty());
    hold
}
fn reading(h: &mut Harness) {
    ended(h);
    let Some(Request::Io { op: Op::Reds { action: Reds::Read { .. }, .. }, .. }) = h.out.pop() else {
        panic!("read at check end")
    };
    assert!(h.out.is_empty());
}
fn results(rows: Box<[Request]>) -> (Token, u64, wire::CheckResults) {
    let mut found = None;
    for row in rows {
        if let Request::RedsAsk { owner, check, ask: wire::Ask::Results(report) } = row {
            assert!(found.is_none(), "one report at a time");
            found = Some((owner, check, report));
        } else if let Request::Link(host::LinkUp::Activity { .. }) = row {
        } else {
            panic!("one results ask: {row:?}");
        }
    }
    found.expect("one results ask")
}
fn sent(h: &mut Harness, owner: Token, check: u64) -> u64 {
    let rows = h.step(Event::RedsAsk { owner, check, bytes: Box::from(b"r".as_slice()) });
    let [Request::Link(host::LinkUp::Ask { number, .. })] = rows.as_ref() else { panic!("one hub ask: {rows:?}") };
    *number
}
fn kept(h: &mut Harness, number: u64) -> Box<[Request]> {
    let rows = h.step(Event::Link(host::LinkDown::Asked {
        run: fixtures::RUN,
        attempt: fixtures::ATTEMPT,
        number,
        answer: host::AskAnswer::Bytes(Box::from(b"kept".as_slice())),
    }));
    let [Request::RedsAnswer { owner, check, .. }] = rows.as_ref() else { panic!("decode kept: {rows:?}") };
    h.step(Event::RedsAnswer { owner: *owner, check: *check, answer: Some(wire::Answer::Kept) })
}
#[test]
fn passed_and_refused_checks_report_the_file_and_kept_changes_nothing() {
    let mut h = prepared();
    for (index, verdict) in ["passed", "refused"].into_iter().enumerate() {
        let hold = checking(&mut h);
        reading(&mut h);
        let rows = h.step(Event::Done { owner: hold, done: Done::Results { bytes: Some(file(verdict)) } });
        let (owner, cookie, report) = results(rows);
        assert_eq!(
            report,
            wire::CheckResults {
                tag: 0,
                check: u64::try_from(index + 1).unwrap(),
                tree: Box::new([0x11; 20]),
                base: None,
                verdict: if verdict == "passed" { wire::Verdict::Passed } else { wire::Verdict::Refused },
                passed: Box::new([Box::from(b"test fabrig-x::t a".as_slice())]),
                failed: Box::new([]),
                flakes: Box::new([wire::Flake {
                    key: Box::from(b"test fabrig-x::t b".as_slice()),
                    log: Box::from(b"boom".as_slice())
                }]),
            }
        );
        let number = sent(&mut h, owner, cookie);
        assert!(kept(&mut h, number).is_empty(), "kept writes no reds and changes no workspace");
    }
}
#[test]
fn absent_oversized_and_malformed_results_log_why_without_an_ask() {
    let prefix = prefix(b"passed");
    let mut too_many = Intake::with_capacity(4096);
    too_many.append(&prefix).unwrap();
    for i in 0..17_u64 {
        too_many.append(&joined(&[b"flake test a::t t", Decimal::of(i).as_bytes(), b"\n"])).unwrap();
    }
    let cases = [
        (None, "absent"),
        (Some(repeated(b"x", 262_145)), "oversized"),
        (Some(joined(&[b"wrong\ncheck ", TREE, b" - passed\n"])), "header"),
        (Some(joined(&[b"temper-check 1\nchecks ", TREE, b" - passed\n"])), "check line"),
        (Some(joined(&[&prefix, b"unknown x\n"])), "unknown"),
        (Some(joined(&[&prefix, b"passed test a::t ", &repeated(b"x", 503), b"\n"])), "key too long"),
        (Some(joined(&[&prefix, b"| boom\n"])), "before flake"),
        (Some(joined(&[&prefix, b"passed test a::t z\npassed test a::t a\n"])), "order"),
        (Some(joined(&[&prefix, b"passed test a::t a\nfailed test a::t a\n"])), "duplicate"),
        (Some(taken(&mut too_many)), "too many flakes"),
    ];
    for (bytes, reason) in cases {
        let mut h = prepared();
        let hold = checking(&mut h);
        reading(&mut h);
        let rows = h.step(Event::Done { owner: hold, done: Done::Results { bytes } });
        let mut ignored = None;
        for row in rows {
            if let Request::ResultsIgnored { tag: 0, reason: found, .. } = row {
                assert!(ignored.is_none());
                ignored = Some(found);
            } else if let Request::Link(host::LinkUp::Activity { .. }) = row {
            } else {
                panic!("one diagnostic, no ask for {reason}: {row:?}");
            }
        }
        let found = ignored.expect("logged reason");
        assert!(skein_lib::bytes::find(&found, reason.as_bytes()).is_some(), "{found:?} names {reason}");
    }
}
fn two_writable() -> Harness {
    let mut items = fixtures::items(&[b"app", b"other"]);
    for (repository, tag) in items.repositories.iter_mut().zip([20, 10]) {
        repository.tag = tag;
        repository.access = wire::Access::Writable {
            push: Box::from(b"topic".as_slice()),
            expected: None,
            landing: Some(Box::from(b"goal".as_slice())),
        };
    }
    prepared_items(items)
}
fn two_files(h: &mut Harness) -> (Token, u64, wire::CheckResults) {
    let hold = checking(h);
    reading(h);
    let rows = h.step(Event::Done { owner: hold, done: Done::Results { bytes: Some(file("passed")) } });
    let mut read = false;
    for row in rows {
        if let Request::Io { op: Op::Reds { at, action: Reds::Read { .. } }, .. } = row {
            assert_eq!(at.repository.as_ref(), b"other");
            read = true;
        } else if let Request::Link(host::LinkUp::Activity { .. }) = row {
        } else {
            panic!("next read before reports: {row:?}");
        }
    }
    assert!(read);
    results(h.step(Event::Done { owner: hold, done: Done::Results { bytes: Some(file("refused")) } }))
}
#[test]
fn repositories_report_in_tag_order_one_after_each_answer_or_refusal() {
    for refuse in [false, true] {
        let mut h = two_writable();
        let (owner, cookie, report) = two_files(&mut h);
        assert_eq!(report.tag, 10);
        let number = sent(&mut h, owner, cookie);
        let rows = if refuse {
            h.step(Event::Link(host::LinkDown::Asked {
                run: fixtures::RUN,
                attempt: fixtures::ATTEMPT,
                number,
                answer: host::AskAnswer::Refused,
            }))
        } else {
            kept(&mut h, number)
        };
        let (owner, cookie, report) = results(rows);
        assert_eq!(report.tag, 20);
        let number = sent(&mut h, owner, cookie);
        assert!(kept(&mut h, number).is_empty());
    }
    let mut h = two_writable();
    let (owner, cookie, _) = two_files(&mut h);
    h.env.limits.host.application_ask_bytes = 0;
    let rows = h.step(Event::RedsAsk { owner, check: cookie, bytes: Box::from(b"r".as_slice()) });
    let (owner, cookie, report) = results(rows);
    assert_eq!(report.tag, 20, "an immediate hub refusal also releases the next report");
    nothing_but_activity(h.step(Event::RedsAsk { owner, check: cookie, bytes: Box::from(b"r".as_slice()) }));
    assert!(!crate::reds::busy(&h.domain, owner));
}
#[test]
fn the_next_check_drops_unsent_reports_and_late_reads_and_decodes() {
    for encode in [false, true] {
        let mut h = two_writable();
        let (owner, cookie, _) = two_files(&mut h);
        let number = if encode { Some(sent(&mut h, owner, cookie)) } else { None };
        let rows = notify(&mut h, agent::Operation::Checks);
        let mut fresh = false;
        for row in rows {
            if let Request::RedsAsk { ask: wire::Ask::Reds, .. } = row {
                fresh = true;
            }
        }
        assert!(fresh);
        if let Some(number) = number {
            let rows = h.step(Event::Link(host::LinkDown::Asked {
                run: fixtures::RUN,
                attempt: fixtures::ATTEMPT,
                number,
                answer: host::AskAnswer::Refused,
            }));
            nothing_but_activity(rows);
        } else {
            nothing_but_activity(h.step(Event::RedsAsk { owner, check: cookie, bytes: Box::from(b"r".as_slice()) }));
        }
        nothing_but_activity(h.step(Event::RedsAnswer { owner, check: cookie, answer: Some(wire::Answer::Kept) }));
    }
    let mut h = prepared();
    let hold = checking(&mut h);
    reading(&mut h);
    notify(&mut h, agent::Operation::Checks);
    let rows = h.step(Event::Done { owner: hold, done: Done::Results { bytes: Some(file("passed")) } });
    for row in rows {
        if let Request::RedsAsk { ask: wire::Ask::Results(_), .. } = row {
            panic!("late read must be dropped");
        }
    }
}

#[test]
fn results_parser_keeps_hash_widths_schema_bounds_and_joined_log_lines() {
    let mut text = Intake::with_capacity(262_144);
    text.append(&joined(&[b"temper-check 1\ncheck ", &repeated(b"ab", 32), b" ", &repeated(b"cd", 20), b" refused\n"]))
        .unwrap();
    for i in 0..256_u64 {
        let decimal = Decimal::of(i);
        let digits = decimal.as_bytes();
        let pad = b"000".get(..3_usize.checked_sub(digits.len()).unwrap()).unwrap();
        text.append(&joined(&[b"passed test a::t ", pad, digits, &repeated(b"x", 499), b"\n"])).unwrap();
    }
    for i in 0..256_u64 {
        let decimal = Decimal::of(i);
        let digits = decimal.as_bytes();
        let pad = b"000".get(..3_usize.checked_sub(digits.len()).unwrap()).unwrap();
        text.append(&joined(&[b"failed lint p c file with spaces ", pad, digits, b"\n"])).unwrap();
    }
    text.append(&joined(&[b"flake test a::t first\n| ", &repeated(b"x", 1536), b"\n"])).unwrap();
    text.append(b"flake test a::t second\n| one\n|\n| three\n").unwrap();
    for i in 2..16_u64 {
        text.append(&joined(&[b"flake test a::t f", Decimal::of(i).as_bytes(), b"\n"])).unwrap();
    }
    let report = crate::results::parse(19, 3, &taken(&mut text)).unwrap();
    assert_eq!(report.tree.as_ref(), [0xab; 32]);
    assert_eq!(report.base.as_deref(), Some([0xcd; 20].as_slice()));
    assert_eq!(report.passed.len(), 256);
    assert_eq!(report.passed[0].len(), 512);
    assert_eq!(report.failed.len(), 256);
    assert_eq!(report.flakes.len(), 16);
    assert_eq!(report.flakes[0].log.len(), 1536);
    assert_eq!(report.flakes[1].log.as_ref(), b"one\n\nthree");
    assert!(report.flakes[2].log.is_empty());
    assert_eq!(report.verdict, wire::Verdict::Refused);
}

#[test]
fn finish_delivers_only_after_the_results_ask_reaches_the_hub() {
    let mut h = prepared();
    let hold = checking(&mut h);
    reading(&mut h);
    let call = agent::Up::Call {
        call: Token::new(1),
        name: agent::CallName { activation: fixtures::ATTEMPT.raw(), completion: 1, position: 0 },
        deadline: h.env.now.saturating_add(Duration::from_secs(10)),
        ask: agent::Ask::Deliver { fields: fixtures::fields(&[(b"title", b"checked"), (b"body", b"")]) },
    };
    let before = h.say(call);
    for row in before {
        if let Request::Io { .. } | Request::Answer { .. } = row {
            panic!("delivery must wait for report read");
        }
    }
    let rows = h.step(Event::Done { owner: hold, done: Done::Results { bytes: Some(file("passed")) } });
    let (owner, cookie, _) = results(rows);
    assert!(crate::reds::busy(&h.domain, owner), "encoding still holds delivery");
    let rows = h.step(Event::RedsAsk { owner, check: cookie, bytes: Box::from(b"r".as_slice()) });
    let [Request::Link(host::LinkUp::Ask { .. }), Request::Io { op: Op::Commit { .. }, .. }] = rows.as_ref() else {
        panic!("report reaches hub before delivery starts: {rows:?}")
    };
    let mut pending = Queue::uncapped();
    for row in rows {
        pending.push(row);
    }
    for _ in 0..32_u32 {
        let Some(row) = pending.pop() else { break };
        let rows = if let Request::Io { owner, op, .. } = row {
            let done = if let Op::Commit { .. } = op {
                Done::Committed { commit: Commit::new([2; 32]) }
            } else if let Op::Push { .. } = op {
                Done::Succeeded
            } else {
                panic!("delivery IO: {op:?}")
            };
            h.step(Event::Done { owner, done })
        } else if let Request::Process(process) = row {
            let agent::process::Request::Send { owner, .. } = *process else { panic!("delivery reply") };
            h.run(Event::Process(Box::new(agent::process::Event::Sent { owner })))
        } else if let Request::Link(host::LinkUp::Ask { .. }) = row {
            Box::new([])
        } else {
            panic!("delivery before the run's answer: {row:?}")
        };
        for row in rows {
            pending.push(row);
        }
    }
    assert!(pending.is_empty());
    let outcome = agent::Declared::new(
        smith_domain_run::outcome::Declared::Change(smith_domain_run::outcome::Change { fields: Box::new([]) }),
        1024,
    )
    .unwrap();
    let rows = h.say(agent::Up::Answer {
        answer: agent::Answer { read: None, turns: 0, spent: 0, result: agent::RunResult::Accepted { outcome } },
    });
    for row in rows {
        if let Request::Answer { .. } = row {
            panic!("cleanup still owed");
        }
    }
    let rows = h.gone();
    let mut answered = false;
    for row in &rows {
        if let Request::Answer { answer: host::RunAnswer { ending: host::Ending::Ended { .. }, .. }, .. } = row {
            answered = true;
        }
    }
    assert!(answered, "the run answers after its report and delivery: {rows:?}");
}

fn nothing_but_activity(rows: Box<[Request]>) {
    for row in rows {
        if let Request::Link(host::LinkUp::Activity { .. }) = row {
        } else {
            panic!("stale report must do nothing: {row:?}");
        }
    }
}
fn repeated(part: &[u8], count: usize) -> Box<[u8]> {
    let mut out = Writer::new(part.len().checked_mul(count).unwrap());
    for _ in 0..count {
        out.put(part).unwrap();
    }
    out.finish()
}
